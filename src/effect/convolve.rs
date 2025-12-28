use crate::dsp::delayline::DelayLine;
use crate::dsp::lerp;
use crate::dsp::smooth::Smooth;
use crate::effect::*;
use crate::worker::RequestData;
use fft_convolver::FFTConvolver;
use std::any::Any;

#[rustfmt::skip]
const IR_PATHS: &[&str] = &[
	"ir/noise_ir3.wav",
	"ir/violin1_whiten_minphase_cut.wav",
	"ir/noise_ir8.wav",
	// "ir/noise_ir.wav",
	// "ir/noise_ir2.wav",

];

pub struct Convolve {
	balance: Smooth,
	convolver: Box<[FFTConvolver<f32>; 2]>,
	pre_delay: [DelayLine; 2],
	pre_delay_len: Smooth,
	buffer: [[f32; MAX_BUF_SIZE]; 2],
}

impl Effect for Convolve {
	fn new(sample_rate: f32) -> Self {
		let convolver = Box::new([FFTConvolver::default(), FFTConvolver::default()]);
		Convolve {
			balance: Smooth::new(1.0, 25.0, sample_rate),
			convolver,
			pre_delay: [DelayLine::new(sample_rate, 0.100), DelayLine::new(sample_rate, 0.100)],
			pre_delay_len: Smooth::new(0., 200., sample_rate),
			buffer: [[0.; MAX_BUF_SIZE]; 2],
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [conv_l, conv_r] = &mut *self.convolver;
		let n = buffer[0].len();
		assert!(n <= MAX_BUF_SIZE);

		conv_l.process(buffer[0], &mut self.buffer[0][..n]).unwrap();
		conv_r.process(buffer[1], &mut self.buffer[1][..n]).unwrap();

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

	fn flush(&mut self) {
		self.convolver.iter_mut().for_each(|c| c.reset());
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
