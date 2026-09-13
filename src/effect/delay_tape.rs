use crate::dsp::simper::Filter;
use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use crate::effect::*;
use crate::worker::RequestData;
use halfband::iir::{Downsampler8, Upsampler8};

const REF_RATE: f32 = 44_100.0;
const LOOP_LEN: f32 = 0.91; // seconds
const HEAD_MOD_DEPTH: f32 = 0.0091; // seconds

// Read head positions, relative to total length
const HEAD1: f32 = 0.270;
const HEAD2: f32 = 0.600;
const HEAD3: f32 = 0.951;

const LFO_RATE: f32 = 0.27;

// Magnetic model parameters
const SAT_C: f32 = 0.5;
const SAT_ALPHA: f32 = 0.1;
const SAT_MAG: f32 = 0.28;
const SAT_K: f32 = 1.0;

const GAIN_PAIR: f32 = 0.7;

const TRIM_GAIN: f32 = 0.31;
const BIAS: f32 = 0.01;
const NOISE_AMT: f32 = 0.01;

// wow/flutter rate constants, per sample
const FLUTTER_TRIM_REF: f32 = 0.00581;
const SWEEP_TRIM_REF: f32 = 0.000079;

// Steady state of the hysteresis model.
fn magnetic_fixed_point(x0: f32) -> (f32, f32) {
	let k2 = 1.0 - (-0.001f32.sqrt() / SAT_K).exp();
	let mut m = 0.0f32;
	let mut m_irr = 0.0f32;
	for _ in 0..512 {
		let m_anh = ((x0 + SAT_ALPHA * m) / SAT_MAG).tanh();
		m_irr += k2 * (m_anh - m_irr);
		m = SAT_C * (m_anh - m_irr) + m_irr;
	}
	(m, m_irr)
}

#[derive(Debug)]
struct TapeBuffer {
	buf: Vec<f32>,
	len: usize,
	head: usize,
}

impl TapeBuffer {
	fn new(len: usize) -> Self {
		Self { buf: vec![0.0; len], len, head: 0 }
	}

	fn push(&mut self, x: f32) {
		self.buf[self.head] = x;
		self.head = (self.head + 1) % self.len;
	}

	fn read(&self, h: f32) -> f32 {
		let h_int = fast_floor(h) as i64;
		let frac = h - h_int as f32;
		let len = self.len as i64;
		let at = |o: i64| self.buf[(h_int + o).rem_euclid(len) as usize];

		hermite4(at(-1), at(0), at(1), at(2), frac)
	}

	fn fill(&mut self, x: f32) {
		self.buf.fill(x);
		self.head = 0;
	}
}

#[derive(Clone, Copy)]
struct Shared {
	h1_gain: f32,
	h2_gain: f32,
	h3_gain: f32,
	wow: f32,
	balance: f32,
	feedback: f32,
	drive: f32,
	drive_inv: f32,
	base_speed: f32,
	flutter_term: f32,
	jitter: f32,
	loop_len: f32,
	head_mod_depth: f32,
}

#[derive(Debug)]
struct Track {
	phase_offset: f32,

	accum: f32,
	x1: f32,
	x2: f32,
	x3: f32,
	buffer: TapeBuffer,
	m: f32,
	m_irr: f32,
	prev_t: f32,
	lfo_phase: f32,

	pre_lp1: Filter,
	pre_lp2: Filter,
	post_lp1: Filter,
	post_lp2: Filter,
	noise_filter: Filter,
	fb_filter: Filter,
	fb_filter2: Filter,
	high: Filter,

	upsampler: Upsampler8,
	downsampler: Downsampler8,

	prev: f32,
}

