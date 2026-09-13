use crate::dsp::delayline::DelayLine;
use crate::dsp::onepole::OnePole;
use crate::dsp::simper::Filter;
use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use crate::effect::*;
use crate::worker::RequestData;

const N: usize = 16;

// Tap tables are in samples at this rate, scaled to whatever we run at.
const TABLE_RATE: f32 = 44100.0;

// early reflections
#[rustfmt::skip]
const LEN1_L: [f32; N] = [1., 178., 355., 936., 1386., 1708., 2051., 2219., 2725., 3126., 3404., 3880., 4092., 4302., 4906., 5114.];
const AMP_L: [f32; N] = [
	0.48236465,
	0.66972786,
	-0.42255732,
	-0.16075869,
	0.13266747,
	0.11500361,
	0.29223543,
	-0.24628632,
	-0.07415399,
	-0.12279022,
	0.05522189,
	-0.13350953,
	0.12158318,
	-0.11085732,
	0.02856836,
	-0.03007264,
];

#[rustfmt::skip]
const LEN1_R: [f32; N] = [1., 254., 507., 1054., 1095., 1674., 1772., 2387., 2689., 2879., 3205., 3658., 4053., 4298., 4919., 5130.];
const AMP_R: [f32; N] = [
	0.53873825,
	-0.7236128,
	-0.39662176,
	-0.17102939,
	0.18526316,
	-0.13042036,
	0.12482437,
	-0.23625538,
	-0.12563056,
	-0.22301023,
	-0.20006062,
	-0.16456714,
	0.046402864,
	-0.041546308,
	0.032239176,
	-0.04559372,
];

// diffusion
#[rustfmt::skip]
const LEN2_L: [f32; N] = [60., 88., 276., 434., 632., 710., 727., 840., 861., 862., 915., 943., 1068., 1080., 1093., 1197.];
#[rustfmt::skip]
const SIGN_L: [f32; N] = [1., -1., 1., -1., 1., 1., -1., 1., -1., 1., -1., -1., 1., -1., -1., -1.];
#[rustfmt::skip]
const LEN2_R: [f32; N] = [2., 125., 272., 350., 374., 657., 725., 859., 904., 955., 985., 1050., 1071., 1092., 1101.,1134.,];
#[rustfmt::skip]
const SIGN_R: [f32; N] = [1., -1., 1., 1., -1., 1., 1., -1., 1., -1., -1., 1., -1., -1., 1., -1.];

// feedback delay network
#[rustfmt::skip]
const LEN3: [f32; N] = [4229., 4391., 4721., 4621., 4871., 5297., 5323., 5741., 5897., 6421., 6701., 7069., 7103., 7669., 8039., 8353.];

// buffer sizes in samples at TABLE_RATE
const LINE_LEN: f32 = 16_000.;
const DIFFUSE_LEN: f32 = 1600.;
const FDN_LEN: f32 = 9000.;

// input lowpass, a butterworth pair
const INPUT_CUTOFF: f32 = 15_000.0;
const SHELF_CUTOFF: f32 = 300.0;

#[derive(Debug)]
pub struct ReverbRoom {
	sample_rate: f32,
	// tap tables are written for TABLE_RATE
	sr_scale: f32,

	line_l: DelayLine,
	line_r: DelayLine,
	diffuse: [DelayLine; N],
	fdn: [DelayLine; N],

	lp1_l: Filter,
	lp1_r: Filter,
	lp2_l: Filter,
	lp2_r: Filter,

	damp: [OnePole; N],
	shelf: [OnePole; N],
	fb_gain: [f32; N],

	s: [f32; N],
	s2: [f32; N],

	balance: Smooth,
	size: Smooth,
	mix_early: Smooth,
	mix_diffuse: Smooth,
	mix_late: Smooth,

	decay: f32,
	dark: f32,
}

fn fdn_stretch(size: f32) -> f32 {
	0.4 + 0.6 * size
}

