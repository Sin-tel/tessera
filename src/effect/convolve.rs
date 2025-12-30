use crate::dsp::delayline::DelayLine;
use crate::dsp::lerp;
use crate::dsp::smooth::Smooth;
use crate::effect::*;
use crate::worker::RequestData;
use fft_convolution::Convolution;
use fft_convolution::fft_convolver::TwoStageFFTConvolver;
use std::any::Any;

#[rustfmt::skip]
const IR_PATHS: &[&str] = &[
	"ir/noise_ir3.wav",
	"ir/noise_burst_2.wav",
	"ir/noise_ir8.wav",
];

pub struct Convolve {
	balance: Smooth,
	convolver: Option<Box<[TwoStageFFTConvolver; 2]>>,
	pre_delay: [DelayLine; 2],
	pre_delay_len: Smooth,
	buffer: [[f32; MAX_BUF_SIZE]; 2],
}

impl Effect for Convolve {
	fn new(sample_rate: f32) -> Self {
		Convolve {
			balance: Smooth::new(1.0, 25.0, sample_rate),
			convolver: None,
			pre_delay: [DelayLine::new(sample_rate, 0.100), DelayLine::new(sample_rate, 0.100)],
			pre_delay_len: Smooth::new(0., 200., sample_rate),
			buffer: [[0.; MAX_BUF_SIZE]; 2],
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		if let Some(convolver) = &mut self.convolver {
			let [conv_l, conv_r] = convolver.as_mut();
			let n = buffer[0].len();
			assert!(n <= MAX_BUF_SIZE);

			conv_l.process(buffer[0], &mut self.buffer[0][..n]);
			conv_r.process(buffer[1], &mut self.buffer[1][..n]);

			for i in 0..n {
				let balance = self.balance.process();
				let pre_delay_len = self.pre_delay_len.process();

				for ch in 0..2 {
					let delayed = if pre_delay_len > 0.0 {
						self.pre_delay[ch].go_back_cubic(pre_delay_len)
					} else {
						self.buffer[ch][i]
					};
					self.pre_delay[ch].push(self.buffer[ch][i]);
					buffer[ch][i] = lerp(buffer[ch][i], delayed, balance);
				}
			}
		}
	}

	fn flush(&mut self) {
		if let Some(convolver) = &mut self.convolver {
			convolver.iter_mut().for_each(|c| c.reset());
		}
	}

	fn receive_data(&mut self, data: ResponseData) -> Option<Box<dyn Any + Send>> {
		if let ResponseData::IR(new_convolver) = data {
			let old_convolver = self.convolver.replace(new_convolver);
			old_convolver.map(|b| b as Box<dyn Any + Send>)
		} else {
			unreachable!();
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		#[allow(clippy::single_match_else)]
		match index {
			0 => self.balance.set(value),
			1 => {
				let index = (value as usize).max(1) - 1;
				match IR_PATHS.get(index) {
					Some(path) => {
						return Some(RequestData::IR(path));
					},
					None => log_warn!("Impulse index out of bounds: {index}"),
				}
			},
			2 => self.pre_delay_len.set(value / 1000.0), // value is in ms
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}

impl Convolve {}