impl Track {
	fn new(sample_rate: f32, loop_len_samples: usize, left: bool) -> Self {
		let mut noise_filter = Filter::new(sample_rate);
		noise_filter.set_lowpass(850.0, 0.7);
		noise_filter.immediate();

		let mut fb_filter = Filter::new(sample_rate);
		fb_filter.set_highpass(200.0, 0.7);
		fb_filter.immediate();

		let mut fb_filter2 = Filter::new(sample_rate);
		fb_filter2.set_highshelf(2700.0, 0.7, -3.0);
		fb_filter2.immediate();

		let mut high = Filter::new(sample_rate);
		high.set_highpass(10.0, 0.7);
		high.immediate();

		// These run at 2x the outer rate, once per oversampled tick.
		let mut pre_lp1 = Filter::new(2.0 * sample_rate);
		let mut pre_lp2 = Filter::new(2.0 * sample_rate);
		let mut post_lp1 = Filter::new(2.0 * sample_rate);
		let mut post_lp2 = Filter::new(2.0 * sample_rate);
		pre_lp1.set_lowpass(2000.0, BUTTERWORTH_4_Q1);
		pre_lp2.set_lowpass(2000.0, BUTTERWORTH_4_Q2);
		post_lp1.set_lowpass(2000.0, BUTTERWORTH_4_Q1);
		post_lp2.set_lowpass(2000.0, BUTTERWORTH_4_Q2);
		pre_lp1.immediate();
		pre_lp2.immediate();
		post_lp1.immediate();
		post_lp2.immediate();

		Track {
			phase_offset: if left { 0.5 * std::f32::consts::PI } else { 0.0 },

			accum: 0.0,
			x1: 0.0,
			x2: 0.0,
			x3: 0.0,
			buffer: TapeBuffer::new(loop_len_samples),
			m: 0.0,
			m_irr: 0.0,
			prev_t: 0.0,
			lfo_phase: 0.0,

			pre_lp1,
			pre_lp2,
			post_lp1,
			post_lp2,
			noise_filter,
			fb_filter,
			fb_filter2,
			high,

			upsampler: Upsampler8::default(),
			downsampler: Downsampler8::default(),

			prev: 0.0,
		}
	}

	fn process_sample(&mut self, sample: &mut f32, shared: &Shared) {
		let in_s = *sample;
		let s0 = in_s + self.prev * shared.feedback;
		let mut s = s0 * TRIM_GAIN;

		s = self.fb_filter.process(s);
		s = self.fb_filter2.process(s);
		s *= 1.0 + fastrand::f32() * NOISE_AMT;
		s += BIAS;

		let [u1, u2] = self.upsampler.process(s);

		let jitter_noise = self.noise_filter.process(fastrand::f32() - 0.5);
		let speed =
			shared.base_speed * (1.0 + 0.5 * shared.jitter * jitter_noise + shared.flutter_term);

		let t1 = self.tick_tape(u1, speed, shared);
		let t2 = self.tick_tape(u2, speed, shared);

		let mut out = self.downsampler.process(t1, t2);
		out = self.high.process(out);

		self.prev = out;
		*sample = in_s * (1.0 - shared.balance) + out * shared.balance;
	}

	fn tick_magnetic(&mut self, s: f32) -> f32 {
		let diff = s - self.prev_t;
		self.prev_t = s;

		let m_anh = ((s + SAT_ALPHA * self.m) / SAT_MAG).tanh();

		let k2 = (diff * diff + 0.001).sqrt() / SAT_K;
		let k2 = 1.0 - f32::exp(-k2);
		self.m_irr += k2 * (m_anh - self.m_irr);

		let m_rev = SAT_C * (m_anh - self.m_irr);
		self.m = m_rev + self.m_irr;
		self.m
	}

	fn tick_tape(&mut self, x_in: f32, speed: f32, shared: &Shared) -> f32 {
		let x = self.pre_lp2.process(self.pre_lp1.process(x_in));

		self.accum += speed;

		let m = self.tick_magnetic(x * shared.drive);
		let x_tape = m * shared.drive_inv;

		while self.accum >= 1.0 {
			self.accum -= 1.0;
			let a = self.accum / (speed + 1e-6);
			let x_interp = hermite4(x_tape, self.x1, self.x2, self.x3, a);
			self.buffer.push(x_interp);
		}
		self.x3 = self.x2;
		self.x2 = self.x1;
		self.x1 = x_tape;

		let read = self.buffer.head as f32 + self.accum;

		self.lfo_phase += LFO_RATE * speed / shared.loop_len;

		let depth = shared.head_mod_depth * shared.wow;
		let h1_mod = depth * sin_cheap(self.lfo_phase * 1.53 + self.phase_offset);
		let h2_mod = depth * sin_cheap(self.lfo_phase * 1.21 + self.phase_offset);
		let h3_mod = depth * sin_cheap(self.lfo_phase * 0.97 + self.phase_offset);

		let h1 = self.buffer.read(read - shared.loop_len * HEAD1 + h1_mod);
		let h2 = self.buffer.read(read - shared.loop_len * HEAD2 + h2_mod);
		let h3 = self.buffer.read(read - shared.loop_len * HEAD3 + h3_mod);

		let out = h1 * shared.h1_gain + h2 * shared.h2_gain + h3 * shared.h3_gain;

		self.post_lp2.process(self.post_lp1.process(out))
	}
}

