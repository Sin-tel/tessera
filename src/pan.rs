use crate::defs::*;

#[derive(Debug, Default)]
pub struct Pan {
	gain: f32,
	pan: f32,
}

impl Pan {
	pub fn new() -> Pan {
		Pan {
			gain: 1.0,
			pan: 0.0,
		}
	}

	pub fn set(&mut self, gain: f32, pan: f32) {
		self.gain = gain;
		self.pan = pan;
	}

	pub fn process(&mut self, buffer: &mut [StereoSample]) {
		let gl: f32 = self.gain * (0.5 - 0.5 * self.pan);
		let gr: f32 = self.gain * (0.5 + 0.5 * self.pan);
		for sample in buffer.iter_mut() {
			sample.l *= gl;
			sample.r *= gr;
		}
	}
}
