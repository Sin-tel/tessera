use std::iter::zip;

use crate::dsp::smooth::*;
use crate::dsp::*;
use crate::instrument::*;

#[derive(Debug)]
pub struct Sine {
	accum: f32,
	freq: SmoothExp,
	vel: SmoothExp,
	sample_rate: f32,
	fixed: bool,
	fixed_freq: f32,
	fixed_gain: f32,
}

impl Instrument for Sine {
	fn new(sample_rate: f32) -> Self {
		let mut vel = SmoothExp::new(10.0, sample_rate);
		vel.set_immediate(0.);
		Sine {
			freq: SmoothExp::new(2.0, sample_rate),
			vel,
			sample_rate,
			fixed: false,
			accum: 0.,
			fixed_freq: 0.01,
			fixed_gain: 1.,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		if self.fixed {
			self.freq.set(self.fixed_freq);
			self.vel.set(self.fixed_gain);
		}
		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			let vel = self.vel.process();
			let f = self.freq.process();

			self.accum += f;
			self.accum = self.accum - self.accum.floor();

			let mut out = (self.accum * TWO_PI).sin();
			out *= vel;

			*l = out;
			*r = out;
		}
	}

	fn cv(&mut self, pitch: f32, _: f32, _id: usize) {
		if !self.fixed {
			let p = pitch_to_hz(pitch) / self.sample_rate;
			self.freq.set(p);
		}
	}

	fn note(&mut self, pitch: f32, vel: f32, _id: usize) {
		if !self.fixed {
			if vel == 0.0 {
				self.vel.set(0.0);
			} else {
				let p = pitch_to_hz(pitch) / self.sample_rate;
				self.freq.set(p);
				self.vel.set(vel);
			}
		}
	}
	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => {
				self.fixed = value > 0.5;
				if !self.fixed {
					self.vel.set(0.0);
				}
			}
			1 => {
				self.fixed_freq = value / self.sample_rate;
			}
			2 => {
				self.fixed_gain = value;
			}
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}
