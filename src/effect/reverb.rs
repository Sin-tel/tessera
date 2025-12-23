use crate::dsp::delayline::DelayLine;
use crate::dsp::onepole::OnePole;
use crate::dsp::smooth::{LinearBuffer, Smooth};
use crate::dsp::*;
use crate::effect::*;
use crate::worker::RequestData;
use std::iter::zip;

// max length in seconds
const MAX_LEN: f32 = 0.2;

const LENGTHS: [f32; 4] = [0.060929704, 0.09861678, 0.17113379, 0.11852608];

const MAX_AP_LEN: f32 = 0.01;

const AP_LEN_L: [f32; 3] = [0.0032199547, 0.002426304, 0.008594104];

const AP_LEN_R: [f32; 3] = [0.006281179, 0.007868481, 0.002743764];

const CUTOFF: f32 = 5000.0;

#[derive(Debug)]
pub struct Reverb {
	sample_rate: f32,

	delaylines: [DelayLine; 4],
	allpass_l: [DelayLine; 3],
	allpass_r: [DelayLine; 3],

	filter1: OnePole,
	filter2: OnePole,

	pre_l: DelayLine,
	pre_r: DelayLine,

	lfo1: LinearBuffer,
	lfo2: LinearBuffer,
	accum1: f32,
	accum2: f32,

	balance: Smooth,
	size: Smooth,
	pre_delay: Smooth,
	decay: f32,
	feedback: Smooth,
	mod_amount: f32,
}

impl Effect for Reverb {
	fn new(sample_rate: f32) -> Self {
		let delaylines = std::array::from_fn(|_| DelayLine::new(sample_rate, MAX_LEN));
		let allpass_l = std::array::from_fn(|_| DelayLine::new(sample_rate, MAX_AP_LEN));
		let allpass_r = std::array::from_fn(|_| DelayLine::new(sample_rate, MAX_AP_LEN));

		let mut filter1 = OnePole::new(sample_rate);
		filter1.set_highshelf(CUTOFF, -3.);
		let mut filter2 = OnePole::new(sample_rate);
		filter2.set_highshelf(CUTOFF, -3.);

		Reverb {
			sample_rate,
			delaylines,
			allpass_l,
			allpass_r,
			pre_l: DelayLine::new(sample_rate, 0.100),
			pre_r: DelayLine::new(sample_rate, 0.100),
			filter1,
			filter2,
			lfo1: LinearBuffer::new(0.),
			lfo2: LinearBuffer::new(0.),
			accum1: 0.,
			accum2: 0.,
			balance: Smooth::new(0., 25., sample_rate),
			feedback: Smooth::new(0., 25., sample_rate),
			size: Smooth::new(0., 200., sample_rate),
			pre_delay: Smooth::new(0., 200., sample_rate),
			decay: 0.1,
			mod_amount: 0.,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		let n = bl.len();

		// update LFOs
		self.accum1 += 0.851 * n as f32 / self.sample_rate;
		self.accum1 -= self.accum1.floor();
		self.lfo1.set(1.0 + 0.010 * self.mod_amount * sin_cheap(self.accum1));
		self.lfo1.process_block(n);

		self.accum2 += 0.497 * n as f32 / self.sample_rate;
		self.accum2 -= self.accum2.floor();
		self.lfo2.set(1.0 + 0.016 * self.mod_amount * sin_cheap(self.accum2));
		self.lfo2.process_block(n);

		for (j, (l, r)) in zip(bl.iter_mut(), br.iter_mut()).enumerate() {
			let pre_delay = self.pre_delay.process();
			let mut sl = self.pre_l.go_back_cubic(pre_delay);
			let mut sr = self.pre_r.go_back_cubic(pre_delay);

			self.pre_l.push(*l);
			self.pre_r.push(*r);

			let k_ap = 0.6;

			// allpass diffusion on inputs
			for (i, v) in self.allpass_l.iter_mut().enumerate() {
				sl = v.allpass(sl, k_ap, AP_LEN_L[i]);
			}
			for (i, v) in self.allpass_r.iter_mut().enumerate() {
				sr = v.allpass(sr, k_ap, AP_LEN_R[i]);
			}

			// update FDN
			let size = self.size.process();
			let d = [
				self.delaylines[0].go_back_cubic(LENGTHS[0] * size),
				self.delaylines[1].go_back_cubic(LENGTHS[1] * size),
				self.delaylines[2].go_back_cubic(LENGTHS[2] * size * self.lfo1.get(j)),
				self.delaylines[3].go_back_cubic(LENGTHS[3] * size * self.lfo2.get(j)),
			];

			// Hadamard matrix
			let mut s = [
				d[0] + d[1] + d[2] + d[3] + sl,
				d[0] - d[1] + d[2] - d[3] + sr,
				d[0] + d[1] - d[2] - d[3] + sl,
				d[0] - d[1] - d[2] + d[3] + sr,
			];

			s[0] = self.filter1.process(s[0]);
			s[1] = self.filter2.process(s[1]);

			let gain = self.feedback.process() * 0.5;
			for (i, v) in self.delaylines.iter_mut().enumerate() {
				v.push(s[i] * gain);
			}

			let out_l = 0.5 * sl + d[0];
			let out_r = 0.5 * sr + d[1];

			let balance = self.balance.process();
			*l = lerp(*l, out_l, balance);
			*r = lerp(*r, out_r, balance);
		}
	}
	fn flush(&mut self) {
		for d in &mut self.delaylines {
			d.flush();
		}
		for d in &mut self.allpass_l {
			d.flush();
		}
		for d in &mut self.allpass_r {
			d.flush();
		}
		self.pre_l.flush();
		self.pre_r.flush();

		self.filter1.reset_state();
		self.filter2.reset_state();
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.balance.set(value),
			1 => {
				self.size.set(value);
				self.update_feedback();
			},
			2 => {
				self.decay = value;
				self.update_feedback();
			},
			3 => self.mod_amount = value,
			4 => self.pre_delay.set(value),
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}

impl Reverb {
	fn update_feedback(&mut self) {
		if self.decay > 15. {
			// freeze
			self.feedback.set(1.0);

			self.filter1.set_highshelf(CUTOFF, 0.);
			self.filter2.set_highshelf(CUTOFF, 0.);
		} else {
			// decay is time to -60 dB
			let avg_len = 0.5 * MAX_LEN;

			let coef = (-60. * avg_len * self.size.target()) / self.decay;
			self.feedback.set(from_db(coef));

			self.filter1.set_highshelf(CUTOFF, 2. * coef);
			self.filter2.set_highshelf(CUTOFF, 2. * coef);
		}
	}
}
