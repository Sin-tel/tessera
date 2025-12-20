use std::sync::atomic::{AtomicU32, Ordering};

pub struct AtomicFloat(AtomicU32);

impl AtomicFloat {
	pub const fn new() -> Self {
		return Self(AtomicU32::new(0));
	}
	pub fn load(&self) -> f32 {
		f32::from_bits(self.0.load(Ordering::Relaxed))
	}
	pub fn store(&self, value: f32) {
		self.0.store(value.to_bits(), Ordering::Relaxed);
	}
}
