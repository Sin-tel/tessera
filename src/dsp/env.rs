use crate::dsp::lerp;

#[derive(Debug)]
pub struct Smoothed {
	value: f32,
	inner: f32,
	f: f32,
}

fn time_constant(t: f32, sample_rate: f32) -> f32 {
	1.0 - (-1000.0 / (sample_rate * t)).exp()
}

impl Smoothed {
	pub fn new(t: f32, sample_rate: f32) -> Self {
		Smoothed {
			inner: 0.0,
			value: 0.0,
			f: time_constant(t, sample_rate),
		}
	}
	pub fn new_direct(f: f32) -> Self {
		Smoothed {
			inner: 0.0,
			value: 0.0,
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

	pub fn get(&self) -> f32 {
		self.value
	}

	pub fn inner(&self) -> f32 {
		self.inner
	}
}

impl Default for Smoothed {
	fn default() -> Self {
		Smoothed::new_direct(0.5)
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
		SmoothedEnv {
			inner: 0.0,
			value: 0.0,
			attack: time_constant(attack, sample_rate),
			release: time_constant(release, sample_rate),
		}
	}

	pub fn new_direct(attack: f32, release: f32) -> Self {
		SmoothedEnv {
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

impl Default for SmoothedEnv {
	fn default() -> Self {
		SmoothedEnv::new_direct(0.5, 0.1)
	}
}
