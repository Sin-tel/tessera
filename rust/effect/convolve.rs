use fft_convolver::FFTConvolver;
use hound;
use std::iter::zip;

use crate::audio::MAX_BUF_SIZE;
use crate::dsp::lerp;
use crate::effect::Effect;

#[derive(Debug)]
pub struct Convolve {
	balance: f32,
	convolver: FFTConvolver,
}

impl Effect for Convolve {
	fn new(_sample_rate: f32) -> Self {
		// TODO: abstract out wav handling into seperate module
		// TODO: we can use some kind of shared resource?
		let reader = hound::WavReader::open("res/noise_burst_3.wav").unwrap();
		let spec = reader.spec();

		// TODO: handle more cases
		assert!(spec.channels == 2);
		assert!(spec.sample_rate == 44100);
		assert!(spec.bits_per_sample == 16);
		assert!(spec.sample_format == hound::SampleFormat::Int);

		let mut impulse_response: Vec<f32> = reader
			.into_samples::<i16>()
			.step_by(2) // stereo samples are interleaved
			.map(|s| s.unwrap() as f32 / f32::from(i16::MAX))
			.collect();

		// for now we only allow short convolution samples
		assert!(impulse_response.len() < 2048);

		let sqr_sum = impulse_response
			.iter()
			.fold(0.0, |sqr_sum, s| sqr_sum + s * s);

		let gain = 1.0 / sqr_sum.sqrt();
		for s in impulse_response.iter_mut() {
			*s = *s * gain
		}

		let mut convolver = FFTConvolver::default();
		let init_result = convolver.init(MAX_BUF_SIZE, &impulse_response);
		assert!(init_result.is_ok());

		Convolve {
			balance: 1.0,
			convolver,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		// we do mono only for now, using the left channel
		// and pass the right as the output
		let conv_result = self.convolver.process(bl, br);
		assert!(conv_result.is_ok());

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			*r = lerp(*l, *r, self.balance);
		}

		bl.copy_from_slice(br);
	}
	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.balance = value,
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}
