include!("wavetable_res.rs");

use crate::device::Param;
use crate::dsp::env::*;
use crate::dsp::*;
use crate::instrument::*;
use realfft::{ComplexToReal, RealFftPlanner};
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use std::sync::Arc;

use std::iter::zip;

const MAX_F: f32 = 20_000.0;

pub struct Wavetable {
	accum: f32,
	freq: Smoothed,
	vel: SmoothedEnv,
	sample_rate: f32,
	buffer_out: Vec<f32>,
	spectrum: Vec<Complex<f32>>,
	spectrum2: Vec<Complex<f32>>,
	c2r_sratch: Vec<Complex<f32>>,
	c2r: Arc<dyn ComplexToReal<f32>>,
	timer: f32,
}

impl Wavetable {
	fn update_fft(&mut self) {
		// calculate maximum allowed partial
		let p_max = (MAX_F / (self.sample_rate * self.freq.get())) as usize;

		// dbg!(p_max);

		let it = self.spectrum.iter().zip(self.spectrum2.iter_mut());
		for (i, (x, y)) in it.enumerate() {
			if i <= p_max {
				*y = *x;
			} else {
				*y = Zero::zero();
			}
		}

		self.c2r
			.process_with_scratch(
				&mut self.spectrum2,
				&mut self.buffer_out,
				&mut self.c2r_sratch,
			)
			.unwrap();

		// normalize
		for v in self.buffer_out.iter_mut() {
			*v /= WT_SIZE as f32;
		}
	}
}

impl Instrument for Wavetable {
	fn new(sample_rate: f32) -> Self {
		let mut real_planner = RealFftPlanner::<f32>::new();
		let r2c = real_planner.plan_fft_forward(WT_SIZE);
		let c2r = real_planner.plan_fft_inverse(WT_SIZE);

		let mut spectrum = r2c.make_output_vec();
		let spectrum2 = r2c.make_output_vec();
		let mut r2c_sratch = r2c.make_scratch_vec();
		let c2r_sratch = c2r.make_scratch_vec();
		let buffer_out = c2r.make_output_vec();
		let mut buffer_in: Vec<f32> = WAVETABLE.to_vec();

		r2c.process_with_scratch(&mut buffer_in, &mut spectrum, &mut r2c_sratch)
			.unwrap();

		// dbg!(&spectrum);

		Wavetable {
			accum: 0.0,
			freq: Smoothed::new(20.0, sample_rate),
			vel: SmoothedEnv::new(20.0, 50.0, sample_rate),
			sample_rate,
			buffer_out,
			spectrum,
			spectrum2,
			c2r_sratch,
			c2r,
			timer: 0.0,
		}
	}

	fn cv(&mut self, pitch: f32, vel: f32) {
		let p = pitch_to_f(pitch, self.sample_rate);
		self.freq.set(p);
		self.vel.set(vel);
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		self.timer -= (buffer[0].len() as f32) / self.sample_rate;
		if self.timer <= 0.0 {
			self.timer = 0.05; // update every 50ms
			self.update_fft();
		}

		let [bl, br] = buffer;

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			self.vel.update();
			self.freq.update();
			self.accum += self.freq.get();
			self.accum = self.accum.fract();

			let idx = self.accum * (WT_SIZE as f32);
			let idx_int = idx as usize;
			let idx_frac = idx.fract();

			// hermite interpolation
			// let w0 = self.buffer_out[(idx_int - 1) & WT_MASK];
			// let w1 = self.buffer_out[idx_int];
			// let w2 = self.buffer_out[(idx_int + 1) & WT_MASK];
			// let w3 = self.buffer_out[(idx_int + 2) & WT_MASK];

			// let slope0 = (w2 - w0) * 0.5;
			// let slope1 = (w3 - w1) * 0.5;
			// let v = w1 - w2;
			// let w = slope0 + v;
			// let a = w + v + slope1;
			// let b_neg = w + a;
			// let s1 = a * idx_frac - b_neg;
			// let s2 = s1 * idx_frac + slope0;
			// let mut out = s2 * idx_frac + w1;

			// linear interpolation
			let w1 = self.buffer_out[idx_int];
			let w2 = self.buffer_out[(idx_int + 1) & WT_MASK];
			let mut out = lerp(w1, w2, idx_frac);

			// no interpolation
			// let mut out = w1;
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
