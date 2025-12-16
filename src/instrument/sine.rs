use fastrand::Rng;
use std::iter::zip;

use crate::dsp::smooth::*;
use crate::dsp::*;
use crate::instrument::*;

#[derive(Debug)]
pub struct Sine {
	sample_rate: f32,
	accum: f32,
	freq: Smooth,
	vel: Smooth,
	rng: Rng,
	fixed: bool,
	fixed_freq: f32,
	fixed_gain: f32,
	noise: bool,
}

impl Instrument for Sine {
	fn new(sample_rate: f32) -> Self {
		Sine {
			sample_rate,
			freq: Smooth::new(0., 50.0, sample_rate),
			vel: Smooth::new(0., 25., sample_rate),
			rng: Rng::new(),
			fixed: false,
			accum: 0.,
			fixed_freq: 0.01,
			fixed_gain: 1.,
			noise: false,
		}
	}

	fn voice_count(&self) -> usize {
		1
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		if self.noise {
			self.vel.set(self.fixed_gain);
			for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
				let vel = self.vel.process();
				*l = vel * (self.rng.f32() - 0.5);
				*r = vel * (self.rng.f32() - 0.5);
			}
			return;
		}
		if self.fixed {
			self.freq.set(self.fixed_freq);
			self.vel.set(self.fixed_gain);
		}
		for sample in bl.iter_mut() {
			let vel = self.vel.process();
			let f = self.freq.process();

			self.accum += f;
			self.accum = self.accum - self.accum.floor();

			let mut out = (self.accum * TWO_PI).sin();
			out *= vel;

			*sample = out;
		}

		br.copy_from_slice(bl);
	}

	fn pitch(&mut self, pitch: f32, _id: usize) {
		if !self.fixed {
			let p = pitch_to_hz(pitch) / self.sample_rate;
			self.freq.set(p);
		}
	}

	fn pressure(&mut self, _presure: f32, _id: usize) {}

	fn note_on(&mut self, pitch: f32, vel: f32, _id: usize) {
		if !self.fixed {
			let p = pitch_to_hz(pitch) / self.sample_rate;
			self.freq.set(p);
			self.vel.set(vel);
		}
	}

	fn note_off(&mut self, _id: usize) {
		if !self.fixed {
			self.vel.set(0.0);
		}
	}
	fn flush(&mut self) {}

	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => {
				self.fixed = value > 0.5;
				if !self.fixed {
					self.vel.set(0.0);
				}
			},
			1 => self.fixed_freq = value / self.sample_rate,
			2 => self.fixed_gain = value,
			3 => {
				self.noise = value > 0.5;
				if !self.noise && !self.fixed {
					self.vel.set_immediate(0.)
				}
			},
			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}
