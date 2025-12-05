use fft_convolver::FFTConvolver;
use std::iter::zip;

use crate::audio::MAX_BUF_SIZE;
use crate::dsp::lerp;
use crate::effect::*;

#[derive(Debug)]
pub struct Convolve {
	balance: f32,
	convolver: FFTConvolver<f32>,
}

impl Effect for Convolve {
	fn new(_sample_rate: f32) -> Self {
		// TODO: abstract out wav handling into seperate module
		// TODO: share resources (lazy_static?). we dont need to reload the file for every instance.
		//       can also just include_bytes and not deal with any resource loading

		// let reader = hound::WavReader::open("assets/samples/noise_ir.wav").unwrap();
		// let reader = hound::WavReader::open("assets/samples/noise_ir2.wav").unwrap();
		let reader = hound::WavReader::open("assets/samples/noise_ir3.wav").unwrap();

		let spec = reader.spec();

		// TODO: handle more cases
		// TODO: resample to get sample rate independence
		assert!(spec.sample_rate == 44100);
		assert!(spec.bits_per_sample == 16);
		assert!(spec.sample_format == hound::SampleFormat::Int);

		let mut impulse_response: Vec<f32> = reader
			.into_samples::<i16>()
			.step_by(spec.channels.into()) // stereo samples are interleaved
			.map(|s| f32::from(s.unwrap()) / f32::from(i16::MAX))
			.collect();

		// for now we only allow short convolution samples
		assert!(impulse_response.len() < 2048);

		let sqr_sum = impulse_response.iter().fold(0.0, |sqr_sum, s| sqr_sum + s * s);

		let gain = 1.0 / sqr_sum.sqrt();
		for s in &mut impulse_response {
			*s *= gain;
		}

		let mut convolver = FFTConvolver::default();
		let init_result = convolver.init(MAX_BUF_SIZE, &impulse_response);
		assert!(init_result.is_ok());

		Convolve { balance: 1.0, convolver }
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

	fn flush(&mut self) {}

	fn set_parameter(&mut self, index: usize, value: f32) {
		#[allow(clippy::single_match_else)]
		match index {
			0 => self.balance = value,
			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}
