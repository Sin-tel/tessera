use fastrand::Rng;
use std::iter::zip;

use crate::dsp::env::*;
use crate::dsp::simper::Filter;
use crate::dsp::*;
use crate::instrument::*;

// TODO: phase reset?
// TODO: ADE env
// TODO: pitch env

#[derive(Debug, Default)]
pub struct Fm {
	sample_rate: f32,
	accum: f32,
	accum2: f32,
	prev: f32,
	freq: Smoothed,
	freq2: Smoothed,
	vel: SmoothedEnv,
	pres: SmoothedEnv,
	dc_killer: DcKiller,
	noise_filter: Filter,
	rng: Rng,

	feedback: f32,
	depth: f32,
	ratio: f32,
	ratio_fine: f32,
	offset: f32,
	noise_mod: f32,
}

impl Instrument for Fm {
	fn new(sample_rate: f32) -> Self {
		let mut noise_filter = Filter::new(sample_rate);
		noise_filter.set_highpass(5.0, 0.7);
		Fm {
			freq: Smoothed::new(10.0, sample_rate),
			freq2: Smoothed::new(10.0, sample_rate),
			vel: SmoothedEnv::new(1.0, 200.0, sample_rate),
			pres: SmoothedEnv::new(20.0, 50.0, sample_rate),
			sample_rate,
			noise_filter,
			..Default::default()
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			self.vel.update();
			let vel = self.vel.get();
			self.pres.update();
			let pres = self.pres.get();

			self.freq.update();
			self.freq2.update();
			let f = self.freq.get();
			let noise = self.noise_filter.process(self.rng.f32() - 0.5);
			self.accum += f + self.noise_mod * noise;
			if self.accum >= 1.0 {
				self.accum -= 1.0;
			}
			self.accum2 += self.freq2.get();
			if self.accum2 >= 1.0 {
				self.accum2 -= 1.0;
			}

			let mut prev = self.prev;
			if self.feedback < 0.0 {
				prev *= prev;
			}
			let feedback = self.feedback.abs();
			let op2 = sin_cheap(self.accum2 + feedback * prev * vel);

			self.prev = op2;
			// self.prev = lerp(self.prev, op2, 0.5);

			// depth and feedback reduction to mitigate aliasing
			// this stuff is all empirical
			let z = 40.0 * (self.ratio + 20.0 * feedback) * f;
			let max_d = 1.0 / (z * z);
			let depth = self.depth.min(max_d);

			let mut out = sin_cheap(self.accum + depth * op2 * (pres + 1.0));
			// self.prev = out;
			out *= vel;

			out = self.dc_killer.process(out);
			*l = out;
			*r = out;
		}
	}

	fn cv(&mut self, pitch: f32, pres: f32) {
		let f = pitch_to_hz(pitch) / self.sample_rate;
		self.freq.set(f);
		self.set_modulator();
		self.pres.set(pres);
	}

	fn note(&mut self, pitch: f32, vel: f32, _id: usize) {
		if vel == 0.0 {
			self.vel.set(0.0);
		} else {
			let f = pitch_to_hz(pitch) / self.sample_rate;
			self.freq.set_hard(f);
			self.set_modulator();
			self.freq2.instant();
			self.vel.set(vel);

			self.pres.set_hard(1.0);
			self.pres.set(0.0);

			if self.vel.get() < 0.01 {
				self.accum = 0.0;
				self.accum2 = 0.0;
				// self.vel.set_hard(vel);
			}
		}
	}

	fn set_param(&mut self, index: usize, value: f32) {
		match index {
			0 => self.feedback = value * 0.5,
			1 => self.depth = value,
			2 => {
				self.ratio = value;
				self.set_modulator();
			}
			3 => {
				self.ratio_fine = value;
				self.set_modulator();
			}
			4 => {
				self.offset = value / self.sample_rate;
				self.set_modulator();
			}
			5 => self.noise_mod = value * 0.01,

			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}

impl Fm {
	fn set_modulator(&mut self) {
		self.freq2
			.set((self.ratio + self.ratio_fine) * self.freq.inner() + self.offset);
	}
}

// branchless approximation of sin(2*pi*x)
fn sin_cheap(x: f32) -> f32 {
	let x = x - x.floor();
	let a = f32::from(x > 0.5);
	let b = 2.0 * x - 1.0 - 2.0 * a;
	(2.0 * a - 1.0) * (x * b + a) / (0.25 * x * b + 0.15625 + 0.25 * a)
	// (TWO_PI * x).sin()
}
