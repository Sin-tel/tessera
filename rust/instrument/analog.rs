use fastrand::Rng;
use std::f32::consts::PI;
use std::iter::zip;

use crate::dsp::env::*;
use crate::dsp::resample::{Downsampler, Upsampler};
use crate::dsp::skf::Skf;
use crate::dsp::*;
use crate::instrument::*;

const MAX_F: f32 = 20_000.0;

#[derive(Debug, Default)]
enum FilterMode {
	#[default]
	Lowpass,
	Bandpass,
	Highpass,
}

#[derive(Debug, Default)]
pub struct Analog {
	accum: f32,
	freq: Smoothed,
	vel: SmoothedEnv,
	sample_rate: f32,
	z: f32,
	rng: Rng,
	filter: Skf,
	upsampler: Upsampler,
	downsampler: Downsampler,

	// parameters
	pulse_width: Smoothed,
	mix_pulse: f32,
	mix_saw: f32,
	mix_sub: f32,
	mix_noise: f32,
	vcf_mode: FilterMode,
	vcf_pitch: f32,
	vcf_res: f32,
	vcf_env: f32,
	vcf_kbd: f32,
}

impl Instrument for Analog {
	fn new(sample_rate: f32) -> Self {
		Analog {
			freq: Smoothed::new(10.0, sample_rate),
			vel: SmoothedEnv::new(5.0, 120.0, sample_rate),
			sample_rate,
			rng: fastrand::Rng::new(),
			filter: Skf::new(2.0 * sample_rate),

			pulse_width: Smoothed::new(5.0, sample_rate),
			..Default::default()
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			self.vel.update();
			self.freq.update();
			self.pulse_width.update();

			let f_sub = 0.5 * self.freq.get();

			self.accum += f_sub;
			if self.accum > 1.0 {
				self.accum -= 1.0;
			}

			let mut a = self.accum * 2.0;
			if a > 1.0 {
				a -= 1.0;
			}

			// calculate maximum allowed partial
			let m = 1.0 + 2.0 * (MAX_F / (self.sample_rate * self.freq.get())).floor();
			let m_sub = 1.0 + 2.0 * (MAX_F / (self.sample_rate * f_sub)).floor();

			let s_sub = blit(self.accum, m_sub) - blit(self.accum - 0.5, m_sub);

			let s1 = blit(a, m);
			let s2 = blit(a - self.pulse_width.get(), m);

			// leaky integrator
			self.z = self.z * 0.999
				+ self.mix_saw * (s1 - 1.0 / m)
				+ self.mix_pulse * (s1 - s2)
				+ self.mix_sub * s_sub;

			let mix = self.z + self.mix_noise * (self.rng.f32() - 0.5);

			let (mut s1, mut s2) = self.upsampler.process_19(mix * 0.5);

			// TODO: move match branch outside of inner loop
			(s1, s2) = match self.vcf_mode {
				FilterMode::Lowpass => (
					self.filter.process_lowpass(s1),
					self.filter.process_lowpass(s2),
				),
				FilterMode::Bandpass => (
					self.filter.process_bandpass(s1),
					self.filter.process_bandpass(s2),
				),
				FilterMode::Highpass => (
					self.filter.process_highpass(s1),
					self.filter.process_highpass(s2),
				),
			};
			let s = self.downsampler.process_19(s1, s2);
			let out = s * 2.0 * self.vel.get();

			*l = out;
			*r = out;
		}
	}

	fn cv(&mut self, pitch: f32, vel: f32) {
		let f = pitch_to_hz(pitch) / self.sample_rate;
		self.freq.set(f);
		self.vel.set(vel);

		self.update_filter();
	}

	fn note(&mut self, pitch: f32, vel: f32, _id: usize) {
		let f = pitch_to_hz(pitch) / self.sample_rate;
		self.freq.set_hard(f);

		// if self.vel.get() < 0.01 {
		// 	// TODO: filter set hard?
		// 	self.vel.set_hard(vel);
		// 	self.accum = 0.5;
		// 	self.z = 0.0;
		// } else {
		self.vel.set(vel);
		// }

		self.update_filter();
	}
	fn set_param(&mut self, index: usize, value: f32) {
		match index {
			0 => self.pulse_width.set(value),
			1 => self.mix_pulse = value,
			2 => self.mix_saw = value,
			3 => self.mix_sub = value,
			4 => self.mix_noise = value,
			5 => {
				self.vcf_mode = match value as usize {
					0 => FilterMode::Lowpass,
					1 => FilterMode::Bandpass,
					2 => FilterMode::Highpass,
					_ => unreachable!(),
				}
			}
			6 => {
				self.vcf_pitch = hz_to_pitch(value);
				self.update_filter();
			}
			7 => {
				self.vcf_res = value;
				self.update_filter();
			}
			8 => {
				self.vcf_env = value;
				self.update_filter();
			}
			9 => {
				self.vcf_kbd = value;
				self.update_filter();
			}

			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}

// analytical band limited impulse train
// https://www.music.mcgill.ca/~gary/307/week5/node14.html
fn blit(s: f32, m: f32) -> f32 {
	if s.abs() < 1e-7 {
		1.0
	} else {
		(s * m * PI).sin() / (m * (s * PI).sin())
	}
}

impl Analog {
	fn update_filter(&mut self) {
		self.filter.set(
			//TODO: store pitch so we can save hz_to_pitch call?
			pitch_to_hz(
				self.vcf_pitch
					+ self.vcf_kbd * (hz_to_pitch(self.freq.inner() * self.sample_rate) - 72.0)
					+ self.vcf_env * self.vel.inner() * 84.0,
			),
			self.vcf_res,
		);
	}
}
