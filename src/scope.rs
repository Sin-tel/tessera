use crate::defs::*;
use bit_mask_ring_buf::BMRingBuf;
use realfft::{RealFftPlanner, RealToComplex};
use std::sync::Arc;

pub struct Scope {
	buf: BMRingBuf<f32>,
	pos: isize,
	r2c: Arc<dyn RealToComplex<f32>>,
}

impl Scope {
	pub fn new() -> Self {
		let mut real_planner = RealFftPlanner::<f32>::new();
		let r2c = real_planner.plan_fft_forward(SPECTRUM_SIZE);

		Scope {
			buf: BMRingBuf::<f32>::from_len(SPECTRUM_SIZE),
			pos: 0,
			r2c,
		}
	}

	pub fn get_spectrum(&self) -> Vec<f64> {
		let (a, b) = self.buf.as_slices(self.pos);

		let mut in_buffer = [a, b].concat();
		let mut spectrum = self.r2c.make_output_vec();

		for (i, v) in in_buffer.iter_mut().enumerate() {
			*v *= 0.5 * (1.0 - ((TWO_PI * (i as f32)) / (SPECTRUM_SIZE as f32)).cos())
		}

		self.r2c.process(&mut in_buffer, &mut spectrum).unwrap();

		let scale = 1.0 / (SPECTRUM_SIZE as f32).sqrt();
		spectrum
			.iter()
			.map(|&e| (e.norm() * scale) as f64)
			.collect()
	}

	pub fn get_oscilloscope(&self) -> Vec<f64> {
		let (a, b) = self.buf.as_slices(self.pos);

		[a, b].concat().iter().map(|&e| e as f64).collect()
	}

	pub fn push(&mut self, s: f32) {
		self.buf[self.pos] = s;
		self.pos = self.buf.constrain(self.pos + 1);
	}
}
