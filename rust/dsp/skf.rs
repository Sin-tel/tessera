// Two pole nonlinear Sallen-key filter

use crate::dsp::env::Smoothed;
use std::f32::consts::PI;

#[derive(Debug, Default)]
pub struct Skf {
	sample_rate: f32,
	f: Smoothed,
	r: f32,
	s1: f32,
	s2: f32,
}

impl Skf {
	pub fn new(sample_rate: f32) -> Self {
		Self {
			sample_rate,
			f: Smoothed::new(5.0, sample_rate),
			r: 0.0,
			s1: 0.0,
			s2: 0.0,
		}
	}
	pub fn process(&mut self, x: f32) -> f32 {
		self.f.update();
		let f = self.f.get();

		// evaluate the non-linear gains
		let tk = distdx(self.r * (self.s1 - self.s2)); // feedback
		let t0 = dist2dx(x + tk * self.r * (self.s1 - self.s2)); // input
		let t1 = tanhdx(self.s1); // integrators
		let t2 = tanhdx(self.s2);

		// feedback gains
		let g1 = 1.0 / (1.0 + f * t1);
		let g2 = 1.0 / (1.0 + f * t2);

		// solve for y0
		let y0 = (t0 * x + self.r * t0 * tk * (g1 * self.s1 * (1.0 - f * g2 * t1) - g2 * self.s2))
			/ (t0 * tk * self.r * g1 * (f * f * g2 * t1 - f) + 1.0);

		// solve remaining outputs
		let y1 = t1 * g1 * (self.s1 + f * y0);
		let y2 = t2 * g2 * (self.s2 + f * y1);

		// update state
		self.s1 += 2.0 * f * (y0 - y1);
		self.s2 += 2.0 * f * (y1 - y2);

		// lowpass
		y2

		// bandpass
		// y1 - y2

		// highpass
		// y0 - 2 * y1 + y2
	}

	pub fn set(&mut self, cutoff: f32, res: f32) {
		let f = ((cutoff / self.sample_rate).min(0.49) * PI).tan();
		self.f.set(f);
		self.r = 2.0 * res;
	}
}

// approximate tanh(x)/x
// WolframAlpha: PadeApproximant[Tanh[x]/x,{x,0,{4,4}}]
fn tanhdx(x: f32) -> f32 {
	let a = x * x;
	((a + 105.0) * a + 945.0) / ((15.0 * a + 420.0) * a + 945.0)
}

// diode clipper feedback / x
fn distdx(x: f32) -> f32 {
	let a = 0.135;
	a + (1.0 - a) / (1.0 + 10.0 * x * x).sqrt()
}

// asymmetric input distortion / x
fn dist2dx(x: f32) -> f32 {
	1.0 / (1.0 + x.abs() + 0.2 * x)
}
