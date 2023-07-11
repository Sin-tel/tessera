use fastrand::Rng;
use std::iter::zip;

use crate::dsp::env::*;
use crate::dsp::smooth::*;
use crate::dsp::*;
use crate::instrument::*;

// TODO: phase reset?
// TODO: ADE env
// TODO: pitch env
// TODO: try ADAA here as well

#[derive(Debug)]
pub struct Fm {
	voices: Vec<Voice>,
	sample_rate: f32,
	dc_killer: DcKiller,

	rng: Rng,
	feedback: f32,
	depth: f32,
	ratio: f32,
	ratio_fine: f32,
	offset: f32,
	noise_mod: f32,
	noise_decay: f32,
}

const N_VOICES: usize = 16;

#[derive(Debug)]
struct Voice {
	active: bool,
	accum: f32,
	accum2: f32,
	prev: f32,
	freq: SmoothExp,
	freq2: SmoothExp,
	env: Adsr,
	vel: f32,
	pres: AttackRelease,
	noise_level: f32,
}

impl Voice {
	fn new(sample_rate: f32) -> Self {
		Self {
			freq: SmoothExp::new(10.0, sample_rate),
			freq2: SmoothExp::new(10.0, sample_rate),
			env: Adsr::new(sample_rate),
			pres: AttackRelease::new(20.0, 50.0, sample_rate),
			active: false,
			accum: 0.,
			accum2: 0.,
			prev: 0.,
			vel: 0.,
			noise_level: 0.,
		}
	}

	fn set_modulator(&mut self, ratio: f32, ratio_fine: f32, offset: f32) {
		self.freq2
			.set((ratio + ratio_fine) * self.freq.target() + offset);
	}
}

impl Instrument for Fm {
	fn new(sample_rate: f32) -> Self {
		let mut voices = Vec::with_capacity(N_VOICES);
		for _ in 0..N_VOICES {
			voices.push(Voice::new(sample_rate));
		}

		Fm {
			voices,
			dc_killer: DcKiller::new(sample_rate),
			sample_rate,

			rng: Rng::new(),
			feedback: 0.,
			depth: 0.,
			ratio: 0.,
			ratio_fine: 0.,
			offset: 0.,
			noise_mod: 0.,
			noise_decay: 0.,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for voice in self.voices.iter_mut().filter(|v| v.active) {
			for sample in bl.iter_mut() {
				let env = voice.env.process();
				let pres = voice.pres.process();
				let f = voice.freq.process();
				let f2 = voice.freq2.process();

				voice.noise_level *= self.noise_decay;

				let noise = voice.noise_level * (self.rng.f32() - 0.5);
				voice.accum += f + noise;
				voice.accum -= voice.accum.floor();

				voice.accum2 += f2;
				voice.accum2 -= voice.accum2.floor();

				let mut prev = voice.prev;
				if self.feedback < 0.0 {
					prev *= prev;
				}
				let feedback = self.feedback.abs();
				let op2 = sin_cheap(voice.accum2 + feedback * prev) /* * mod_env */ ;

				voice.prev = lerp(voice.prev, op2, 0.3);

				// depth and feedback reduction to mitigate aliasing
				// this stuff is all empirical
				let z = 40.0 * (self.ratio + 20.0 * feedback) * f;
				let max_d = 1.0 / (z * z);
				let depth = (self.depth * voice.vel).min(max_d);

				let mut out = sin_cheap(voice.accum + depth * op2 * (pres + 1.0));
				out *= env;

				*sample += 0.5 * out;
			}
			if voice.env.get() < 1e-4 {
				voice.active = false;
			}
		}

		for s in bl.iter_mut() {
			*s = self.dc_killer.process(*s);
		}

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			*r = *l;
		}
	}

	fn cv(&mut self, pitch: f32, _: f32, id: usize) {
		let voice = &mut self.voices[id];
		let p = pitch_to_hz(pitch) / self.sample_rate;
		voice.freq.set(p);
		voice.set_modulator(self.ratio, self.ratio_fine, self.offset);
	}

	fn note(&mut self, pitch: f32, vel: f32, id: usize) {
		let voice = &mut self.voices[id];
		if vel == 0.0 {
			voice.env.note_off();
		} else {
			let f = pitch_to_hz(pitch) / self.sample_rate;
			voice.freq.set_immediate(f);
			voice.set_modulator(self.ratio, self.ratio_fine, self.offset);
			voice.freq2.immediate();
			voice.env.note_on(vel);
			voice.vel = vel;

			// phase reset
			if !voice.active {
				voice.accum = 0.0;
				voice.accum2 = 0.0;
			}
			voice.active = true;

			voice.noise_level = self.noise_mod;
		}
	}
	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.feedback = value * 0.5,
			1 => self.depth = value,
			2 => {
				self.ratio = value;
				self.voices
					.iter_mut()
					.for_each(|v| v.set_modulator(self.ratio, self.ratio_fine, self.offset));
			}
			3 => {
				self.ratio_fine = value;
				self.voices
					.iter_mut()
					.for_each(|v| v.set_modulator(self.ratio, self.ratio_fine, self.offset));
			}
			4 => {
				self.offset = value / self.sample_rate;
				self.voices
					.iter_mut()
					.for_each(|v| v.set_modulator(self.ratio, self.ratio_fine, self.offset));
			}
			5 => self.voices.iter_mut().for_each(|v| v.env.set_attack(value)),
			6 => self.voices.iter_mut().for_each(|v| v.env.set_decay(value)),
			7 => self
				.voices
				.iter_mut()
				.for_each(|v| v.env.set_sustain(value)),
			8 => self
				.voices
				.iter_mut()
				.for_each(|v| v.env.set_release(value)),
			9 => self.noise_mod = value * value * 0.05,
			10 => self.noise_decay = 1.0 - time_constant(value, self.sample_rate),

			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}
