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
	"wavetable/bell.wav",
	"wavetable/brass.wav",
	"wavetable/crushed_sine.wav",
	"wavetable/digital.wav",
	"wavetable/drive.wav",
	"wavetable/fold.wav",
	"wavetable/glass.wav",
	"wavetable/liquid.wav",
	"wavetable/organ.wav",
	"wavetable/reese.wav",
	"wavetable/shimmer.wav",
	"wavetable/simple.wav",
	"wavetable/skew.wav",
	"wavetable/squash.wav",
];

struct Lfo {
	phase: f32,
	v_prev: f32,
	v: f32,
	f: f32,
	shape: f32,
	random: f32,
	switch: bool,
	sample_rate: f32,
}

impl Lfo {
	fn new(sample_rate: f32) -> Self {
		Self {
			phase: 0.,
			v_prev: 0.,
			v: 1.,
			f: 0.,
			shape: 0.,
			random: 0.,
			switch: false,
			sample_rate,
		}
	}

	fn step_value(&mut self) {
		self.switch = !self.switch;

		let mut v_new = if self.switch { -1.0 } else { 1.0 };
		v_new = lerp(v_new, fastrand::f32() * 2.0 - 1.0, self.random);

		self.v_prev = self.v;
		self.v = v_new;
	}

	fn reset(&mut self) {
		self.phase = 0.;
		self.switch = false;
		// fill v_prev and v
		self.step_value();
		self.step_value();
	}

	fn tick(&mut self) {
		self.phase += self.f;
		if self.phase > 1.0 {
			self.phase -= 1.0;
			self.step_value();
		}
	}

	fn get(&self) -> f32 {
		let mut alpha = self.phase;
		if self.shape > 0.99 {
			alpha = 0.;
		} else {
			alpha = ((alpha - self.shape) / (1. - self.shape)).max(0.);
		}
		alpha = smoothstep(alpha);
		lerp(self.v_prev, self.v, alpha)
	}

	fn set_rate(&mut self, rate: f32) {
		self.f = 2. * rate / self.sample_rate;
	}
}

struct Voice {
	active: bool,
	note_on: bool,
	accum: f32,
	accum2: f32,
	accum3: f32,
	freq: Smooth,
	pres: AttackRelease,
	interpolate: f32,
	animate: f32,
	lfo: Lfo,
	env: Adsr,
	pos_start: f32,

	// parameters
	pos: f32,
	unison: bool,
	unison_set: bool,
	unison_l: f32,
	unison_r: f32,
	animate_range: f32,
	animate_step: f32,
	lfo_depth: f32,

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
			accum2: 0.33,
			accum3: 0.66,
			interpolate: 1.0,
			animate: 0.0,
			lfo: Lfo::new(sample_rate),
			freq: Smooth::new(1., 10.0, sample_rate),
			env: Adsr::new(sample_rate),
			pres: AttackRelease::new(50.0, 500.0, sample_rate),

			pos: 0.0,
			pos_start: 0.0,
			unison: false,
			unison_set: false,
			unison_l: 1.,
			unison_r: 1.,
			animate_range: 0.0,
			animate_step: 0.0,
			lfo_depth: 0.0,

