use std::iter::zip;

use crate::dsp::env::*;
use crate::dsp::*;
use crate::instrument::*;

#[derive(Debug, Default)]
pub struct Fm {
	accum: f32,
	accum2: f32,
	freq: Smoothed,
	freq2: Smoothed,
	vel: SmoothedEnv,
	sample_rate: f32,
	prev: f32,
	feedback: f32,
	depth: f32,
	ratio: f32,
	offset: f32,
	dc_killer: DcKiller,
}

impl Instrument for Fm {
	fn new(sample_rate: f32) -> Self {
		Fm {
			freq: Smoothed::new(10.0, sample_rate),
			freq2: Smoothed::new(10.0, sample_rate),
			vel: SmoothedEnv::new(1.0, 25.0, sample_rate),
			sample_rate,
			..Default::default()
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		// let f = self.freq.inner();
		// println!("{:?}", f + 50.0 * (self.ratio * f * self.depth));
		// // println!("{:}, {:}", f, (self.ratio * f * self.depth));

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			self.vel.update();
			self.freq.update();
			self.freq2.update();
			let f = self.freq.get();
			self.accum += f;
			if self.accum > 1.0 {
				self.accum -= 1.0;
			}
			self.accum2 += self.freq2.get();
			if self.accum2 > 1.0 {
				self.accum2 -= 1.0;
			}

			let mut prev = self.prev;
			if self.feedback < 0.0 {
				prev *= prev;
			}
			let op2 = sin_cheap(self.accum2);

			// depth and feedback reduction to mitigate aliasing
			// this stuff is all empirical
			let max_f = 1.0 / (4.0 + 300.0 * f);
			let feedback = self.feedback.abs().min(max_f);

			let z = 40.0 * (self.ratio + 20.0 * feedback) * f;
			let max_d = 1.0 / (z * z);
			let depth = self.depth.min(max_d);

			let vel = self.vel.get();

			let mut out = sin_cheap(self.accum + feedback * prev * vel + depth * op2 * vel);
			self.prev = out;
			out *= vel;

			out = self.dc_killer.process(out);

			*l = out;
			*r = out;
		}
	}

	fn cv(&mut self, pitch: f32, vel: f32) {
		let f = pitch_to_hz(pitch) / self.sample_rate;
		self.freq.set(f);
		self.set_modulator();
		self.vel.set(vel);
	}

	fn note(&mut self, pitch: f32, vel: f32, _id: usize) {
		let f = pitch_to_hz(pitch) / self.sample_rate;
		self.freq.set_hard(f);
		self.set_modulator();
		self.freq2.instant();

		// self.vel.set_hard(vel);
		// self.accum = 0.0;

		// if self.vel.get() < 0.01 {
		// 	self.vel.set_hard(vel);
		// 	self.accum = 0.0;
		// } else {
		self.vel.set(vel);
		// }
	}

	fn set_param(&mut self, index: usize, value: f32) {
		match index {
			0 => self.feedback = value / TWO_PI,
			1 => self.depth = value / TWO_PI,
			2 => {
				self.ratio = value;
				self.set_modulator();
			}
			3 => {
				self.offset = value / self.sample_rate;
				self.set_modulator();
			}
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}

impl Fm {
	fn set_modulator(&mut self) {
		self.freq2.set(self.ratio * self.freq.inner() + self.offset);
	}
}

// branchless approximation of sin(2*pi*x)
// only valid in [0, 1]
fn sin_cheap(x: f32) -> f32 {
	let x = x - x.floor();
	let a = (x > 0.5) as usize as f32;
	let b = 2.0 * x - 1.0 - 2.0 * a;
	(2.0 * a - 1.0) * (x * b + a) / (0.25 * x * b + 0.15625 + 0.25 * a)
	// (TWO_PI * x).sin()
}
