use crate::dsp::smooth::Smooth;
use crate::effect::*;
use crate::worker::RequestData;
use std::iter::zip;

#[derive(Debug)]
pub struct Gain {
	gain: Smooth,
}

impl Effect for Gain {
	fn new(sample_rate: f32) -> Self {
		Gain { gain: Smooth::new(1., 25., sample_rate) }
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;
		let len = bl.len();
		assert!(len <= MAX_BUF_SIZE);

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			let gain = self.gain.process();
			*l *= gain;
			*r *= gain;
		}
	}
	fn flush(&mut self) {}
	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		#[allow(clippy::single_match_else)]
		match index {
			0 => self.gain.set(value),
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