#[inline]
fn tap(len: f32, sr_scale: f32) -> isize {
	(len * sr_scale + 0.5) as isize
}

// In place fast Walsh-Hadamard, normalized so the matrix is orthonormal.
fn hadamard(x: &mut [f32; N]) {
	let mut h = 1;
	while h < N {
		let mut i = 0;
		while i < N {
			for j in i..i + h {
				let a = x[j];
				let b = x[j + h];
				x[j] = a + b;
				x[j + h] = a - b;
			}
			i += h * 2;
		}
		h *= 2;
	}
	// 1 / sqrt(N)
	const G: f32 = 0.25;
	for v in x.iter_mut() {
		*v *= G;
	}
}

impl ReverbRoom {
	// Size stretches the fdn delays, so the gain that gives the requested t60
	// has to be recomputed whenever either changes.
	fn update_feedback(&mut self) {
		let stretch = fdn_stretch(self.size.target());
		for i in 0..N {
			let len = LEN3[i] * stretch * self.sr_scale;
			let gain = (-60.0 * len) / (self.decay * self.sample_rate);
			self.fb_gain[i] = from_db(gain);

			self.damp[i].set_lowpass(INPUT_CUTOFF - 10_000.0 * self.dark);
			self.shelf[i].set_lowshelf(SHELF_CUTOFF, gain * 0.5);
		}
	}
}

