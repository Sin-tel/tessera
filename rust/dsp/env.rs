use crate::dsp::{lerp, pow2_cheap};

// millis to tau (time to reach 1/e)
fn time_constant(t: f32, sample_rate: f32) -> f32 {
	// 1.0 - (-1000.0 / (sample_rate * t)).exp())
	const T_LOG2: f32 = -1442.6951;
	1.0 - pow2_cheap(T_LOG2 / (sample_rate * t))
}

#[derive(Debug)]
pub struct Smoothed {
	value: f32,
	inner: f32,
	f: f32,
}

impl Smoothed {
	pub fn new(t: f32, sample_rate: f32) -> Self {
		Self {
			inner: 0.01,
			value: 0.01,
			f: time_constant(t, sample_rate),
		}
	}
	pub fn new_direct(f: f32) -> Self {
		Self {
			inner: 0.01,
			value: 0.01,
			f,
		}
	}
	pub fn update(&mut self) {
		self.value = lerp(self.value, self.inner, self.f);
	}

	pub fn set(&mut self, v: f32) {
		self.inner = v;
	}

	pub fn set_hard(&mut self, v: f32) {
		self.inner = v;
		self.value = v;
	}

	pub fn instant(&mut self) {
		self.value = self.inner;
	}

	pub fn get(&self) -> f32 {
		self.value
	}

	pub fn inner(&self) -> f32 {
		self.inner
	}
}

#[derive(Debug)]
pub struct SmoothedEnv {
	value: f32,
	inner: f32,
	attack: f32,
	release: f32,
}

impl SmoothedEnv {
	pub fn new(attack: f32, release: f32, sample_rate: f32) -> Self {
		Self {
			inner: 0.0,
			value: 0.0,
			attack: time_constant(attack, sample_rate),
			release: time_constant(release, sample_rate),
		}
	}

	pub fn new_direct(attack: f32, release: f32) -> Self {
		Self {
			inner: 0.0,
			value: 0.0,
			attack,
			release,
		}
	}

	pub fn update(&mut self) {
		self.value = lerp(
			self.value,
			self.inner,
			if self.inner > self.value {
				self.attack
			} else {
				self.release
			},
		);
	}

	pub fn set(&mut self, v: f32) {
		self.inner = v;
	}

	pub fn set_hard(&mut self, v: f32) {
		self.inner = v;
		self.value = v;
	}

	pub fn get(&self) -> f32 {
		self.value
	}

	pub fn inner(&self) -> f32 {
		self.inner
	}
}

impl Default for Smoothed {
	fn default() -> Self {
		Self::new_direct(0.001)
	}
}

impl Default for SmoothedEnv {
	fn default() -> Self {
		Self::new_direct(0.005, 0.001)
	}
}
