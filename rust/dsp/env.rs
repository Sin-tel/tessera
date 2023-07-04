use crate::dsp::{lerp, pow2_cheap};

// millis to tau (time to reach 10^-2)
pub fn time_constant(t: f32, sample_rate: f32) -> f32 {
	// - 1000 * ln(0.01) / ln(2)
	const T_LOG2: f32 = 6643.856;
	// - 1000 * ln(0.01)
	const T_LN: f32 = 4605.1704;

	assert!(t > 0.);

	let denom = sample_rate * t;
	if denom < 1000. {
		1.0 - pow2_cheap(-T_LOG2 / denom)
	} else {
		// 1 - exp(-x) ~ x for small values
		T_LN / denom
	}
}

pub fn time_constant_linear(t: f32, sample_rate: f32) -> f32 {
	assert!(t > 0.);
	1000. / (sample_rate * t)
}

#[derive(Debug)]
pub struct SmoothExp {
	value: f32,
	target: f32,
	f: f32,
}

impl SmoothExp {
	pub fn new(t: f32, sample_rate: f32) -> Self {
		Self {
			target: 0.01,
			value: 0.01,
			f: time_constant(t, sample_rate),
		}
	}
	pub fn new_direct(f: f32) -> Self {
		Self {
			target: 0.01,
			value: 0.01,
			f,
		}
	}

	#[must_use]
	pub fn process(&mut self) -> f32 {
		self.value += self.f * (self.target - self.value);
		self.value
	}

	pub fn set(&mut self, v: f32) {
		self.target = v;
	}

	pub fn set_immediate(&mut self, v: f32) {
		self.target = v;
		self.value = v;
	}

	pub fn immediate(&mut self) {
		self.value = self.target;
	}

	#[must_use]
	pub fn get(&self) -> f32 {
		self.value
	}

	pub fn target(&self) -> f32 {
		self.target
	}
}

#[derive(Debug)]
pub struct SmoothLinear {
	value: f32,
	target: f32,
	step: f32,
	f: f32,
	timer: usize,
}

impl SmoothLinear {
	pub fn new(t: f32, sample_rate: f32) -> Self {
		Self {
			target: 0.01,
			value: 0.01,
			f: time_constant_linear(t, sample_rate),
			step: 0.,
			timer: 0,
		}
	}
	pub fn new_direct(f: f32) -> Self {
		Self {
			target: 0.01,
			value: 0.01,
			step: 0.,
			f,
			timer: 0,
		}
	}

	#[must_use]
	pub fn process(&mut self) -> f32 {
		if self.timer > 0 {
			self.timer -= 1;
			self.value += self.step;

			if self.timer == 0 {
				self.value = self.target;
			}
		}
		self.value
	}

	pub fn set(&mut self, v: f32) {
		self.target = v;
		if self.target == self.value {
			self.timer = 0;
		} else {
			self.step = (self.target - self.value) * self.f;
			self.timer = (1. / self.f).floor() as usize;
		}
	}

	pub fn set_immediate(&mut self, v: f32) {
		self.timer = 0;
		self.target = v;
		self.value = v;
	}

	pub fn immediate(&mut self) {
		self.timer = 0;
		self.value = self.target;
	}

	#[must_use]
	pub fn get(&self) -> f32 {
		self.value
	}

	pub fn target(&self) -> f32 {
		self.target
	}
}

#[derive(Debug)]
pub struct AttackRelease {
	value: f32,
	target: f32,
	attack: f32,
	release: f32,
}

impl AttackRelease {
	pub fn new(attack: f32, release: f32, sample_rate: f32) -> Self {
		Self {
			target: 0.0,
			value: 0.0,
			attack: time_constant(attack, sample_rate),
			release: time_constant(release, sample_rate),
		}
	}

	pub fn new_direct(attack: f32, release: f32) -> Self {
		Self {
			target: 0.0,
			value: 0.0,
			attack,
			release,
		}
	}

	#[must_use]
	pub fn process(&mut self) -> f32 {
		self.value = lerp(
			self.value,
			self.target,
			if self.target > self.value {
				self.attack
			} else {
				self.release
			},
		);

		self.value
	}

	pub fn set(&mut self, v: f32) {
		self.target = v;
	}

	pub fn set_immediate(&mut self, v: f32) {
		self.target = v;
		self.value = v;
	}

	#[must_use]
	pub fn get(&self) -> f32 {
		self.value
	}

	pub fn target(&self) -> f32 {
		self.target
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
	sample_rate: f32,
	attack_step: f32,
	vel: f32,
}

impl Adsr {
	pub fn new(attack: f32, decay: f32, sustain: f32, release: f32, sample_rate: f32) -> Self {
		Self {
			attack: time_constant_linear(attack, sample_rate),
			attack_step: 1.0,
			decay: time_constant(decay, sample_rate),
			sustain,
			release: 1. - time_constant(release, sample_rate),
			value: 0.,
			vel: 1.,
			stage: AdsrStage::Release,
			sample_rate,
		}
	}

	#[must_use]
	pub fn process(&mut self) -> f32 {
		use AdsrStage::*;
		match self.stage {
			Attack => {
				self.value += self.attack_step;
				if self.value >= self.vel {
					self.value = self.vel;
					self.stage = Sustain;
				}
			}
			Sustain => self.value = lerp(self.value, self.sustain * self.vel, self.decay),
			Release => self.value *= self.release,
		}
		self.value
	}

	#[must_use]
	pub fn get(&self) -> f32 {
		self.value
	}

	pub fn note_on(&mut self, vel: f32) {
		self.stage = AdsrStage::Attack;
		self.vel = vel;
		self.attack_step = self.attack * self.vel * (1. + 20. * self.vel * self.vel);
		println!("{:?}", (1. + 20. * self.vel * self.vel));
	}

	pub fn note_off(&mut self) {
		self.stage = AdsrStage::Release;
	}

	pub fn set_attack(&mut self, attack: f32) {
		self.attack = time_constant_linear(attack, self.sample_rate);
	}

	pub fn set_decay(&mut self, decay: f32) {
		self.decay = time_constant(decay, self.sample_rate);
	}

	pub fn set_sustain(&mut self, sustain: f32) {
		self.sustain = sustain;
	}

	pub fn set_release(&mut self, release: f32) {
		self.release = 1.0 - time_constant(release, self.sample_rate);
	}
}

impl Default for Adsr {
	fn default() -> Self {
		Self::new(1., 1., 1., 1., 44100.)
	}
}

impl Default for SmoothExp {
	fn default() -> Self {
		Self::new_direct(0.001)
	}
}

impl Default for SmoothLinear {
	fn default() -> Self {
		Self::new_direct(0.001)
	}
}

impl Default for AttackRelease {
	fn default() -> Self {
		Self::new_direct(0.005, 0.001)
	}
}
