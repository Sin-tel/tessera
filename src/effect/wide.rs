use crate::dsp::delayline::DelayLine;
use crate::dsp::simper::Filter;
use crate::effect::*;
use std::iter::zip;

// 3.2 ms
const MAX_DELAY: f32 = 0.0032;

#[derive(Debug)]
pub struct Wide {
	amount: f32,

	delay: DelayLine,

	ap_1: Filter,
	ap_2: Filter,
}

impl Effect for Wide {
	fn new(sample_rate: f32) -> Self {
		let mut ap_1 = Filter::new(sample_rate);
		let mut ap_2 = Filter::new(sample_rate);

		ap_1.set_allpass(50.0, 0.5);
		ap_2.set_allpass(300.0, 0.5);

		Wide { amount: 0.0, delay: DelayLine::new(sample_rate, MAX_DELAY), ap_1, ap_2 }
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			let mut s = self.delay.go_back_cubic(MAX_DELAY * self.amount);

			let s_in = 0.5 * (*l + *r);
			self.delay.push(s_in);

			s = self.ap_1.process(s);
			s = self.ap_2.process(s);

			*l += self.amount * s;
			*r -= self.amount * s;
		}
	}
	fn flush(&mut self) {}
	fn set_parameter(&mut self, index: usize, value: f32) {
		#[allow(clippy::single_match_else)]
		match index {
			0 => self.amount = value,
			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}
