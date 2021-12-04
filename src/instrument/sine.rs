use crate::instrument::*;
use crate::math::*;

pub struct Sine {
	accum: f32,
	freq: f32,
	vol: f32,
	vol_: f32,
	sample_rate: f32,
}

impl Instrument for Sine {
	fn new(sample_rate: f32) -> Sine {
		Sine {
			accum: 0.0f32,
			freq: 440.0f32 / sample_rate,
			vol: 0.1f32,
			vol_: 0.1f32,
			sample_rate,
		}
	}

	fn cv(&mut self, pitch: f32, vol: f32) {
		self.vol_ = vol;
		self.freq = pitch_to_f(pitch, self.sample_rate);
	}

	fn process(&mut self, buffer: &mut [StereoSample]) {
		for sample in buffer.iter_mut() {
			self.vol += (self.vol_ - self.vol) * 0.001;
			self.accum += self.freq;
			self.accum = self.accum.fract();
			let mut out = (self.accum * TWO_PI).sin();

			// let mut out = accum;
			out *= self.vol;
			sample.l = out;
			sample.r = out;
		}
	}

	fn set_param(&mut self, index: usize, val: f32) {}
}
