use crate::audio::MAX_BUF_SIZE;
use crate::dsp::onepole::{OnePoleCoefs, OnePoleState};
use crate::dsp::smooth::*;
use crate::dsp::*;
use crate::instrument::*;

const N_VOICES: usize = 32;
const N_BARS: usize = 9;

// drawbar footages 16' 5⅓' 8' 4' 2⅔' 2' 1⅗' 1⅓' 1', relative to 8'
const RATIOS: [f32; N_BARS] = [0.5, 1.5, 1., 2., 3., 4., 5., 6., 8.];

// uniform spread in cents
const DETUNE: f32 = 2.0;

const GAIN: f32 = 0.05;

// dB per drawbar step, 1 to 8
const DRAWBAR_DB: [f32; 8] = [-32., -26., -20., -15., -10., -6., -3., 0.];

// 3rd harmonic, fast, soft
const PERC_BAR: usize = 4;
const PERC_LEVEL: f32 = 1.2;
// ms decay time
const PERC_DECAY: f32 = 400.0;

// key contact timings, per partial (ms)
// closing starts at a random time in the window, with a noisy bounce
const CLICK_WINDOW: f32 = 3.2;
const CLICK_BURST_MIN: f32 = 0.36;
const CLICK_BURST_MAX: f32 = 1.2;
// linear
const RELEASE: f32 = 2.7;
// click setting 0..1 maps depth 0..1 and contact smoothing cutoff
const CONTACT_SMOOTH_MIN: f32 = 1200.;
const CONTACT_SMOOTH_MAX: f32 = 4000.;

#[derive(Debug)]
pub struct Organ {
	voices: Vec<Voice>,
	sample_rate: f32,
	drawbars: [Smooth; N_BARS],
	bar_buf: [[f32; N_BARS]; MAX_BUF_SIZE],
	percussion: bool,
	// shared by all voices, only retriggers when all keys are up
	perc_env: f32,
	perc_decay: f32,
	click_window: u32,
	click_burst: (u32, u32),
	release_step: f32,
	contact_smooth: OnePoleCoefs,
	click_depth: f32,
	// global clock so phases are free-running, like tonewheels
	time: u64,
}

#[derive(Debug)]
struct Voice {
	accum: [f32; N_BARS],
	freq: [f32; N_BARS],
	contacts: [Contact; N_BARS],
	note_on: bool,
	active: bool,
}

impl Voice {
	fn new() -> Self {
		Self {
			accum: [0.; N_BARS],
			freq: [0.; N_BARS],
			contacts: [Contact::default(); N_BARS],
			note_on: false,
			active: false,
		}
	}
}

#[derive(Debug, Default, Clone, Copy)]
struct Contact {
	closed: bool,
	delay: u32,
	burst: u32,
	raw: f32,
	filter: OnePoleState,
	value: f32,
}

impl Contact {
	fn close(&mut self, window: u32, burst: (u32, u32)) {
		self.closed = true;
		self.burst = fastrand::u32(burst.0..=burst.1);
		self.delay = fastrand::u32(0..=window.saturating_sub(self.burst));
	}

	fn open(&mut self) {
		self.closed = false;
	}

	fn process(&mut self, release_step: f32, smooth: &OnePoleCoefs, depth: f32) -> f32 {
		if self.closed {
			if self.delay > 0 {
				self.delay -= 1;
			} else if self.burst > 0 {
				self.burst -= 1;
				self.raw = 1. - depth * fastrand::f32();
			} else {
				self.raw = 1.;
			}
		} else {
			self.raw = (self.raw - release_step).max(0.);
		}
		self.value = self.filter.process(smooth, self.raw);
		self.value
	}

	fn is_silent(&self) -> bool {
		!self.closed && self.raw == 0. && self.value < 1e-4
	}

	fn reset(&mut self) {
		*self = Self::default();
	}
}

fn drawbar_gain(value: f32) -> f32 {
	let i = value.round() as usize;
	if i == 0 { 0. } else { from_db(DRAWBAR_DB[i.min(8) - 1]) }
}

// uniform in [-1, 1)
fn hash_uniform(x: i64) -> f32 {
	// splitmix64 finalizer
	let mut z = (x as u64).wrapping_add(0x9e37_79b9_7f4a_7c15);
	z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
	z ^= z >> 31;
	(z >> 40) as f32 / (1u64 << 23) as f32 - 1.
}

