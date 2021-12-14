use crate::defs::*;
use crate::math::*;

#[derive(Debug, Default)]
pub struct Pan {
	gainl: Smoothed,
	gainr: Smoothed,
}

impl Pan {
	pub fn new(sample_rate: f32) -> Pan {
		Pan {
			gainl: Smoothed::new(0.25, 200.0 / sample_rate),
			gainr: Smoothed::new(0.25, 200.0 / sample_rate),
		}
	}

	pub fn set(&mut self, gain: f32, pan: f32) {
		self.gainl.set(gain * (0.5 - 0.5 * pan).sqrt());
		self.gainr.set(gain * (0.5 + 0.5 * pan).sqrt());
	}

	pub fn process(&mut self, buffer: &mut [StereoSample]) {
		for sample in buffer.iter_mut() {
			self.gainl.update();
			self.gainr.update();
			sample.l *= self.gainl.value;
			sample.r *= self.gainr.value;
		}
	}
}
