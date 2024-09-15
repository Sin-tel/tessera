use bit_mask_ring_buf::BMRingBuf;
use realfft::{RealFftPlanner, RealToComplex};
use ringbuf::traits::*;
use ringbuf::HeapCons;
use std::sync::Arc;

use crate::audio::SPECTRUM_SIZE;
use crate::dsp::TWO_PI;

pub struct Scope {
	buf: BMRingBuf<f32>,
	pos: isize,
	r2c: Arc<dyn RealToComplex<f32>>,
	rx: HeapCons<f32>,
}

impl Scope {
	pub fn new(rx: HeapCons<f32>) -> Self {
		let mut real_planner = RealFftPlanner::<f32>::new();
		let r2c = real_planner.plan_fft_forward(SPECTRUM_SIZE);

		Scope { buf: BMRingBuf::<f32>::from_len(SPECTRUM_SIZE), pos: 0, r2c, rx }
	}

	pub fn get_spectrum(&self) -> Vec<f64> {
		let (a, b) = self.buf.as_slices(self.pos);

		let mut in_buffer = [a, b].concat();
		let mut spectrum = self.r2c.make_output_vec();

		// Apply Hann window
		for (i, v) in in_buffer.iter_mut().enumerate() {
			*v *= 0.5 * (1.0 - ((TWO_PI * (i as f32)) / (SPECTRUM_SIZE as f32)).cos());
		}

		// Forward fft
		self.r2c.process(&mut in_buffer, &mut spectrum).unwrap();

		// Normalize and calculate norm
		let scale = 1.0 / (SPECTRUM_SIZE as f32).sqrt();
		spectrum.iter().map(|&e| f64::from(e.norm() * scale)).collect()
	}

	pub fn get_oscilloscope(&self) -> Vec<f64> {
		let (a, b) = self.buf.as_slices(self.pos);

		[a, b].concat().iter().map(|&e| f64::from(e)).collect()
	}

	pub fn update(&mut self) {
		for sample in self.rx.pop_iter() {
			self.buf[self.pos] = sample;
			self.pos = self.buf.constrain(self.pos + 1);
		}
	}
}
