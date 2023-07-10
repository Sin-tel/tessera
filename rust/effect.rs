use crate::device::*;

pub mod delay;
pub mod drive;
pub mod gain;
pub mod pan;

pub trait Effect {
	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn process(&mut self, buffer: &mut [&mut [f32]; 2]);
	fn set_parameter(&mut self, index: usize, val: f32);
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
