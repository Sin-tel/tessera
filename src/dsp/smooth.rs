use crate::audio::MAX_BUF_SIZE;
use crate::dsp::lerp;
use crate::dsp::time_constant;

#[derive(Debug)]
pub struct Smooth {
	value: f32,
	target: f32,
	f: f32,
	done: bool,
}

impl Smooth {
	pub fn new(value: f32, t: f32, sample_rate: f32) -> Self {
		Self { target: value, value, f: time_constant(t, sample_rate), done: false }
	}
	pub fn new_direct(value: f32, f: f32) -> Self {
		Self { target: value, value, f, done: false }
	}

	#[must_use]
	pub fn process(&mut self) -> f32 {
		if !self.done {
			self.value += self.f * (self.target - self.value);

			if (self.value - self.target).abs() < 1e-4 {
				self.value = self.target;
				self.done = true;
			}
		}
		self.value
	}

	pub fn set(&mut self, v: f32) {
		self.target = v;
		self.done = false;
	}

	pub fn set_immediate(&mut self, v: f32) {
		self.target = v;
		self.value = v;
		self.done = true;
	}

	pub fn immediate(&mut self) {
		self.value = self.target;
		self.done = true;
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
pub struct LinearBuffer {
	value: f32,
	target: f32,
	buffer: [f32; MAX_BUF_SIZE],
}

impl LinearBuffer {
	pub fn new(value: f32) -> Self {
		Self { target: value, value, buffer: [value; MAX_BUF_SIZE] }
	}

	pub fn process_block(&mut self, len: usize) {
		// Used for LFOs and such, so don't skip processing
		const FREQ: f32 = 1. / (MAX_BUF_SIZE as f32);
		for i in 0..len {
			let a = (i as f32) * FREQ;

			self.buffer[i] = lerp(self.value, self.target, a);
		}
		self.value = lerp(self.value, self.target, (len as f32) * FREQ);
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

#[derive(Debug)]
pub struct SmoothBuffer {
	value: f32,
	target: f32,
	f: f32,
	buffer: [f32; MAX_BUF_SIZE],
}

impl SmoothBuffer {
	pub fn new(value: f32, t: f32, sample_rate: f32) -> Self {
		Self {
			target: value,
			value,
			f: time_constant(t, sample_rate),
			buffer: [value; MAX_BUF_SIZE],
		}
	}
	pub fn new_direct(f: f32) -> Self {
		let v = 0.01;
		Self { target: v, value: v, f, buffer: [v; MAX_BUF_SIZE] }
	}

	pub fn process_block(&mut self, len: usize) {
		if (self.value - self.target).abs() < 1e-5 {
			for i in 0..len {
				self.buffer[i] = self.value;
			}
		} else {
			for i in 0..len {
				self.value += self.f * (self.target - self.value);

				self.buffer[i] = self.value;
			}
		}
	}

	pub fn multiply_block(&self, buf: &mut [f32]) {
		for (i, sample) in buf.iter_mut().enumerate() {
			*sample *= self.get(i);
		}
	}

	pub fn add_block(&self, buf: &mut [f32]) {
		for (i, sample) in buf.iter_mut().enumerate() {
			*sample += self.get(i);
		}
	}
	pub fn subtract_block(&self, buf: &mut [f32]) {
		for (i, sample) in buf.iter_mut().enumerate() {
			*sample -= self.get(i);
		}
	}

	pub fn lerp_block(&self, buf_a: &[f32], buf_b: &mut [f32]) {
		for (i, (a, b)) in buf_a.iter().zip(buf_b.iter_mut()).enumerate() {
			*b = lerp(*a, *b, self.get(i));
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

	#[must_use]
	pub fn get(&self, i: usize) -> f32 {
		self.buffer[i]
	}

	pub fn target(&self) -> f32 {
		self.target
	}
}