#[derive(Debug)]
pub struct DelayTape {
	tracks: [Track; 2],

	inv_sr_scale: f32,
	loop_len: f32,
	head_mod_depth: f32,

	speed_filter: Filter,
	speed_target: f32,

	h1_gain: Smooth,
	h2_gain: Smooth,
	h3_gain: Smooth,
	wow: Smooth,
	balance: Smooth,
	feedback: Smooth,
	drive: Smooth,

	jitter: f32,
	flutter_depth: f32,
	flutter_rate: Smooth,
	next_rate: f32,
	sweep: f32,
	flutter_trim: f32,
}

impl DelayTape {
	fn step_flutter(&mut self) -> f32 {
		let flutter_rate = self.flutter_rate.process();
		self.sweep += flutter_rate * self.flutter_trim;

		let sweep_trim = SWEEP_TRIM_REF * self.flutter_depth * self.inv_sr_scale;
		self.sweep += self.sweep * sweep_trim;

		if self.sweep >= 1.0 {
			self.sweep = 0.0;
			self.next_rate = 0.02 + fastrand::f32() * 0.98;
			self.flutter_rate.set(self.next_rate);
		}

		0.03 * self.flutter_depth * sin_cheap(self.sweep)
	}
}

impl Effect for DelayTape {
	fn new(sample_rate: f32) -> Self {
		let sr_scale = sample_rate / REF_RATE;
		let inv_sr_scale = 1.0 / sr_scale;
		let loop_len_samples = (LOOP_LEN * sample_rate).round() as usize;
		let loop_len = loop_len_samples as f32;
		let head_mod_depth = HEAD_MOD_DEPTH * sample_rate;

		let mut speed_filter = Filter::new(sample_rate);
		speed_filter.set_lowpass(0.6, 0.5);
		speed_filter.immediate();

		DelayTape {
			tracks: [
				Track::new(sample_rate, loop_len_samples, true),
				Track::new(sample_rate, loop_len_samples, false),
			],

			inv_sr_scale,
			loop_len,
			head_mod_depth,

			speed_filter,
			speed_target: 1.0,

			h1_gain: Smooth::new(0., 25., sample_rate),
			h2_gain: Smooth::new(0., 25., sample_rate),
			h3_gain: Smooth::new(1., 25., sample_rate),
			wow: Smooth::new(0.09, 25., sample_rate),
			balance: Smooth::new(0.5, 25., sample_rate),
			feedback: Smooth::new(0.4, 25., sample_rate),
			drive: Smooth::new(1., 25., sample_rate),

			jitter: 0.25,
			flutter_depth: 0.25,
			flutter_rate: Smooth::new(0.5, 25., sample_rate),
			next_rate: 0.5,
			sweep: 0.0,
			flutter_trim: FLUTTER_TRIM_REF * inv_sr_scale,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let n = buffer[0].len();

		// tape bandwidth
		let cutoff = (self.speed_target * 20_000.0).max(1000.0);
		for track in &mut self.tracks {
			track.pre_lp1.set_lowpass(cutoff, BUTTERWORTH_4_Q1);
			track.pre_lp2.set_lowpass(cutoff, BUTTERWORTH_4_Q2);
			track.post_lp1.set_lowpass(cutoff, BUTTERWORTH_4_Q1);
			track.post_lp2.set_lowpass(cutoff, BUTTERWORTH_4_Q2);
		}

		let [bl, br] = buffer;
		for i in 0..n {
			let drive = self.drive.process();

			let shared = Shared {
				h1_gain: self.h1_gain.process(),
				h2_gain: self.h2_gain.process(),
				h3_gain: self.h3_gain.process(),
				wow: self.wow.process(),
				balance: self.balance.process(),
				feedback: self.feedback.process(),
				drive,
				drive_inv: 1.0 / drive,
				base_speed: self.speed_filter.process(self.speed_target),
				flutter_term: self.step_flutter(),
				jitter: self.jitter,
				loop_len: self.loop_len,
				head_mod_depth: self.head_mod_depth,
			};

			self.tracks[0].process_sample(&mut bl[i], &shared);
			self.tracks[1].process_sample(&mut br[i], &shared);
		}
	}

	fn flush(&mut self) {
		self.balance.immediate();
		self.feedback.immediate();
		self.drive.immediate();
		self.h1_gain.immediate();
		self.h2_gain.immediate();
		self.h3_gain.immediate();
		self.wow.immediate();

		self.sweep = 0.0;
		self.next_rate = 0.5;
		self.flutter_rate.set_immediate(0.5);

		self.speed_filter.immediate();
		self.speed_filter.prime(self.speed_target);

		// Calculate steady state based on bias
		let drive = self.drive.target();
		let drive_inv = 1.0 / drive;
		let gain_sum = self.h1_gain.target() + self.h2_gain.target() + self.h3_gain.target();

		let (m_ss, m_irr_ss) = magnetic_fixed_point(BIAS * drive);
		let tape_value = m_ss * drive_inv;
		let post_tap = tape_value * gain_sum;

		// follow process logic
		let cutoff = (self.speed_target * 20_000.0).max(1000.0);

		for track in &mut self.tracks {
			track.pre_lp1.set_lowpass(cutoff, BUTTERWORTH_4_Q1);
			track.pre_lp1.immediate();
			track.pre_lp1.prime(BIAS);
			track.pre_lp2.set_lowpass(cutoff, BUTTERWORTH_4_Q2);
			track.pre_lp2.immediate();
			track.pre_lp2.prime(BIAS);

			track.post_lp1.set_lowpass(cutoff, BUTTERWORTH_4_Q1);
			track.post_lp1.immediate();
			track.post_lp1.prime(post_tap);
			track.post_lp2.set_lowpass(cutoff, BUTTERWORTH_4_Q2);
			track.post_lp2.immediate();
			track.post_lp2.prime(post_tap);

			track.noise_filter.reset_state();
			track.noise_filter.immediate();
			track.fb_filter.reset_state();
			track.fb_filter.immediate();
			track.fb_filter2.reset_state();
			track.fb_filter2.immediate();

			track.high.immediate();
			track.high.prime(post_tap);

			track.upsampler.set_dc(BIAS);
			track.downsampler.set_dc(post_tap);

			track.buffer.fill(tape_value);
			track.x1 = tape_value;
			track.x2 = tape_value;
			track.x3 = tape_value;
			track.m = m_ss;
			track.m_irr = m_irr_ss;
			track.prev_t = BIAS * drive;
			track.accum = 0.0;
			track.lfo_phase = 0.0;
			track.prev = 0.0;
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.balance.set(value),
			1 => self.speed_target = value,
			2 => self.feedback.set(value),
			3 => self.drive.set(from_db(value)),
			4 => self.wow.set(value * value),
			5 => self.flutter_depth = value * value,
			6 => self.jitter = value * value,
			7 => {
				let (h1, h2, h3) = match value as i32 {
					1 => (1.0, 0.0, 0.0),
					2 => (0.0, 1.0, 0.0),
					4 => (GAIN_PAIR, 0.0, GAIN_PAIR),
					5 => (0.0, GAIN_PAIR, GAIN_PAIR),
					_ => (0.0, 0.0, 1.0),
				};
				self.h1_gain.set(h1);
				self.h2_gain.set(h2);
				self.h3_gain.set(h3);
			},
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