			buffer_a: c2r.make_output_vec(),
			buffer_b: c2r.make_output_vec(),
			spectrum: r2c.make_output_vec(),
		}
	}

	fn note_on(&mut self, pitch_hz: f32, velocity: f32) {
		self.active = true;
		self.note_on = true;
		self.freq.set_immediate(pitch_hz);
		self.env.note_on(velocity);
		self.animate = 0.;
		self.unison = self.unison_set;

		// LFO always resets
		self.lfo.reset();

		if self.animate_range < 0.0 {
			self.pos_start = self.pos * (1.0 + self.animate_range);
		} else {
			self.pos_start = self.pos * (1.0 - self.animate_range) + self.animate_range;
		}
		self.interpolate = 0.0;
	}

	fn note_off(&mut self) {
		self.env.note_off();
		self.note_on = false;
	}

	// table lookup and interpolation
	fn lookup(&self, a: f32) -> f32 {
		let idx = a * (WT_SIZE as f32);
		let (idx_int, idx_frac) = make_usize_frac(idx);

		let w1a = self.buffer_a[idx_int];
		let w2a = self.buffer_a[(idx_int + 1) & WT_MASK];
		let wa = lerp(w1a, w2a, idx_frac);

		let w1b = self.buffer_b[idx_int];
		let w2b = self.buffer_b[(idx_int + 1) & WT_MASK];
		let wb = lerp(w1b, w2b, idx_frac);

		lerp(wa, wb, self.interpolate.clamp(0.0, 1.0))
	}

	fn current_position(&self) -> f32 {
		let mut pos = lerp(self.pos_start, self.pos, self.animate);
		let lfo = self.lfo.get();
		pos += self.lfo_depth * lfo;

		pos
	}

	fn update_voice_fft(&mut self, sample_rate: f32, data: &mut Data) {
		if let Some(table) = &data.table {
			// linear interpolation between frames
			let pos = self.current_position();
			let wt_idx = (pos * (WT_NUM as f32)).clamp(0.0, (WT_NUM as f32) - 1.001);
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

		Wavetable { sample_rate, voices, data }
	}

	fn voice_count(&self) -> usize {
		VOICE_COUNT
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		// update every 10ms
		let update_speed = 1.0 / (0.01 * self.sample_rate);

		// find voice with higest priority
		let voice = self
			.voices
			.iter_mut()
			.filter(|v| v.active)
			.max_by(|a, b| a.interpolate.total_cmp(&b.interpolate));
		if let Some(voice) = voice {
			if voice.interpolate >= 1.0 {
				voice.interpolate = 0.0;
				voice.update_voice_fft(self.sample_rate, &mut self.data);
			}
		}

		for voice in self.voices.iter_mut().filter(|v| v.active) {
			for (l, r) in bl.iter_mut().zip(br.iter_mut()) {
				let env = voice.env.process();
				let _pres = voice.pres.process();
				let f = voice.freq.process();
				voice.lfo.tick();

				// position mod
				voice.animate = (voice.animate + voice.animate_step).min(1.);
				voice.interpolate += update_speed;

				// oscillators
				voice.accum += f;
				voice.accum -= fast_floor(voice.accum);

				let mut out_l = voice.lookup(voice.accum);
				let mut out_r = out_l;

				if voice.unison {
					voice.accum2 += f * voice.unison_l;
					voice.accum2 -= fast_floor(voice.accum2);
					voice.accum3 += f * voice.unison_r;
					voice.accum3 -= fast_floor(voice.accum3);
					out_l += voice.lookup(voice.accum2);
					out_r += voice.lookup(voice.accum3);

					out_l *= 0.707;
					out_r *= 0.707;
				}

				out_l *= env;
				out_r *= env;

				*l += out_l * 0.25;
				*r += out_r * 0.25;
			}

			if voice.env.done() {
				voice.active = false;
			}
		}
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
		voice.note_on(p, vel);

		// force update for the new note
		voice.update_voice_fft(self.sample_rate, &mut self.data);

		// Initial buffers are the same
		voice.buffer_a.copy_from_slice(&voice.buffer_b);
	}

	fn note_off(&mut self, id: usize) {
		let voice = &mut self.voices[id];
		voice.note_off();
	}

	fn flush(&mut self) {
		for v in &mut self.voices {
			v.env.reset();
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
			0 => self.voices.iter_mut().for_each(|v| v.pos = value),
			1 => {
				let index = (value as usize).max(1) - 1;
				if let Some(path) = PATHS.get(index) {
					return Some(RequestData::Wavetable(path));
				}
				log_warn!("Wavetable index out of bounds: {index}");
			},
			2 => {
				let unison = 0.05 * value * value;
				let unison_l = pow2_cheap(unison);
				let unison_r = pow2_cheap(-unison);
				self.voices.iter_mut().for_each(|v| v.unison_set = value > 0.01);
				self.voices.iter_mut().for_each(|v| v.unison_l = unison_l);
				self.voices.iter_mut().for_each(|v| v.unison_r = unison_r);
			},
			3 => self.voices.iter_mut().for_each(|v| v.animate_range = value),
			4 => {
				let t = time_constant_linear(value, self.sample_rate);
				self.voices.iter_mut().for_each(|v| v.animate_step = t);
			},
			5 => self.voices.iter_mut().for_each(|v| v.env.set_attack(value)),
			6 => self.voices.iter_mut().for_each(|v| v.env.set_decay(value)),
			7 => self.voices.iter_mut().for_each(|v| v.env.set_sustain(value)),
			8 => self.voices.iter_mut().for_each(|v| v.env.set_release(value)),

			9 => self.voices.iter_mut().for_each(|v| v.lfo.set_rate(value)),
			10 => self.voices.iter_mut().for_each(|v| v.lfo.shape = value),
			11 => self.voices.iter_mut().for_each(|v| v.lfo.random = value),
			12 => self.voices.iter_mut().for_each(|v| v.lfo_depth = 0.5 * value),

			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
