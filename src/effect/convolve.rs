use crate::dsp::delayline::DelayLine;
use crate::dsp::lerp;
use crate::dsp::smooth::Smooth;
use crate::effect::*;
use crate::worker::RequestData;
use fft_convolver::FFTConvolver;
use std::any::Any;
use std::iter::zip;

#[rustfmt::skip]
const IR_PATHS: &[&str] = &[
	"samples/noise_ir3.wav",
	"samples/360 Violin 1_whiten_minphase_cut.wav",
	"samples/noise_ir8l.wav",
	// "samples/noise_ir.wav",
	// "samples/noise_ir2.wav",

];

pub struct Convolve {
	balance: Smooth,
	convolver: Box<FFTConvolver<f32>>,
	pre_delay: DelayLine,
	pre_delay_len: Smooth,
}

impl Effect for Convolve {
	fn new(sample_rate: f32) -> Self {
		let convolver = Box::new(FFTConvolver::default());
		Convolve {
			balance: Smooth::new(1.0, 25.0, sample_rate),
			convolver,
			pre_delay: DelayLine::new(sample_rate, 0.100),
			pre_delay_len: Smooth::new(0., 200., sample_rate),
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		// we do mono only for now, using the left channel
		// and pass the right as the output
		let conv_result = self.convolver.process(bl, br);
		assert!(conv_result.is_ok());

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			// It's called pre_delay but we do it after anyway.
			let pre_delay_len = self.pre_delay_len.process();

			let mut wet = *r;
			if pre_delay_len > 0.0 {
				wet = self.pre_delay.go_back_cubic(pre_delay_len);
			}

			self.pre_delay.push(*r);

			let balance = self.balance.process();
			*r = lerp(*l, wet, balance);
		}

		bl.copy_from_slice(br);
	}

	fn flush(&mut self) {
		self.convolver.reset();
	}

	fn receive_data(&mut self, data: ResponseData) -> Option<Box<dyn Any + Send>> {
		if let ResponseData::IR(new_convolver) = data {
			let old_convolver = std::mem::replace(&mut self.convolver, new_convolver);
			Some(old_convolver)
		} else {
			unreachable!();
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		#[allow(clippy::single_match_else)]
		match index {
			0 => self.balance.set(value),

			1 => self.pre_delay_len.set(value / 1000.0), // value is in ms
			2 => {
				let index = value as usize - 1;
				let path = IR_PATHS[index];
				return Some(RequestData::IR(path));
			},
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}

impl Convolve {}
