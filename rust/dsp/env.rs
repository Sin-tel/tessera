use crate::dsp::*;

#[derive(Debug)]
pub struct AttackRelease {
	value: f32,
	target: f32,
	attack: f32,
	release: f32,
	sample_rate: f32,
}

impl AttackRelease {
	pub fn new(attack: f32, release: f32, sample_rate: f32) -> Self {
		Self {
			target: 0.0,
			value: 0.0,
			attack: time_constant(attack, sample_rate),
			release: time_constant(release, sample_rate),
			sample_rate,
		}
	}

	pub fn new_direct(attack: f32, release: f32) -> Self {
		Self {
			target: 0.0,
			value: 0.0,
			attack,
			release,
			sample_rate: 1.0,
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

	pub fn set_attack(&mut self, attack: f32) {
		self.attack = time_constant(attack, self.sample_rate);
	}

	pub fn set_release(&mut self, release: f32) {
		self.release = time_constant(release, self.sample_rate);
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

		let vel_scale = 1. + 20. * self.vel.powi(2);
		self.attack_step = self.attack * self.vel * vel_scale;
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

impl Default for AttackRelease {
	fn default() -> Self {
		Self::new_direct(0.005, 0.001)
	}
}
