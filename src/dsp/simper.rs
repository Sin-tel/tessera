// after Andrew Simper, Cytomic, 2013
// see: https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf

// use resonance instead of Q?
// k = 2 - 2*res
// res = 1 - 0.5 / Q

#![allow(dead_code)]

use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use num_complex::Complex;

// TODO: factor out the smoothing 'done' checks (also in onepole)

#[derive(Debug)]
pub struct Filter {
	sample_rate: f32,
	k: f32,
	g: f32,
	s1: f32,
	s2: f32,

	a1: Smooth,
	a2: Smooth,
	a3: Smooth,

	m0: Smooth,
	m1: Smooth,
	m2: Smooth,
}

impl Filter {
	pub fn new(sample_rate: f32) -> Self {
		Self {
			sample_rate,
			a1: Smooth::new(0., 25., sample_rate),
			a2: Smooth::new(0., 25., sample_rate),
			a3: Smooth::new(0., 25., sample_rate),
			m0: Smooth::new(0., 25., sample_rate),
			m1: Smooth::new(0., 25., sample_rate),
			m2: Smooth::new(0., 25., sample_rate),
			s1: 0.,
			s2: 0.,
			k: 0.,
			g: 0.,
		}
	}

	pub fn reset_state(&mut self) {
		self.s1 = 0.;
		self.s2 = 0.;
	}

	pub fn immediate(&mut self) {
		self.a1.immediate();
		self.a2.immediate();
		self.a3.immediate();
		self.m0.immediate();
		self.m1.immediate();
		self.m2.immediate();
	}

	fn set_coefs(&mut self, g: f32, k: f32) {
		self.g = g;
		self.k = k;

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
		let a = from_db(0.25 * gain);
		let g = prewarp(cutoff / self.sample_rate) / a;
		let k = 1.0 / q;
		self.set_coefs(g, k);

		let a2 = a * a;
		self.m0.set(1.0);
		self.m1.set(k * (a2 - 1.0));
		self.m2.set(a2 * a2 - 1.0);
	}
	pub fn set_highshelf(&mut self, cutoff: f32, q: f32, gain: f32) {
		let a = from_db(0.25 * gain);
		let g = prewarp(cutoff / self.sample_rate) * a;
		let k = 1.0 / q;
		self.set_coefs(g, k);

		let a2 = a * a;
		self.m0.set(a2 * a2);
		self.m1.set(k * (1.0 - a2) * a2);
		self.m2.set(1.0 - a2 * a2);
	}
	pub fn set_tilt(&mut self, cutoff: f32, q: f32, gain: f32) {
		let a = from_db(0.25 * gain);
		let g = prewarp(cutoff / self.sample_rate) * a;
		let k = 1.0 / q;
		self.set_coefs(g, k);

		let a2 = a * a;
		self.m0.set(a2);
		self.m1.set(k * (1.0 - a2));
		self.m2.set(1.0 / a2 - a2);
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

	#[must_use]
	pub fn predict(&mut self) -> (f32, f32) {
		// Note: not ideal, should probably first process all the params and then update.
		let a1 = self.a1.get();
		let a2 = self.a2.get();
		let a3 = self.a3.get();
		let m0 = self.m0.get();
		let m1 = self.m1.get();
		let m2 = self.m2.get();

		let g = m0 + m1 * a2 + m2 * a3;

		let v3 = -self.s2;
		let v1 = a1 * self.s1 + a2 * v3;
		let v2 = self.s2 + a2 * self.s1 + a3 * v3;

		let s = m1 * v1 + m2 * v2;

		(g, s)
	}

	// Process block in-place
	pub fn process_block(&mut self, buf: &mut [f32]) {
		for s in buf {
			*s = self.process(*s);
		}
	}

	#[must_use]
	pub fn phase_delay(&self, f: f32) -> f32 {
		let g = self.g;
		let k = self.k;
		let m0 = self.m0.target();
		let m1 = self.m1.target();
		let m2 = self.m2.target();

		let g2 = g * g;

		// denominator coefs
		let d0 = 1. + g2 + g * k;
		let d1 = -2. + 2. * g2;
		let d2 = 1. + g2 - g * k;

		// numerator coefs
		let n0 = m0 * d0 + m1 * g + m2 * g2;
		let n1 = m0 * d1 + m2 * 2. * g2;
		let n2 = m0 * d2 - m1 * g + m2 * g2;

		let omega = TWO_PI * f / self.sample_rate;

		// z^0 = 1
		let z0 = Complex::new(1., 0.);
		// z^{-1} = e^{-jw}
		let z1 = Complex::from_polar(1., -omega);
		// z^{-2} = e^{-2jw}
		let z2 = Complex::from_polar(1., -2. * omega);

		// evaluate transfer function
		let n = n0 * z0 + n1 * z1 + n2 * z2;
		let d = d0 * z0 + d1 * z1 + d2 * z2;

		let phase = (n / d).arg();

		// phase delay = -phase / w
		-phase / omega
	}
}
