use std::iter::zip;

use crate::dsp::env::*;
use crate::dsp::*;
use crate::instrument::*;

#[derive(Debug, Default)]
pub struct Sine {
	accum: f32,
	freq: Smoothed,
	vel: SmoothedEnv,
	sample_rate: f32,
	prev: f32,
	feedback: f32,
	dc_killer: DcKiller,
}

impl Instrument for Sine {
	fn new(sample_rate: f32) -> Self {
		Sine {
			freq: Smoothed::new(10.0, sample_rate),
			vel: SmoothedEnv::new(10.0, 25.0, sample_rate),
			sample_rate,
			..Default::default()
		}
	}

	fn cv(&mut self, pitch: f32, vel: f32) {
		let p = pitch_to_hz(pitch) / self.sample_rate;
		self.freq.set(p);
		self.vel.set(vel);
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			self.vel.update();
			self.freq.update();
			self.accum += self.freq.get();
			if self.accum > 1.0 {
				self.accum -= 1.0;
			}
			let mut out = (self.accum * TWO_PI + self.feedback * self.prev).sin();
			out *= self.vel.get();

			self.prev = out;
			out = self.dc_killer.process(out);

			*l = out;
			*r = out;
		}
	}

	fn note(&mut self, pitch: f32, vel: f32, _id: usize) {
		let p = pitch_to_hz(pitch) / self.sample_rate;
		self.freq.set_hard(p);

		// self.vel.set_hard(vel);
		// self.accum = 0.0;

		if self.vel.get() < 0.01 {
			self.vel.set_hard(vel);
			self.accum = 0.0;
		} else {
			self.vel.set(vel);
		}
	}
	fn set_param(&mut self, index: usize, value: f32) {
		match index {
			0 => self.feedback = value,
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}
