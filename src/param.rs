// can autogen this
use crate::instrument::sine::Sine;
use crate::instrument::Instrument;

// list of instruments
pub fn new_instrument(sample_rate: f32, index: usize) -> Box<dyn Instrument + Send> {
	Box::new(match index {
		0 => Sine::new(sample_rate),
		_ => {
			eprintln!(
				"Instrument with index {} not found. Returning default.",
				index
			);
			Sine::new(sample_rate)
		}
	})
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
