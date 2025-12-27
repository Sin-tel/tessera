// High quality windowed sinc resampling.
// Pretty expensive, only done once when loading a file. Not real-time safe!
// Strategy: first do some large N times oversampling (here 32x), then use linear interpolation for the fractional offsets.

use windowfunctions::{Symmetry, WindowFunction, window};

#[inline]
fn sinc(x: f32) -> f32 {
	if x.abs() < 1e-6 {
		1.0
	} else {
		let px = x * std::f32::consts::PI;
		px.sin() / px
	}
}

#[inline]
fn make_usize_frac(x: f64) -> (usize, f32) {
	let x_int = x.floor();
	let x_frac = x - x_int;

	(x_int as usize, x_frac as f32)
}

pub struct Resampler {
	ratio: f64,
	oversample_factor: usize,
	window_size: usize,
	table: Vec<Vec<f32>>,
}

// increase window size for steeper filter (expensive)
// increase oversample_factor size for better interpolation quality (generally cheap)
impl Resampler {
	pub fn new(source_rate: f32, target_rate: f32) -> Self {
		// reasonable defaults for various common cases (e.g. 48kHz <-> 44.1kHz)
		let oversample_factor = 32;
		let window_size = 16;
		let window_type = WindowFunction::Kaiser { beta: 6.0 };

		let ratio = source_rate as f64 / target_rate as f64;

		let mut resampler = Resampler {
			ratio,
			oversample_factor,
			window_size,
			table: Vec::with_capacity(oversample_factor + 1),
		};

		resampler.build_table(window_type);
		resampler
	}

	fn build_table(&mut self, window_type: WindowFunction) {
		let taps = self.window_size * 2;

		// maintain original nyquist when upsampling, filter to new nyquist when downsampling
		let scale_factor = if self.ratio > 1.0 { (1.0 / self.ratio) as f32 } else { 1.0 };

		let filter_len = taps * self.oversample_factor + 1;
		let center = self.window_size * self.oversample_factor;

		let filter: Vec<f32> = window::<f32>(filter_len, window_type, Symmetry::Symmetric)
			.enumerate()
			.map(|(i, w)| {
				let x = (i as f32 - center as f32) / self.oversample_factor as f32;
				sinc(x * scale_factor) * w * scale_factor
			})
			.collect();

		// build polyphase branches
		for phase in 0..=self.oversample_factor {
			let mut coeffs = Vec::with_capacity(taps);
			for i in 0..taps {
				let idx = (i + 1) * self.oversample_factor - phase;
				let val = filter[idx];
				coeffs.push(val);
			}
			self.table.push(coeffs);
		}
	}

	pub fn process(&self, input: &[f32]) -> Vec<f32> {
		let output_len = (input.len() as f64 / self.ratio).ceil() as usize;
		let mut output = Vec::with_capacity(output_len);

		let offset = self.window_size as isize - 1;
		let input_len = input.len() as isize;

		for i in 0..output_len {
			// source position
			let pos = (i as f64) * self.ratio;
			let (pos_int, pos_frac) = make_usize_frac(pos);

			// offset into polyphase
			let phase = pos_frac * self.oversample_factor as f32;
			let (phase_int, phase_frac) = make_usize_frac(phase as f64);

			// get two polyphase branches
			let coeffs_a = &self.table[phase_int];
			let coeffs_b = &self.table[phase_int + 1];

			// convolution
			let mut accum = 0.0;
			for i in 0..coeffs_a.len() {
				let sample_idx = (pos_int + i) as isize - offset;
				if sample_idx >= 0 && sample_idx < input_len {
					let sample = input[sample_idx as usize];

					// linear interpolation
					let a = coeffs_a[i];
					let b = coeffs_b[i];
					let coeff = a + (b - a) * phase_frac;

					accum += sample * coeff;
				}
			}
			output.push(accum);
		}
		output
	}
}
