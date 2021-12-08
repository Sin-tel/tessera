// can autogen this
use crate::instrument::sine::*;

pub trait Param {
	fn set_param(&mut self, index: usize, val: f32);
}

impl Param for Sine {
	fn set_param(&mut self, index: usize, value: f32) {
		match index {
			0 => self.feedback = value,
			_ => eprintln!("Parameter with index {} not found", index),
		}
	}
}
