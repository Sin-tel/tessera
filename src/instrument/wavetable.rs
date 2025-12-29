// TODO: we should probably just support the wavetable format used by Surge
// see: https://github.com/surge-synthesizer/surge/blob/main/resources/data/wavetables/WT%20fileformat.txt

// TODO: simd? https://docs.rs/rustfft/6.1.0/rustfft/struct.FftPlannerAvx.html

// TODO: probably faster to store all of the wavetables in frequency domain and then mix those (only requires fwd fft)

use crate::dsp::env::*;
use crate::dsp::smooth::*;
use crate::dsp::*;
use crate::instrument::*;
use crate::worker::RequestData;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use std::any::Any;
use std::sync::Arc;

const WT_SIZE: usize = 1024;
const WT_MASK: usize = WT_SIZE - 1;
const WT_NUM: usize = 32;
const WT_TOTAL: usize = WT_NUM * WT_SIZE;
const MAX_F: f32 = 20_000.0;
const VOICE_COUNT: usize = 16;

#[rustfmt::skip]
const PATHS: &[&str] = &[
	"wavetable/fold.wav",
	"wavetable/glass.wav",
	"wavetable/noise.wav",
];

struct Voice {
	active: bool,
	note_on: bool,
	accum: f32,
	freq: Smooth,
	vel: AttackRelease,
	pres: AttackRelease,
	interpolate: f32,

	// parameters
	position: f32,

	// buffers
	buffer_a: Vec<f32>,
	buffer_b: Vec<f32>,
	spectrum: Vec<Complex<f32>>,
}

impl Voice {
	fn new(
		sample_rate: f32,
		r2c: &Arc<dyn RealToComplex<f32>>,
		c2r: &Arc<dyn ComplexToReal<f32>>,
	) -> Self {
		Self {
			active: false,
			note_on: false,
			accum: 0.0,
			interpolate: 1.0,
			freq: Smooth::new(1., 10.0, sample_rate),
			vel: AttackRelease::new(5.0, 120.0, sample_rate),
			pres: AttackRelease::new(50.0, 500.0, sample_rate),

			position: 0.0,

			buffer_a: c2r.make_output_vec(),
			buffer_b: c2r.make_output_vec(),
			spectrum: r2c.make_output_vec(),
		}
	}

	fn trigger(&mut self, pitch_hz: f32, velocity: f32) {
		self.active = true;
		self.note_on = true;
		self.freq.set_immediate(pitch_hz);
		self.vel.set(velocity);
		self.interpolate = 1.0; // force immediate update
	}

	fn release(&mut self) {
		self.note_on = false;
		self.vel.set(0.0);
	}

	fn update_voice_fft(&mut self, sample_rate: f32, data: &mut Data) {
		if let Some(table) = &data.table {
			// linear interpolation between frames
			let wt_idx = (self.position * (WT_NUM as f32)).clamp(0.0, (WT_NUM as f32) - 1.001);
			let (wt_idx_int, wt_idx_frac) = make_usize_frac(wt_idx);

			for (i, v) in self.buffer_a.iter_mut().enumerate() {
				let w1 = table[wt_idx_int * WT_SIZE + i];
				let w2 = table[(wt_idx_int + 1) * WT_SIZE + i];
				*v = lerp(w1, w2, wt_idx_frac);
			}

			// forward fft
			data.r2c
				.process_with_scratch(&mut self.buffer_a, &mut self.spectrum, &mut data.r2c_scratch)
				.unwrap();

			// calculate maximum allowed partial
			let p_max = (MAX_F / (sample_rate * self.freq.get())) as usize;

			// zero out everything above p_max
			for (i, x) in self.spectrum.iter_mut().enumerate() {
				if i > p_max {
					*x = Zero::zero();
				}
			}

			// inverse fft
			data.c2r
				.process_with_scratch(&mut self.spectrum, &mut self.buffer_a, &mut data.c2r_scratch)
				.unwrap();

			// normalize
			let gain = 1. / WT_SIZE as f32;
			for v in &mut self.buffer_a {
				*v *= gain;
			}

			std::mem::swap(&mut self.buffer_a, &mut self.buffer_b);
		} else {
			self.buffer_a.fill(0.);
			self.buffer_b.fill(0.);
		}
	}
}

