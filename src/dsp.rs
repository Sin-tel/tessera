pub mod delayline;
pub mod simper;

#[inline]
pub fn pitch_to_f(p: f32, sample_rate: f32) -> f32 {
	(2.0_f32).powf((p - 49.0) / 12.0) * 440.0 / sample_rate
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
	a * (1.0 - t) + b * t
}

#[inline]
#[allow(dead_code)]
pub fn from_db(x: f32) -> f32 {
	(10.0f32).powf(x / 20.0)
}

#[derive(Debug)]
pub struct Smoothed {
	pub value: f32,
	inner: f32,
	f: f32,
}

impl Smoothed {
	pub fn new(value: f32, f: f32) -> Self {
		Smoothed {
			inner: value,
			value,
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
}

impl Default for Smoothed {
	fn default() -> Self {
		Smoothed {
			inner: 0.0,
			value: 0.0,
			f: 0.001,
		}
	}
}

#[derive(Debug)]
pub struct SmoothedEnv {
	pub value: f32,
	inner: f32,
	attack: f32,
	release: f32,
}

impl SmoothedEnv {
	pub fn new(value: f32, attack: f32, release: f32) -> Self {
		SmoothedEnv {
			inner: value,
			value,
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
}

impl Default for SmoothedEnv {
	fn default() -> Self {
		SmoothedEnv {
			inner: 0.0,
			value: 0.0,
			attack: 0.002,
			release: 0.0005,
		}
	}
}
