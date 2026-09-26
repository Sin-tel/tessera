use crate::dsp::{from_db, lerp, prewarp, time_constant};

// Coefficients, can be shared between many states
#[derive(Debug, Clone, Copy, Default)]
pub struct OnePoleCoefs {
	g: f32,
	mx: f32,
	my: f32,
}

impl OnePoleCoefs {
	fn new(f: f32, mx: f32, my: f32) -> Self {
		Self { g: f / (1. + f), mx, my }
	}

	pub fn lowpass(cutoff: f32, sample_rate: f32) -> Self {
		let f = prewarp(cutoff / sample_rate);
		Self::new(f, 0., 1.)
	}

	pub fn highpass(cutoff: f32, sample_rate: f32) -> Self {
		let f = prewarp(cutoff / sample_rate);
		Self::new(f, 1., -1.)
	}

	pub fn allpass(cutoff: f32, sample_rate: f32) -> Self {
		let f = prewarp(cutoff / sample_rate);
		Self::new(f, -1., 2.)
	}

	pub fn lowshelf(cutoff: f32, gain: f32, sample_rate: f32) -> Self {
		let a = from_db(0.5 * gain);
		let f = prewarp(cutoff / sample_rate) / a;
		Self::new(f, 1., a * a - 1.)
	}

	pub fn highshelf(cutoff: f32, gain: f32, sample_rate: f32) -> Self {
		let a = from_db(0.5 * gain);
		let f = prewarp(cutoff / sample_rate) * a;
		Self::new(f, a * a, 1. - a * a)
	}

	pub fn tilt(cutoff: f32, gain: f32, sample_rate: f32) -> Self {
		let a = from_db(0.5 * gain);
		let f = prewarp(cutoff / sample_rate) * a;
		Self::new(f, a, 1. / a - a)
	}

	#[must_use]
	pub fn dc_gain(&self) -> f32 {
		self.mx + self.my
	}

	fn lerp(&self, other: &Self, a: f32) -> Self {
		Self {
			g: lerp(self.g, other.g, a),
			mx: lerp(self.mx, other.mx, a),
			my: lerp(self.my, other.my, a),
		}
	}

	fn max_diff(&self, other: &Self) -> f32 {
		(self.g - other.g)
			.abs()
			.max((self.mx - other.mx).abs())
			.max((self.my - other.my).abs())
	}
}

// Integrator state only
#[derive(Debug, Clone, Copy, Default)]
pub struct OnePoleState {
	s: f32,
}

impl OnePoleState {
	pub fn reset(&mut self) {
		self.s = 0.;
	}

	// Set to DC response.
	// Lowpass output equals its input:
	// y = x, v = 0, s = x
	pub fn prime(&mut self, x: f32) {
		self.s = x;
	}

	#[must_use]
	pub fn process(&mut self, c: &OnePoleCoefs, x: f32) -> f32 {
		let v = (x - self.s) * c.g;
		let y = v + self.s;
		self.s = y + v;

		c.mx * x + c.my * y
	}
}

// Filter with smoothed coefficients
#[derive(Debug)]
pub struct OnePole {
	sample_rate: f32,
	state: OnePoleState,
	coefs: OnePoleCoefs,
	target: OnePoleCoefs,
	f: f32,
	done: bool,
}

impl OnePole {
	pub fn new(sample_rate: f32) -> Self {
		Self {
			sample_rate,
			state: OnePoleState::default(),
			coefs: OnePoleCoefs::default(),
			target: OnePoleCoefs::default(),
			f: time_constant(25., sample_rate),
			done: true,
		}
	}

	fn set(&mut self, c: OnePoleCoefs) {
		self.target = c;
		self.done = false;
	}

	pub fn reset_state(&mut self) {
		self.state.reset();
	}

	pub fn prime(&mut self, x: f32) {
		self.state.prime(x);
	}

	#[must_use]
	pub fn dc_gain(&self) -> f32 {
		self.target.dc_gain()
	}

	pub fn reset(&mut self) {
		self.state.reset();
		self.target = OnePoleCoefs::default();
		self.immediate();
	}

	pub fn immediate(&mut self) {
		self.coefs = self.target;
		self.done = true;
	}

	pub fn set_lowpass(&mut self, cutoff: f32) {
		self.set(OnePoleCoefs::lowpass(cutoff, self.sample_rate));
	}

	pub fn set_highpass(&mut self, cutoff: f32) {
		self.set(OnePoleCoefs::highpass(cutoff, self.sample_rate));
	}

	pub fn set_allpass(&mut self, cutoff: f32) {
		self.set(OnePoleCoefs::allpass(cutoff, self.sample_rate));
	}

	pub fn set_lowshelf(&mut self, cutoff: f32, gain: f32) {
		self.set(OnePoleCoefs::lowshelf(cutoff, gain, self.sample_rate));
	}

	pub fn set_highshelf(&mut self, cutoff: f32, gain: f32) {
		self.set(OnePoleCoefs::highshelf(cutoff, gain, self.sample_rate));
	}

	pub fn set_tilt(&mut self, cutoff: f32, gain: f32) {
		self.set(OnePoleCoefs::tilt(cutoff, gain, self.sample_rate));
	}

	#[must_use]
	pub fn process(&mut self, x: f32) -> f32 {
		if !self.done {
			self.coefs = self.coefs.lerp(&self.target, self.f);
			if self.coefs.max_diff(&self.target) < 1e-6 {
				self.coefs = self.target;
				self.done = true;
			}
		}
		self.state.process(&self.coefs, x)
	}

	// Process block in-place
	pub fn process_block(&mut self, buf: &mut [f32]) {
		for s in buf {
			*s = self.process(*s);
		}
	}
}
