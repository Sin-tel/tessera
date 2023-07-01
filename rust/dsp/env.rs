use crate::dsp::{lerp, pow2_cheap};

// millis to tau (time to reach 1/e)
pub fn time_constant(t: f32, sample_rate: f32) -> f32 {
	// 1.0 - (-1000.0 / (sample_rate * t)).exp())
	assert!(t > 0.);

	const T_LOG2: f32 = -1442.6951;
	1.0 - pow2_cheap(T_LOG2 / (sample_rate * t))
}

pub fn time_constant_linear(t: f32, sample_rate: f32) -> f32 {
	assert!(t > 0.);

	1000. / (sample_rate * t)
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

	#[must_use]
	pub fn process(&mut self) -> f32 {
		self.value = lerp(self.value, self.inner, self.f);
		self.value
	}

	pub fn set(&mut self, v: f32) {
		self.inner = v;
	}

	pub fn set_immediate(&mut self, v: f32) {
		self.inner = v;
		self.value = v;
	}

	pub fn immediate(&mut self) {
		self.value = self.inner;
	}

	#[must_use]
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

	#[must_use]
	pub fn process(&mut self) -> f32 {
		self.value = lerp(
			self.value,
			self.inner,
			if self.inner > self.value {
				self.attack
			} else {
				self.release
			},
		);

		self.value
	}

	pub fn set(&mut self, v: f32) {
		self.inner = v;
	}

	pub fn set_immediate(&mut self, v: f32) {
		self.inner = v;
		self.value = v;
	}

	#[must_use]
	pub fn get(&self) -> f32 {
		self.value
	}

	pub fn inner(&self) -> f32 {
		self.inner
	}
}

#[derive(Debug)]
enum AdsrStage {
	Attack,
	Sustain,
	Release,
}

// classic ADSR envelope
#[derive(Debug)]
pub struct Adsr {
	attack: f32,
	decay: f32,
	sustain: f32,
	release: f32,
	value: f32,
	stage: AdsrStage,
}

impl Adsr {
	pub fn new(attack: f32, decay: f32, sustain: f32, release: f32, sample_rate: f32) -> Self {
		Self {
			attack: time_constant_linear(attack, sample_rate),
			decay: time_constant(decay, sample_rate),
			sustain,
			release: time_constant(release, sample_rate),
			value: 0.,
			stage: AdsrStage::Release,
		}
	}

	#[must_use]
	pub fn process(&mut self) -> f32 {
		use AdsrStage::*;
		match self.stage {
			Attack => {
				self.value += self.attack;
				if self.value >= 1. {
					self.value = 1.;
					self.stage = Sustain;
				}
			}
			Sustain => self.value = lerp(self.value, self.sustain, self.decay),
			Release => self.value = lerp(self.value, 0., self.release),
		}
		self.value
	}

	#[must_use]
	pub fn get(&self) -> f32 {
		self.value
	}

	pub fn trigger(&mut self, _vel: f32) {
		// TODO: do something with velocity
		self.stage = AdsrStage::Attack;
	}

	pub fn release(&mut self) {
		self.stage = AdsrStage::Release;
	}
}

impl Default for Adsr {
	fn default() -> Self {
		Self::new(1., 1., 1., 1., 44100.)
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
