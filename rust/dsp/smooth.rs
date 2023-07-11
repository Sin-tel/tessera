use crate::audio::MAX_BUF_SIZE;
use crate::dsp::lerp;
use crate::dsp::{time_constant, time_constant_linear};

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
	step_size: f32,
	steps: usize,
	timer: usize,
}

impl SmoothLinear {
	pub fn new(t: f32, sample_rate: f32) -> Self {
		Self {
			target: 0.01,
			value: 0.01,
			steps: (1. / time_constant_linear(t, sample_rate)) as usize,
			step_size: 0.,
			timer: 0,
		}
	}
	pub fn new_steps(steps: usize) -> Self {
		Self {
			target: 0.01,
			value: 0.01,
			step_size: 0.,
			steps,
			timer: 0,
		}
	}

	#[must_use]
	pub fn process(&mut self) -> f32 {
		if self.timer > 0 {
			self.timer -= 1;
			self.value += self.step_size;

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
			self.timer = self.steps;
			self.step_size = (self.target - self.value) / (self.steps as f32);
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
pub struct SmoothBuffer {
	value: f32,
	target: f32,
	buffer: [f32; MAX_BUF_SIZE],
}

impl SmoothBuffer {
	pub fn new() -> Self {
		let v = 0.01;
		Self {
			target: v,
			value: v,
			buffer: [v; MAX_BUF_SIZE],
		}
	}

	pub fn process_buffer(&mut self, len: usize) {
		if self.value == self.target {
			for i in 0..len {
				self.buffer[i] = self.value;
			}
		} else {
			let f = 1. / (len as f32);
			for i in 0..len {
				let a = (i as f32) * f;

				self.buffer[i] = lerp(self.value, self.target, a);
			}

			self.value = self.target;
		}
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

	pub fn get(&self, i: usize) -> f32 {
		self.buffer[i]
	}

	pub fn target(&self) -> f32 {
		self.target
	}
}

impl Default for SmoothExp {
	fn default() -> Self {
		Self::new_direct(0.001)
	}
}

impl Default for SmoothLinear {
	fn default() -> Self {
		Self::new_steps(64)
	}
}

impl Default for SmoothBuffer {
	fn default() -> Self {
		Self::new()
	}
}
