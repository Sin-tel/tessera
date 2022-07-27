use crate::dsp::lerp;

#[derive(Debug)]
pub struct Smoothed {
	value: f32,
	inner: f32,
	f: f32,
}

impl Smoothed {
	pub fn new(value: f32, f: f32, sample_rate: f32) -> Self {
		Smoothed {
			inner: value,
			value,
			f: f / sample_rate,
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
}

impl Default for Smoothed {
	fn default() -> Self {
		Smoothed::new(0.0, 0.5, 1.0)
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
	pub fn new(value: f32, attack: f32, release: f32, sample_rate: f32) -> Self {
		SmoothedEnv {
			inner: value,
			value,
			attack: attack / sample_rate,
			release: release / sample_rate,
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
}

impl Default for SmoothedEnv {
	fn default() -> Self {
		SmoothedEnv::new(0.0, 0.5, 0.1, 1.0)
	}
}
