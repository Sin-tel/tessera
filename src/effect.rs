// use crate::defs::*;
use crate::device::*;

pub mod gain;

pub trait Effect: Param {
	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn process(&mut self, buffer: &mut [&mut [f32]; 2]);
}

pub struct BypassEffect {
	pub effect: Box<dyn Effect + Send>,
	pub bypassed: bool,
}

impl BypassEffect {
	pub fn new(sample_rate: f32, index: usize) -> Self {
		BypassEffect {
			effect: new_effect(sample_rate, index),
			bypassed: false,
		}
	}
	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		if !self.bypassed {
			self.effect.process(buffer);
		}
	}
}
