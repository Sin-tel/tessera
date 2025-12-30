use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use crate::effect::*;
use crate::worker::RequestData;
use std::iter::zip;

#[derive(Debug)]
pub struct Tremolo {
	sample_rate: f32,
	accum: f32,
	amount: Smooth,
	lfo_rate: Smooth,
	phase: Smooth,
}

impl Effect for Tremolo {
	fn new(sample_rate: f32) -> Self {
		Tremolo {
			sample_rate,
			accum: 0.,
			amount: Smooth::new(0., 25.0, sample_rate),
			lfo_rate: Smooth::new(0., 25.0, sample_rate),
			phase: Smooth::new(0., 25.0, sample_rate),
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			let lfo_rate = self.lfo_rate.process();
			let phase = self.phase.process();
			let amount = self.amount.process();

			self.accum += lfo_rate / self.sample_rate;
			self.accum -= self.accum.floor();

			let gain_l = 1.0 + amount * sin_cheap(self.accum);
			let gain_r = 1.0 + amount * sin_cheap(self.accum + phase);

			*l *= gain_l;
			*r *= gain_r;
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.amount.set(value),
			1 => self.lfo_rate.set(value),
			2 => self.phase.set(value * 0.5),
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