impl Organ {
	fn set_click(&mut self, value: f32) {
		self.click_depth = value;
		let hz = CONTACT_SMOOTH_MIN + value * (CONTACT_SMOOTH_MAX - CONTACT_SMOOTH_MIN);
		self.contact_smooth = OnePoleCoefs::lowpass(hz, self.sample_rate);
	}
}

impl Instrument for Organ {
	fn new(sample_rate: f32) -> Self {
		let mut voices = Vec::with_capacity(N_VOICES);
		for _ in 0..N_VOICES {
			voices.push(Voice::new());
		}

		let perc_decay = 1.0 - time_constant(PERC_DECAY, sample_rate);
		let ms = |t: f32| (t * 0.001 * sample_rate).round() as u32;
		let mut organ = Organ {
			voices,
			sample_rate,
			drawbars: std::array::from_fn(|_| Smooth::new(0., 10.0, sample_rate)),
			bar_buf: [[0.; N_BARS]; MAX_BUF_SIZE],
			percussion: false,
			perc_env: 0.,
			perc_decay,
			click_window: ms(CLICK_WINDOW),
			click_burst: (ms(CLICK_BURST_MIN).max(1), ms(CLICK_BURST_MAX).max(1)),
			release_step: time_constant_linear(RELEASE, sample_rate),
			contact_smooth: OnePoleCoefs::default(),
			click_depth: 0.,
			time: 0,
		};
		organ.set_click(0.5);
		organ
	}

	fn voice_count(&self) -> usize {
		N_VOICES
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;
		let len = bl.len();
		assert!(len <= MAX_BUF_SIZE);

		for bars in &mut self.bar_buf[..len] {
			for (b, d) in bars.iter_mut().zip(&mut self.drawbars) {
				*b = d.process();
			}
			if self.percussion {
				bars[PERC_BAR] += self.perc_env;
				self.perc_env *= self.perc_decay;
			}
		}

		for voice in self.voices.iter_mut().filter(|v| v.active) {
			for (sample, bars) in bl.iter_mut().zip(&self.bar_buf) {
				let mut out = 0.;
				for k in 0..N_BARS {
					let a = &mut voice.accum[k];
					*a += voice.freq[k];
					*a -= fast_floor(*a);
					let env = voice.contacts[k].process(
						self.release_step,
						&self.contact_smooth,
						self.click_depth,
					);
					out += bars[k] * env * sin_cheap(*a);
				}

				*sample += out * GAIN;
			}
			if !voice.note_on && voice.contacts.iter().all(Contact::is_silent) {
				voice.active = false;
			}
		}

		br.copy_from_slice(bl);
		self.time += len as u64;
	}

	fn pitch(&mut self, _pitch: f32, _id: usize) {}

	fn pressure(&mut self, _pressure: f32, _id: usize) {}

	fn note_on(&mut self, pitch: f32, _vel: f32, id: usize) {
		if self.percussion && !self.voices.iter().any(|v| v.note_on) {
			self.perc_env = PERC_LEVEL;
		}

		let voice = &mut self.voices[id];
		let f0 = pitch_to_hz(pitch) / self.sample_rate;

		for k in 0..N_BARS {
			let r = RATIOS[k];
			// the partial's own pitch rounded to 10 cents, so coinciding partials of different keys match
			let key = (10. * (pitch + 12. * r.log2())).round() as i64;
			let detune = (DETUNE / 1200. * hash_uniform(key)).exp2();

			let f = f0 * r * detune;
			if f < 0.45 {
				voice.freq[k] = f;
				voice.accum[k] = (f64::from(f) * self.time as f64).fract() as f32;
			} else {
				voice.freq[k] = 0.;
				voice.accum[k] = 0.;
			}
		}
		for c in &mut voice.contacts {
			c.close(self.click_window, self.click_burst);
		}
		voice.note_on = true;
		voice.active = true;
	}

	fn note_off(&mut self, id: usize) {
		let voice = &mut self.voices[id];
		for c in &mut voice.contacts {
			c.open();
		}
		voice.note_on = false;
	}

	fn flush(&mut self) {
		self.time = 0;
		self.perc_env = 0.;
		for d in &mut self.drawbars {
			d.immediate();
		}
		for v in &mut self.voices {
			v.contacts.iter_mut().for_each(Contact::reset);
			v.accum = [0.; N_BARS];
			v.note_on = false;
			v.active = false;
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0..N_BARS => self.drawbars[index].set(drawbar_gain(value)),
			9 => self.percussion = value > 0.5,
			10 => self.set_click(value),
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
