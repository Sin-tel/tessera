use crate::effect::*;

#[derive(Debug)]
pub struct Gain {
	gain: f32,
}

impl Effect for Gain {
	fn new(_sample_rate: f32) -> Self {
		Gain { gain: 1.0 }
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		for b in buffer.iter_mut() {
			for s in b.iter_mut() {
				*s *= self.gain;
			}
		}
	}
	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.gain = value,
			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}
