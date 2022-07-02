// can autogen this
use crate::instrument::sine::*;
use crate::instrument::*;

// list of instruments
pub fn new_instrument(sample_rate: f32, index: usize) -> Box<dyn Instrument + Send>  {
	match index {
		0 => Box::new(Sine::new(sample_rate)),
		_ => {
			eprintln!("Instrument with index {} not found. Returning default.", index);
			Box::new(Sine::new(sample_rate))
		}
	}
}

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
