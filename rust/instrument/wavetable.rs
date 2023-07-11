const WT_SIZE: usize = 1024;
const WT_MASK: usize = 1023;
const WT_NUM: usize = 16;

// TODO: we should probably just support the wavetable format used by Surge
// see: https://github.com/surge-synthesizer/surge/blob/main/resources/data/wavetables/WT%20fileformat.txt

// TODO: simd? https://docs.rs/rustfft/6.1.0/rustfft/struct.FftPlannerAvx.html

// TODO: probably faster to store all of the wavetables in frequency domain and then mix those (only requires fwd fft)

use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use std::fs::File;
use std::io::{BufReader, Read};
use std::iter::zip;
use std::sync::Arc;

use crate::dsp::env::*;
use crate::dsp::smooth::*;
use crate::dsp::*;
use crate::instrument::*;

const MAX_F: f32 = 20_000.0;

pub struct Wavetable {
	accum: f32,
	freq: SmoothExp,
	vel: AttackRelease,
	pres: AttackRelease,
	sample_rate: f32,
	interpolate: f32,
	buffer_a: Vec<f32>,
	buffer_b: Vec<f32>,
	spectrum: Vec<Complex<f32>>,
	r2c_scratch: Vec<Complex<f32>>,
	r2c: Arc<dyn RealToComplex<f32>>,
	c2r_scratch: Vec<Complex<f32>>,
	c2r: Arc<dyn ComplexToReal<f32>>,
	table: [f32; 16384],

	depth_vel: f32,
	depth_pres: f32,
}

impl Instrument for Wavetable {
	fn new(sample_rate: f32) -> Self {
		let mut real_planner = RealFftPlanner::<f32>::new();
		let r2c = real_planner.plan_fft_forward(WT_SIZE);
		let c2r = real_planner.plan_fft_inverse(WT_SIZE);

		let spectrum = r2c.make_output_vec();
		let r2c_scratch = r2c.make_scratch_vec();
		let c2r_scratch = c2r.make_scratch_vec();
		let buffer_a = c2r.make_output_vec();
		let buffer_b = c2r.make_output_vec();

		// read binary file
		let file = File::open("./res/wavetable.bin").unwrap();
		let mut reader = BufReader::new(file);
		let mut table = [0.0f32; WT_SIZE * WT_NUM];
		let mut buffer = [0u8; 4];
		for v in table.iter_mut() {
			reader.read_exact(&mut buffer).unwrap();
			*v = f32::from_le_bytes(buffer);
		}

		let mut new = Wavetable {
			accum: 0.0,
			interpolate: 1.0,
			freq: SmoothExp::new(20.0, sample_rate),
			vel: AttackRelease::new(20.0, 120.0, sample_rate),
			pres: AttackRelease::new(50.0, 500.0, sample_rate),
			sample_rate,
			buffer_a,
			buffer_b,
			spectrum,
			r2c_scratch,
			c2r_scratch,
			r2c,
			c2r,
			table,

			depth_vel: 0.0,
			depth_pres: 0.0,
		};
		new.update_fft();
		new
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		if self.interpolate >= 1.0 {
			self.interpolate = 0.0;
			self.update_fft();
		}

		let [bl, br] = buffer;

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			self.interpolate += 1.0 / (0.05 * self.sample_rate); // update every 50ms

			let _pres = self.pres.process();
			let vel = self.vel.process();
			let freq = self.freq.process();

			self.accum += freq;
			if self.accum >= 1.0 {
				self.accum -= 1.0;
			}

			let idx = self.accum * (WT_SIZE as f32);
			let (idx_int, idx_frac) = make_usize_frac(idx);

			// bilinear interpolation between samples and buffers
			let w1a = self.buffer_a[idx_int];
			let w2a = self.buffer_a[(idx_int + 1) & WT_MASK];
			let wa = lerp(w1a, w2a, idx_frac);

			let w1b = self.buffer_b[idx_int];
			let w2b = self.buffer_b[(idx_int + 1) & WT_MASK];
			let wb = lerp(w1b, w2b, idx_frac);

			let mut out = lerp(wa, wb, self.interpolate.clamp(0.0, 1.0));

			out *= vel;

			*l = out;
			*r = out;
		}
	}

	fn cv(&mut self, pitch: f32, pres: f32, _id: usize) {
		let p = pitch_to_hz(pitch) / self.sample_rate;
		self.freq.set(p);
		self.pres.set(pres);
	}

	fn note(&mut self, pitch: f32, vel: f32, _id: usize) {
		if vel == 0.0 {
			self.vel.set(0.0);
		} else {
			let p = pitch_to_hz(pitch) / self.sample_rate;
			self.freq.set_immediate(p);

			// if self.vel.get() < 0.01 {
			// self.vel.set_immediate(vel);
			// self.accum = 0.0;
			self.interpolate = 1.0;
			// } else {
			self.vel.set(vel);
			// }
		}
	}

	#[allow(clippy::match_single_binding)]
	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.depth_vel = value,
			1 => self.depth_pres = value,
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}

impl Wavetable {
	fn update_fft(&mut self) {
		// linear interpolation between frames
		let wdepth = self.depth_vel * self.vel.get() + self.depth_pres * self.pres.get();
		let wt_idx = (wdepth * (WT_NUM as f32)).clamp(0.0, (WT_NUM as f32) - 1.001);

		let (wt_idx_int, wt_idx_frac) = make_usize_frac(wt_idx);

		for (i, v) in self.buffer_a.iter_mut().enumerate() {
			let w1 = self.table[wt_idx_int * WT_SIZE + i];
			let w2 = self.table[(wt_idx_int + 1) * WT_SIZE + i];
			*v = lerp(w1, w2, wt_idx_frac);
		}

		// forward fft
		self.r2c
			.process_with_scratch(
				&mut self.buffer_a,
				&mut self.spectrum,
				&mut self.r2c_scratch,
			)
			.unwrap(); // only panics when passed incorrect buffer sizes

		// calculate maximum allowed partial
		let p_max = (MAX_F / (self.sample_rate * self.freq.get())) as usize;

		// zero out everything above p_max
		for (i, x) in self.spectrum.iter_mut().enumerate() {
			if i > p_max {
				*x = Zero::zero();
			}
		}

		// inverse fft
		self.c2r
			.process_with_scratch(
				&mut self.spectrum,
				&mut self.buffer_a,
				&mut self.c2r_scratch,
			)
			.unwrap(); // only panics when passed incorrect buffer sizes

		// normalize
		for v in &mut self.buffer_a {
			*v /= WT_SIZE as f32;
		}

		std::mem::swap(&mut self.buffer_a, &mut self.buffer_b);
	}
}
