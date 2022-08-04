include!("wavetable_res.rs");

use crate::dsp::env::*;
use crate::dsp::*;
use crate::instrument::*;
use crate::param::Param;
use std::iter::zip;

#[derive(Debug, Default)]
pub struct Wavetable {
	accum: f32,
	freq: Smoothed,
	vel: SmoothedEnv,
	sample_rate: f32,
}

impl Instrument for Wavetable {
	fn new(sample_rate: f32) -> Self {
		Wavetable {
			freq: Smoothed::new(20.0, sample_rate),
			vel: SmoothedEnv::new(20.0, 50.0, sample_rate),
			sample_rate,
			..Default::default()
		}
	}

	fn cv(&mut self, pitch: f32, vel: f32) {
		let p = pitch_to_f(pitch, self.sample_rate);
		self.freq.set(p);
		self.vel.set(vel);
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			self.vel.update();
			self.freq.update();
			self.accum += self.freq.get();
			self.accum = self.accum.fract();

			let idx = self.accum * WT_SIZE;
			let idx_int = idx as usize;
			let idx_frac = idx.fract();

			let w1 = WAVETABLE[idx_int];
			let w2 = WAVETABLE[(idx_int + 1) & WT_MASK];
			let mut out = lerp(w1, w2, idx_frac);
			out *= self.vel.get();

			*l = out;
			*r = out;
		}
	}

	fn note(&mut self, pitch: f32, vel: f32, _id: usize) {
		let p = pitch_to_f(pitch, self.sample_rate);
		self.freq.set_hard(p);

		if self.vel.get() < 0.01 {
			self.vel.set_hard(vel);
			self.accum = 0.0;
		} else {
			self.vel.set(vel);
		}
	}
}

impl Param for Wavetable {
	fn set_param(&mut self, index: usize, _value: f32) {
		match index {
			_ => eprintln!("Parameter with index {} not found", index),
		}
	}
}
