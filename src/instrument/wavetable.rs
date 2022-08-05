include!("wavetable_res.rs");

use crate::device::Param;
use crate::dsp::env::*;
use crate::dsp::*;
use crate::instrument::*;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
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
	interpolate: f32,
	buffer_in: Vec<f32>,
	buffer_a: Vec<f32>,
	buffer_b: Vec<f32>,
	spectrum: Vec<Complex<f32>>,
	spectrum2: Vec<Complex<f32>>,
	r2c_sratch: Vec<Complex<f32>>,
	r2c: Arc<dyn RealToComplex<f32>>,
	c2r_sratch: Vec<Complex<f32>>,
	c2r: Arc<dyn ComplexToReal<f32>>,
}

impl Wavetable {
	fn update_fft(&mut self) {
		// @todo precalculate forward fft and only run inverse with corrected partials

		// linear interpolation between frames
		let wt_idx = (self.vel.inner() * (WT_NUM as f32)).clamp(0.0, (WT_NUM as f32) - 1.001);
		let wt_idx_int = wt_idx as usize;
		let wt_idx_frac = wt_idx.fract();

		for (i, v) in self.buffer_in.iter_mut().enumerate() {
			let w1 = WAVETABLE[wt_idx_int * WT_SIZE + i];
			let w2 = WAVETABLE[(wt_idx_int + 1) * WT_SIZE + i];
			*v = lerp(w1, w2, wt_idx_frac);
		}

		// forward fft
		self.r2c
			.process_with_scratch(
				&mut self.buffer_in,
				&mut self.spectrum,
				&mut self.r2c_sratch,
			)
			.unwrap(); // only panics when passed incorrect buffer sizes

		// calculate maximum allowed partial
		let p_max = (MAX_F / (self.sample_rate * self.freq.get())) as usize;

		// zero out everything above p_max
		let it = self.spectrum.iter().zip(self.spectrum2.iter_mut());
		for (i, (x, y)) in it.enumerate() {
			if i <= p_max {
				*y = *x;
			} else {
				*y = Zero::zero();
			}
		}

		// inverse fft
		self.c2r
			.process_with_scratch(
				&mut self.spectrum2,
				&mut self.buffer_a,
				&mut self.c2r_sratch,
			)
			.unwrap(); // only panics when passed incorrect buffer sizes

		// normalize
		for v in self.buffer_a.iter_mut() {
			*v /= WT_SIZE as f32;
		}

		std::mem::swap(&mut self.buffer_a, &mut self.buffer_b);
	}
}

impl Instrument for Wavetable {
	fn new(sample_rate: f32) -> Self {
		let mut real_planner = RealFftPlanner::<f32>::new();
		let r2c = real_planner.plan_fft_forward(WT_SIZE);
		let c2r = real_planner.plan_fft_inverse(WT_SIZE);

		let buffer_in = r2c.make_input_vec();
		let spectrum = r2c.make_output_vec();
		let spectrum2 = r2c.make_output_vec();
		let r2c_sratch = r2c.make_scratch_vec();
		let c2r_sratch = c2r.make_scratch_vec();
		let buffer_a = c2r.make_output_vec();
		let buffer_b = c2r.make_output_vec();

		let mut new = Wavetable {
			accum: 0.0,
			interpolate: 1.0,
			freq: Smoothed::new(20.0, sample_rate),
			vel: SmoothedEnv::new(20.0, 40.0, sample_rate),
			sample_rate,
			buffer_in,
			buffer_a,
			buffer_b,
			spectrum,
			spectrum2,
			r2c_sratch,
			c2r_sratch,
			r2c,
			c2r,
		};
		new.update_fft();
		new
	}

	fn cv(&mut self, pitch: f32, vel: f32) {
		let p = pitch_to_f(pitch, self.sample_rate);
		self.freq.set(p);
		self.vel.set(vel);
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		if self.interpolate >= 1.0 {
			self.interpolate = 0.0;
			self.update_fft();
		}

		let [bl, br] = buffer;

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			self.interpolate += 1.0 / (0.05 * self.sample_rate); // update every 50ms

			self.vel.update();
			self.freq.update();
			self.accum += self.freq.get();
			self.accum = self.accum.fract();

			let idx = self.accum * (WT_SIZE as f32);
			let idx_int = idx as usize;
			let idx_frac = idx.fract();

			// bilinear interpolation between samples and buffers
			let w1a = self.buffer_a[idx_int];
			let w2a = self.buffer_a[(idx_int + 1) & WT_MASK];
			let wa = lerp(w1a, w2a, idx_frac);

			let w1b = self.buffer_b[idx_int];
			let w2b = self.buffer_b[(idx_int + 1) & WT_MASK];
			let wb = lerp(w1b, w2b, idx_frac);

			let mut out = lerp(wa, wb, self.interpolate.clamp(0.0, 1.0));

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
			self.interpolate = 1.0;
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