impl Effect for ReverbRoom {
	fn new(sample_rate: f32) -> Self {
		let sr_scale = sample_rate / TABLE_RATE;
		let samples = |n: f32| (n * sr_scale) as usize;

		let mut lp1_l = Filter::new(sample_rate);
		let mut lp1_r = Filter::new(sample_rate);
		let mut lp2_l = Filter::new(sample_rate);
		let mut lp2_r = Filter::new(sample_rate);
		lp1_l.set_lowpass(INPUT_CUTOFF, BUTTERWORTH_4_Q1);
		lp1_r.set_lowpass(INPUT_CUTOFF, BUTTERWORTH_4_Q1);
		lp2_l.set_lowpass(INPUT_CUTOFF, BUTTERWORTH_4_Q2);
		lp2_r.set_lowpass(INPUT_CUTOFF, BUTTERWORTH_4_Q2);

		let mut new = ReverbRoom {
			sample_rate,
			sr_scale,

			line_l: DelayLine::new_samples(sample_rate, samples(LINE_LEN)),
			line_r: DelayLine::new_samples(sample_rate, samples(LINE_LEN)),
			diffuse: std::array::from_fn(|_| {
				DelayLine::new_samples(sample_rate, samples(DIFFUSE_LEN))
			}),
			fdn: std::array::from_fn(|_| DelayLine::new_samples(sample_rate, samples(FDN_LEN))),

			lp1_l,
			lp1_r,
			lp2_l,
			lp2_r,

			damp: std::array::from_fn(|_| OnePole::new(sample_rate)),
			shelf: std::array::from_fn(|_| OnePole::new(sample_rate)),
			fb_gain: [0.5; N],

			s: [0.; N],
			s2: [0.; N],

			balance: Smooth::new(1., 25., sample_rate),
			size: Smooth::new(0.5, 200., sample_rate),
			mix_early: Smooth::new(1., 25., sample_rate),
			mix_diffuse: Smooth::new(0., 25., sample_rate),
			mix_late: Smooth::new(0.5, 25., sample_rate),

			decay: 1.0,
			dark: 0.1,
		};
		new.update_feedback();
		new
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for (out_l, out_r) in bl.iter_mut().zip(br.iter_mut()) {
			let input_l = *out_l;
			let input_r = *out_r;

			let size = self.size.process();
			let early_stretch = 0.4 + 1.6 * size;
			let diffuse_stretch = 0.5 + 0.5 * size;
			let fdn_stretch = fdn_stretch(size);

			let balance = self.balance.process();
			let mix_early = self.mix_early.process();
			let mix_diffuse = self.mix_diffuse.process();
			let mix_late = self.mix_late.process();

			let in_l = self.lp2_l.process(self.lp1_l.process(input_l));
			let in_r = self.lp2_r.process(self.lp1_r.process(input_r));

			self.line_l.push(in_l);
			self.line_r.push(in_r);

			// early reflections, mono sum for next stage
			let mut early_l = 0.;
			let mut early_r = 0.;
			for i in 0..N {
				let tl = (LEN1_L[i] - 1.) * early_stretch + 1.;
				let tr = (LEN1_R[i] - 1.) * early_stretch + 1.;
				let l = self.line_l.go_back_int_s(tap(tl, self.sr_scale)) * AMP_L[i];
				let r = self.line_r.go_back_int_s(tap(tr, self.sr_scale)) * AMP_R[i];

				early_l += l;
				early_r += r;
				self.s[i] = 0.5 * (l + r);
			}
			hadamard(&mut self.s);

			// velvet noise diffusion
			let mut diffuse_l = 0.;
			let mut diffuse_r = 0.;
			for i in 0..N {
				self.diffuse[i].push(self.s[i]);
				let tl = (LEN2_L[i] - 1.) * diffuse_stretch + 1.;
				let tr = (LEN2_R[i] - 1.) * diffuse_stretch + 1.;
				diffuse_l += SIGN_L[i] * self.diffuse[i].go_back_int_s(tap(tl, self.sr_scale));
				diffuse_r += SIGN_R[i] * self.diffuse[i].go_back_int_s(tap(tr, self.sr_scale));
			}

			// feedback delay network
			for i in 0..N {
				let mut v = self.fdn[i].go_back_int_s(tap(LEN3[i] * fdn_stretch, self.sr_scale));
				v = self.damp[i].process(v);
				v = self.shelf[i].process(v);
				self.s2[i] = self.fb_gain[i] * v;
			}
			hadamard(&mut self.s2);

			for i in 0..N {
				self.fdn[i].push(self.s[i] + self.s2[i]);
			}

			let mut wet_l = early_l * mix_early + diffuse_l * mix_diffuse;
			let mut wet_r = early_r * mix_early + diffuse_r * mix_diffuse;

			// late reflections from FDN output
			let late_l = 3. * self.s2[0];
			let late_r = 3. * self.s2[1];
			wet_l = wet_l * (1. - mix_late) + late_l * mix_late;
			wet_r = wet_r * (1. - mix_late) + late_r * mix_late;

			*out_l = lerp(input_l, wet_l, balance);
			*out_r = lerp(input_r, wet_r, balance);
		}
	}

	fn flush(&mut self) {
		self.line_l.flush();
		self.line_r.flush();
		for d in &mut self.diffuse {
			d.flush();
		}
		for d in &mut self.fdn {
			d.flush();
		}

		for f in [&mut self.lp1_l, &mut self.lp1_r, &mut self.lp2_l, &mut self.lp2_r] {
			f.reset_state();
			f.immediate();
		}
		for f in &mut self.damp {
			f.reset_state();
			f.immediate();
		}
		for f in &mut self.shelf {
			f.reset_state();
			f.immediate();
		}

		self.s = [0.; N];
		self.s2 = [0.; N];

		self.balance.immediate();
		self.size.immediate();
		self.mix_early.immediate();
		self.mix_diffuse.immediate();
		self.mix_late.immediate();
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.balance.set(value),
			1 => {
				self.size.set(value);
				self.update_feedback();
			},
			2 => {
				// equal power, so diffusion does not change the level
				self.mix_early.set((1. - value).sqrt());
				self.mix_diffuse.set(value.sqrt());
			},
			3 => {
				self.decay = value;
				self.update_feedback();
			},
			4 => self.mix_late.set(value),
			5 => {
				self.dark = value;
				self.update_feedback();
			},
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
