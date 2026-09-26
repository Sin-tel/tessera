use crate::audio::MAX_BUF_SIZE;
use crate::dsp::env::AttackRelease;
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

#[derive(Debug)]
pub struct Organ {
	voices: Vec<Voice>,
	sample_rate: f32,
	drawbars: [Smooth; N_BARS],
	bar_buf: [[f32; N_BARS]; MAX_BUF_SIZE],
	// global clock so phases are free-running, like tonewheels
	time: u64,
}

#[derive(Debug)]
struct Voice {
	accum: [f32; N_BARS],
	freq: [f32; N_BARS],
	env: AttackRelease,
	note_on: bool,
	active: bool,
}

impl Voice {
	fn new(sample_rate: f32) -> Self {
		Self {
			accum: [0.; N_BARS],
			freq: [0.; N_BARS],
			env: AttackRelease::new(1.0, 3.0, sample_rate),
			note_on: false,
			active: false,
		}
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

impl Instrument for Organ {
	fn new(sample_rate: f32) -> Self {
		let mut voices = Vec::with_capacity(N_VOICES);
		for _ in 0..N_VOICES {
			voices.push(Voice::new(sample_rate));
		}

		Organ {
			voices,
			sample_rate,
			drawbars: std::array::from_fn(|_| Smooth::new(0., 10.0, sample_rate)),
			bar_buf: [[0.; N_BARS]; MAX_BUF_SIZE],
			time: 0,
		}
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
		}

		for voice in self.voices.iter_mut().filter(|v| v.active) {
			for (sample, bars) in bl.iter_mut().zip(&self.bar_buf) {
				let env = voice.env.process();

				let mut out = 0.;
				for k in 0..N_BARS {
					let a = &mut voice.accum[k];
					*a += voice.freq[k];
					*a -= fast_floor(*a);
					out += bars[k] * sin_cheap(*a);
				}

				*sample += out * env * GAIN;
			}
			if !voice.note_on && voice.env.get() < 1e-4 {
				voice.active = false;
			}
		}

		br.copy_from_slice(bl);
		self.time += len as u64;
	}

	fn pitch(&mut self, _pitch: f32, _id: usize) {}

	fn pressure(&mut self, _pressure: f32, _id: usize) {}

	fn note_on(&mut self, pitch: f32, _vel: f32, id: usize) {
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
		voice.note_on = true;
		voice.active = true;
		voice.env.set(1.0);
	}

	fn note_off(&mut self, id: usize) {
		let voice = &mut self.voices[id];
		voice.env.set(0.0);
		voice.note_on = false;
	}

	fn flush(&mut self) {
		self.time = 0;
		for d in &mut self.drawbars {
			d.immediate();
		}
		for v in &mut self.voices {
			v.env.set_immediate(0.);
			v.accum = [0.; N_BARS];
			v.note_on = false;
			v.active = false;
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0..N_BARS => self.drawbars[index].set(drawbar_gain(value)),
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
