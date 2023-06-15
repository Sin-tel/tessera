use crate::dsp::env::*;
use crate::dsp::*;
use crate::instrument::*;
use core::f32::consts::PI;
use fastrand::Rng;
use std::iter::zip;

const MAX_F: f32 = 20_000.0;

#[derive(Debug, Default)]
pub struct Analog {
	accum: f32,
	freq: Smoothed,
	vel: SmoothedEnv,
	sample_rate: f32,
	z: f32,
	rng: Rng,
	filter: simper::Filter,

	// parameters
	pulse_width: Smoothed,
	mix_pulse: f32,
	mix_saw: f32,
	mix_sub: f32,
	mix_noise: f32,
	vcf_freq: Smoothed,
	vcf_res: f32,
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

impl Instrument for Analog {
	fn new(sample_rate: f32) -> Self {
		Analog {
			freq: Smoothed::new(20.0, sample_rate),
			vel: SmoothedEnv::new(20.0, 50.0, sample_rate),
			pulse_width: Smoothed::new(20.0, sample_rate),
			sample_rate,
			rng: fastrand::Rng::new(),
			filter: simper::Filter::new(sample_rate),
			vcf_freq: Smoothed::new(20.0, sample_rate),

			..Default::default()
		}
	}

	fn cv(&mut self, pitch: f32, vel: f32) {
		let p = pitch_to_f(pitch, self.sample_rate);
		self.freq.set(p);
		self.vel.set(vel);
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			self.vel.update();
			self.freq.update();
			self.pulse_width.update();
			self.vcf_freq.update();

			self.filter.set_lowpass(self.vcf_freq.get(), self.vcf_res);

			let f_sub = 0.5 * self.freq.get();

			self.accum += f_sub;
			self.accum = self.accum.fract();

			let a = (self.accum * 2.0).fract();

			// calculate maximum allowed partial
			let m = 1.0 + 2.0 * (MAX_F / (self.sample_rate * self.freq.get())).round();
			let m_sub = 1.0 + 2.0 * (MAX_F / (self.sample_rate * f_sub)).round();

			let s_sub = blit(self.accum, m_sub) - blit(self.accum - 0.5, m_sub);

			let s1 = blit(a, m);
			let s2 = blit(a - self.pulse_width.get(), m);

			// leaky integrator
			self.z = self.z * 0.999
				+ self.mix_saw * (s1 - 1.0 / m)
				+ self.mix_pulse * (s1 - s2)
				+ self.mix_sub * s_sub;

			let mut out = self.z + self.mix_noise * (self.rng.f32() - 0.5);

			out *= self.vel.get();

			out = self.filter.process(out);

			*l = out;
			*r = out;
		}
	}

	fn note(&mut self, pitch: f32, vel: f32, _id: usize) {
		let p = pitch_to_f(pitch, self.sample_rate);
		self.freq.set_hard(p);

		// self.vel.set_hard(vel);
		// self.accum = 0.0;

		if self.vel.get() < 0.01 {
			self.vel.set_hard(vel);
			self.accum = 0.5;
			self.z = 0.0;
		} else {
			self.vel.set(vel);
		}
	}
	fn set_param(&mut self, index: usize, value: f32) {
		match index {
			0 => self.pulse_width.set(value),
			1 => self.mix_pulse = value,
			2 => self.mix_saw = value,
			3 => self.mix_sub = value,
			4 => self.mix_noise = value,
			5 => self.vcf_freq.set(value),
			6 => self.vcf_res = value,
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}
