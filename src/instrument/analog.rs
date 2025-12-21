use fastrand::Rng;
use halfband::iir::design::compute_coefs_tbw;
use halfband::iir::{Downsampler, Upsampler};
use std::f32::consts::PI;

use crate::audio::MAX_BUF_SIZE;
use crate::dsp::env::*;
use crate::dsp::skf::{FilterMode, Skf};
use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use crate::instrument::*;

// TODO: at high frequencies, the switching between different BLITs causes discontinuities
// TODO: LFO (tri, s&h, noise, saw) -> PWM, pitch
// TODO: env -> PWM, pitch

const MAX_F: f32 = 20_000.0;

#[derive(Debug)]
pub struct Analog {
	freq: Smooth,
	gate: Smooth,
	pres: AttackRelease,
	sample_rate: f32,
	accum: f32,
	z: f32,
	rng: Rng,
	filter: Skf,
	upsampler: Upsampler<4>,
	downsampler: Downsampler<4>,
	dc_killer: DcKiller,
	envelope: Adsr,
	note_on: bool,
	buf_up: [f32; MAX_BUF_SIZE * 2],

	// parameters
	pulse_width: Smooth,
	mix_pulse: f32,
	mix_saw: f32,
	mix_sub: f32,
	mix_noise: f32,
	vcf_mode: FilterMode,
	vcf_cutoff: f32,
	vcf_res: f32,
	vcf_env: f32,
	vcf_kbd: f32,
	legato: bool,
	use_gate: bool,
}

impl Instrument for Analog {
	fn new(sample_rate: f32) -> Self {
		let coefs = compute_coefs_tbw(8, 0.0343747);
		let upsampler = Upsampler::new(&coefs);
		let downsampler = Downsampler::new(&coefs);

		Self {
			freq: Smooth::new(0.01, 8.0, sample_rate),
			gate: Smooth::new(0., 2.0, sample_rate),
			pres: AttackRelease::new(50.0, 120.0, sample_rate),
			sample_rate,
			envelope: Adsr::new(sample_rate),
			filter: Skf::new(2.0 * sample_rate),
			dc_killer: DcKiller::new(sample_rate),
			accum: 0.,
			upsampler,
			downsampler,
			buf_up: [0.0; MAX_BUF_SIZE * 2],
			z: 0.,
			rng: Rng::new(),
			note_on: false,

			pulse_width: Smooth::new(0., 25.0, sample_rate),
			mix_pulse: 0.,
			mix_saw: 0.,
			mix_sub: 0.,
			mix_noise: 0.,
			vcf_mode: FilterMode::default(),
			vcf_cutoff: 0.,
			vcf_res: 0.,
			vcf_env: 0.,
			vcf_kbd: 0.,
			legato: true,
			use_gate: false,
		}
	}

	fn voice_count(&self) -> usize {
		1
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		if self.envelope.done() {
			return;
		}
		let [bl, br] = buffer;

		let up_len = 2 * bl.len();

		self.update_filter();

		// Oscillator
		for sample in bl.iter_mut() {
			let _pres = self.pres.process();
			let freq = self.freq.process();
			let pulse_width = self.pulse_width.process();

			let f_sub = 0.5 * freq;

			self.accum += f_sub;
			self.accum -= self.accum.floor();

			// calculate maximum allowed partial
			let m = 1.0 + 2.0 * (MAX_F / (self.sample_rate * freq)).floor();
			let m_sub = 1.0 + 2.0 * (MAX_F / (self.sample_rate * f_sub)).floor();

			let s0 = blit(self.accum, m_sub);
			let s1 = blit(self.accum - 0.5, m_sub);
			let s2 = blit(self.accum * 2.0 - pulse_width, m);

			// leaky integrator
			self.z = self.z * 0.998
				+ self.mix_saw * (s0 + s1)
				+ self.mix_pulse * (s0 + s1 - s2)
				+ self.mix_sub * (s0 - s1);

			let mix = self.z + self.mix_noise * (self.rng.f32() - 0.5);

			*sample = mix * 0.20;
		}

		// Upsample + filter
		self.upsampler.process_block(bl, &mut self.buf_up[..up_len]);
		self.filter.process_block(&mut self.buf_up[..up_len], self.vcf_mode);
		self.downsampler.process_block(&self.buf_up[..up_len], bl);

		// Output processing
		for sample in bl.iter_mut() {
			let gate = self.gate.process();
			let env = self.envelope.process();

			let mut out = *sample;
			out *= 5.;
			if self.use_gate {
				out *= gate;
			} else {
				out *= env;
			}
			*sample = out;
		}

		self.dc_killer.process_block(bl);
		br.copy_from_slice(bl);
	}

	fn pitch(&mut self, pitch: f32, _id: usize) {
		let f = pitch_to_hz(pitch) / self.sample_rate;
		self.freq.set(f);
		self.update_filter();
	}

	fn pressure(&mut self, pressure: f32, _id: usize) {
		self.pres.set(pressure);
	}

	fn note_on(&mut self, pitch: f32, vel: f32, _id: usize) {
		let f = pitch_to_hz(pitch) / self.sample_rate;
		self.freq.set(f);
		// make it less sensitive to velocity
		let v = vel * (2. - vel);
		self.envelope.set_vel(v);

		self.gate.set(0.3);
		self.update_filter();
		if !(self.legato && self.note_on) {
			self.envelope.note_on(v);
			self.freq.immediate();
			// TODO: if attack is really fast the filter should be set to max immediately
			self.filter.immediate();
		}
		self.note_on = true;
	}

	fn note_off(&mut self, _id: usize) {
		self.note_on = false;
		self.gate.set(0.0);
		self.envelope.note_off();
	}

	fn flush(&mut self) {
		self.envelope.reset();
		self.gate.set(0.0);
		self.note_on = false;
	}

	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.pulse_width.set(value),
			1 => self.mix_pulse = value,
			2 => self.mix_saw = value,
			3 => self.mix_sub = value,
			4 => self.mix_noise = value,
			5 => {
				self.vcf_mode = match value as usize {
					1 => FilterMode::Lowpass,
					2 => FilterMode::Bandpass,
					3 => FilterMode::Highpass,
					_ => unreachable!(),
				}
			},
			6 => {
				self.vcf_cutoff = hz_to_pitch(value);
				self.update_filter();
			},
			7 => {
				self.vcf_res = value;
				self.update_filter();
			},
			8 => {
				self.vcf_env = value;
				self.update_filter();
			},
			9 => {
				self.vcf_kbd = value;
				self.update_filter();
			},
			10 => self.use_gate = value > 0.5,
			11 => self.envelope.set_attack(value),
			12 => self.envelope.set_decay(value),
			13 => self.envelope.set_sustain(value),
			14 => self.envelope.set_release(value),
			15 => self.legato = value > 0.5,
			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}

// analytical band limited impulse train
// https://www.music.mcgill.ca/~gary/307/week5/node14.html
// the 1/m term is to guarantee the integral is 0
fn blit(s: f32, m: f32) -> f32 {
	let denom = m * (s * PI).sin();
	if denom.abs() < 1e-7 { 1. - (1. / m) } else { ((s * m * PI).sin() / denom) - (1. / m) }
}

impl Analog {
	fn update_filter(&mut self) {
		self.filter.set(
			//TODO: store pitch so we can save hz_to_pitch call?
			pitch_to_hz(
				self.vcf_cutoff
					+ self.vcf_kbd * (hz_to_pitch(self.freq.get() * self.sample_rate) - 72.0)
					+ self.vcf_env * self.envelope.get() * 84.0,
			),
			self.vcf_res,
		);
	}
}
