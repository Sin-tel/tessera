// after Andrew Simper, Cytomic, 2013
// see: https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf

// use resonance instead of Q?
// k = 2 - 2*res
// res = 1 - 0.5 / Q

// TODO: smoothing should be done by whoever owns the filter, but  this is kind of annoying since theres 6 values

#![allow(dead_code)]

use crate::dsp::env::SmoothLinear;
use crate::dsp::{from_db, prewarp};

#[derive(Debug, Default)]
pub struct Filter {
	sample_rate: f32,
	s1: f32,
	s2: f32,

	a1: SmoothLinear,
	a2: SmoothLinear,
	a3: SmoothLinear,

	m0: SmoothLinear,
	m1: SmoothLinear,
	m2: SmoothLinear,
}

impl Filter {
	pub fn new(sample_rate: f32) -> Self {
		Self {
			sample_rate,
			a1: SmoothLinear::new(5.0, sample_rate),
			a2: SmoothLinear::new(5.0, sample_rate),
			a3: SmoothLinear::new(5.0, sample_rate),
			m0: SmoothLinear::new(5.0, sample_rate),
			m1: SmoothLinear::new(5.0, sample_rate),
			m2: SmoothLinear::new(5.0, sample_rate),
			..Default::default()
		}
	}

	fn set_coefs(&mut self, g: f32, k: f32) {
		let a1 = 1.0 / (1.0 + g * (g + k));
		let a2 = g * a1;
		let a3 = g * a2;

		self.a1.set(a1);
		self.a2.set(a2);
		self.a3.set(a3);
	}

	pub fn set_lowpass(&mut self, cutoff: f32, q: f32) {
		let g = prewarp(cutoff / self.sample_rate);
		let k = 1.0 / q;
		self.set_coefs(g, k);
		self.m0.set(0.0);
		self.m1.set(0.0);
		self.m2.set(1.0);
	}
	pub fn set_bandpass(&mut self, cutoff: f32, q: f32) {
		let g = prewarp(cutoff / self.sample_rate);
		let k = 1.0 / q;
		self.set_coefs(g, k);
		self.m0.set(0.0);
		self.m1.set(1.0);
		self.m2.set(0.0);
	}
	pub fn set_bandpass_norm(&mut self, cutoff: f32, q: f32) {
		let g = prewarp(cutoff / self.sample_rate);
		let k = 1.0 / q;
		self.set_coefs(g, k);
		self.m0.set(0.0);
		self.m1.set(k);
		self.m2.set(0.0);
	}
	pub fn set_highpass(&mut self, cutoff: f32, q: f32) {
		let g = prewarp(cutoff / self.sample_rate);
		let k = 1.0 / q;
		self.set_coefs(g, k);
		self.m0.set(1.0);
		self.m1.set(-k);
		self.m2.set(-1.0);
	}
	pub fn set_notch(&mut self, cutoff: f32, q: f32) {
		let g = prewarp(cutoff / self.sample_rate);
		let k = 1.0 / q;
		self.set_coefs(g, k);
		self.m0.set(1.0);
		self.m1.set(-k);
		self.m2.set(0.0);
	}
	pub fn set_allpass(&mut self, cutoff: f32, q: f32) {
		let g = prewarp(cutoff / self.sample_rate);
		let k = 1.0 / q;
		self.set_coefs(g, k);
		self.m0.set(1.0);
		self.m1.set(-2.0 * k);
		self.m2.set(0.0);
	}
	pub fn set_bell(&mut self, cutoff: f32, q: f32, gain: f32) {
		let a = from_db(0.5 * gain);
		let g = prewarp(cutoff / self.sample_rate);
		let k = 1.0 / (q * a);
		self.set_coefs(g, k);
		self.m0.set(1.0);
		self.m1.set(k * (a * a - 1.0));
		self.m2.set(0.0);
	}
	pub fn set_lowshelf(&mut self, cutoff: f32, q: f32, gain: f32) {
		let a = from_db(0.5 * gain);
		let g = prewarp(cutoff / self.sample_rate) / a.sqrt();
		let k = 1.0 / q;
		self.set_coefs(g, k);
		self.m0.set(1.0);
		self.m1.set(k * (a - 1.0));
		self.m2.set(a * a - 1.0);
	}
	pub fn set_highshelf(&mut self, cutoff: f32, q: f32, gain: f32) {
		let a = from_db(0.5 * gain);
		let g = prewarp(cutoff / self.sample_rate) * a.sqrt();
		let k = 1.0 / q;
		self.set_coefs(g, k);
		self.m0.set(a * a);
		self.m1.set(k * (1.0 - a) * a);
		self.m2.set(1.0 - a * a);
	}
	pub fn set_tilt(&mut self, cutoff: f32, q: f32, gain: f32) {
		let a = from_db(0.5 * gain);
		let g = prewarp(cutoff / self.sample_rate) * a.sqrt();
		let k = 1.0 / q;
		self.set_coefs(g, k);
		self.m0.set(a);
		self.m1.set(k * (1.0 - a));
		self.m2.set(1.0 / a - a);
	}

	#[must_use]
	pub fn process(&mut self, v0: f32) -> f32 {
		let a1 = self.a1.process();
		let a2 = self.a2.process();
		let a3 = self.a3.process();
		let m0 = self.m0.process();
		let m1 = self.m1.process();
		let m2 = self.m2.process();

		let v3 = v0 - self.s2;
		let v1 = a1 * self.s1 + a2 * v3;
		let v2 = self.s2 + a2 * self.s1 + a3 * v3;
		self.s1 = 2. * v1 - self.s1;
		self.s2 = 2. * v2 - self.s2;

		m0 * v0 + m1 * v1 + m2 * v2
	}
}