// FFT data shared between voices
struct Data {
	r2c: Arc<dyn RealToComplex<f32>>,
	c2r: Arc<dyn ComplexToReal<f32>>,
	r2c_scratch: Vec<Complex<f32>>,
	c2r_scratch: Vec<Complex<f32>>,
	table: Option<Arc<Vec<f32>>>,
}

pub struct Wavetable {
	sample_rate: f32,
	voices: [Voice; VOICE_COUNT],
	data: Data,
}

impl Instrument for Wavetable {
	fn new(sample_rate: f32) -> Self {
		assert!(WT_SIZE.is_power_of_two());

		let mut real_planner = RealFftPlanner::<f32>::new();
		let r2c = real_planner.plan_fft_forward(WT_SIZE);
		let c2r = real_planner.plan_fft_inverse(WT_SIZE);
		let r2c_scratch = r2c.make_scratch_vec();
		let c2r_scratch = c2r.make_scratch_vec();

		let voices = std::array::from_fn(|_| Voice::new(sample_rate, &r2c, &c2r));

		let data = Data { r2c, c2r, r2c_scratch, c2r_scratch, table: None };

		Wavetable { voices, sample_rate, data }
	}

	fn voice_count(&self) -> usize {
		VOICE_COUNT
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		// update every 50ms
		let update_speed = 1.0 / (0.05 * self.sample_rate);

		for voice in self.voices.iter_mut().filter(|v| v.active) {
			if !voice.active && voice.vel.get() < 0.0001 {
				voice.active = false;
				continue;
			}
			// TODO: maybe some strategy here to stagger updates
			if voice.interpolate >= 1.0 {
				voice.interpolate = 0.0;
				voice.update_voice_fft(self.sample_rate, &mut self.data);
			}

			for sample in bl.iter_mut() {
				voice.interpolate += update_speed;

				let _pres = voice.pres.process();
				let vel = voice.vel.process();
				let freq = voice.freq.process();

				voice.accum += freq;
				if voice.accum >= 1.0 {
					voice.accum -= 1.0;
				}

				// table lookup and interpolation
				let idx = voice.accum * (WT_SIZE as f32);
				let (idx_int, idx_frac) = make_usize_frac(idx);

				let w1a = voice.buffer_a[idx_int];
				let w2a = voice.buffer_a[(idx_int + 1) & WT_MASK];
				let wa = lerp(w1a, w2a, idx_frac);

				let w1b = voice.buffer_b[idx_int];
				let w2b = voice.buffer_b[(idx_int + 1) & WT_MASK];
				let wb = lerp(w1b, w2b, idx_frac);

				let mut out = lerp(wa, wb, voice.interpolate.clamp(0.0, 1.0));
				out *= vel;

				*sample += out * 0.4;
			}

			if !voice.note_on && voice.vel.get() < 1e-4 {
				voice.active = false;
			}
		}

		br.copy_from_slice(bl);
	}

	fn pitch(&mut self, pitch: f32, id: usize) {
		let p = pitch_to_hz(pitch) / self.sample_rate;
		let voice = &mut self.voices[id];
		voice.freq.set(p);
	}

	fn pressure(&mut self, pressure: f32, id: usize) {
		let voice = &mut self.voices[id];
		voice.pres.set(pressure);
	}

	fn note_on(&mut self, pitch: f32, vel: f32, id: usize) {
		let p = pitch_to_hz(pitch) / self.sample_rate;
		let voice = &mut self.voices[id];
		voice.trigger(p, vel);

		// force update for the new note
		voice.update_voice_fft(self.sample_rate, &mut self.data);
	}

	fn note_off(&mut self, id: usize) {
		let voice = &mut self.voices[id];
		voice.release();
	}

	fn flush(&mut self) {
		for v in &mut self.voices {
			v.vel.set_immediate(0.0);
			v.active = false;
		}
	}

	fn receive_data(&mut self, data: ResponseData) -> Option<Box<dyn Any + Send>> {
		if let ResponseData::Wavetable(new_table) = data {
			assert!(new_table.len() == WT_TOTAL);
			self.data.table = Some(new_table);
		} else {
			unreachable!()
		}
		None
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.voices.iter_mut().for_each(|v| v.position = value),
			1 => {
				let index = (value as usize).max(1) - 1;
				let path = PATHS[index];
				return Some(RequestData::Wavetable(path));
			},
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
