use crate::dsp::smooth::Smooth;
use crate::dsp::{from_db, prewarp};

#[derive(Debug)]
pub struct OnePole {
	sample_rate: f32,
	s: f32,

	g: Smooth,
	my: Smooth,
	mx: Smooth,
}

impl OnePole {
	pub fn new(sample_rate: f32) -> Self {
		Self {
			sample_rate,
			g: Smooth::new(0., 25., sample_rate),
			my: Smooth::new(0., 25., sample_rate),
			mx: Smooth::new(0., 25., sample_rate),

			s: 0.,
		}
	}

	fn set_coef(&mut self, f: f32) {
		let g = f / (1. + f);
		self.g.set(g);
	}

	pub fn reset_state(&mut self) {
		self.s = 0.;
	}

	pub fn immediate(&mut self) {
		self.g.immediate();
		self.my.immediate();
		self.mx.immediate();
	}

	pub fn set_lowpass(&mut self, cutoff: f32) {
		let f = prewarp(cutoff / self.sample_rate);
		self.set_coef(f);
		self.my.set(1.);
		self.mx.set(0.);
	}

	pub fn set_highpass(&mut self, cutoff: f32) {
		let f = prewarp(cutoff / self.sample_rate);
		self.set_coef(f);
		self.my.set(-1.);
		self.mx.set(1.);
	}

	pub fn set_allpass(&mut self, cutoff: f32) {
		let f = prewarp(cutoff / self.sample_rate);
		self.set_coef(f);
		self.my.set(2.);
		self.mx.set(-1.);
	}

	pub fn set_lowshelf(&mut self, cutoff: f32, gain: f32) {
		let a = from_db(0.5 * gain);
		let f = prewarp(cutoff / self.sample_rate) / a;
		self.set_coef(f);
		self.my.set(a * a - 1.);
		self.mx.set(1.);
	}

	pub fn set_highshelf(&mut self, cutoff: f32, gain: f32) {
		let a = from_db(0.5 * gain);
		let f = prewarp(cutoff / self.sample_rate) * a;
		self.set_coef(f);
		self.my.set(1. - a * a);
		self.mx.set(a * a);
	}

	pub fn set_tilt(&mut self, cutoff: f32, gain: f32) {
		let a = from_db(0.5 * gain);
		let f = prewarp(cutoff / self.sample_rate) * a;
		self.set_coef(f);
		self.my.set(1. / a - a);
		self.mx.set(a);
	}

	#[must_use]
	pub fn process(&mut self, x: f32) -> f32 {
		let g = self.g.process();
		let my = self.my.process();
		let mx = self.mx.process();

		let v = (x - self.s) * g;
		let y = v + self.s;
		self.s = y + v;

		mx * x + my * y
	}

	// Process block in-place
	pub fn process_block(&mut self, buf: &mut [f32]) {
		for s in buf {
			*s = self.process(*s);
		}
	}
}
