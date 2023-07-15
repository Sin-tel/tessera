use fastrand::Rng;
use std::f32::consts::PI;
use std::iter::zip;

use crate::dsp::env::*;
use crate::dsp::resample::{Downsampler31, Upsampler19};
use crate::dsp::skf::Skf;
use crate::dsp::smooth::{SmoothExp, SmoothLinear};
use crate::dsp::*;
use crate::instrument::*;

// TODO: at high frequencies, the switching between different BLITs causes discontinuities
// TODO: LFO (tri, s&h, noise, saw) -> PWM, pitch
// TODO: env -> PWM, pitch

const MAX_F: f32 = 20_000.0;

#[derive(Debug, Default)]
enum FilterMode {
	#[default]
	Lowpass,
	Bandpass,
	Highpass,
}

#[derive(Debug)]
pub struct Analog {
	freq: SmoothExp,
	gate: SmoothExp,
	pres: AttackRelease,
	sample_rate: f32,
	accum: f32,
	z: f32,
	rng: Rng,
	filter: Skf,
	upsampler: Upsampler19,
	downsampler: Downsampler31,
	dc_killer: DcKiller,
	envelope: Adsr,
	note_on: bool,

	// parameters
	pulse_width: SmoothLinear,
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
		Self {
			freq: SmoothExp::new(8.0, sample_rate),
			gate: SmoothExp::new(2.0, sample_rate),
			pres: AttackRelease::new(50.0, 120.0, sample_rate),
			sample_rate,
			envelope: Adsr::new(sample_rate),
			filter: Skf::new(2.0 * sample_rate),
			dc_killer: DcKiller::new(sample_rate),
			accum: 0.,
			upsampler: Upsampler19::default(),
			downsampler: Downsampler31::default(),
			z: 0.,
			rng: Rng::new(),
			note_on: false,

			pulse_width: SmoothLinear::new(20.0, sample_rate),
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

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;
		self.update_filter();
		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			let env = self.envelope.process();
			let _pres = self.pres.process();
			let gate = self.gate.process();
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

			let (mut s1, mut s2) = self.upsampler.process(mix * 0.20);

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

			let mut out = self.downsampler.process(s1, s2);
			out *= 5.;
			if self.use_gate {
				out *= gate;
			} else {
				out *= env;
			}
			out = self.dc_killer.process(out);

			*l = out;
			*r = out;
		}
	}

	fn cv(&mut self, pitch: f32, pres: f32, _id: usize) {
		let f = pitch_to_hz(pitch) / self.sample_rate;
		self.freq.set(f);
		self.pres.set(pres);
		self.update_filter();
	}

	fn note(&mut self, pitch: f32, vel: f32, _id: usize) {
		if vel == 0.0 {
			self.note_on = false;
			self.gate.set(0.0);
			self.envelope.note_off();
		} else {
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
					0 => FilterMode::Lowpass,
					1 => FilterMode::Bandpass,
					2 => FilterMode::Highpass,
					_ => unreachable!(),
				}
			}
			6 => {
				self.vcf_cutoff = hz_to_pitch(value);
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
			10 => self.use_gate = value > 0.5,
			11 => self.envelope.set_attack(value),
			12 => self.envelope.set_decay(value),
			13 => self.envelope.set_sustain(value),
			14 => self.envelope.set_release(value),
			15 => self.legato = value > 0.5,
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}

// analytical band limited impulse train
// https://www.music.mcgill.ca/~gary/307/week5/node14.html
// the 1/m term is to guarantee the integral is 0
fn blit(s: f32, m: f32) -> f32 {
	let denom = m * (s * PI).sin();
	if denom.abs() < 1e-7 {
		1. - (1. / m)
	} else {
		((s * m * PI).sin() / denom) - (1. / m)
	}
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
