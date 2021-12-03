use crate::instrument::*;

pub struct Sine {
	accum: f32,
	freq: f32,
	vol: f32,
	vol_: f32,
}

impl Sine {
	pub fn new(sample_rate: f32) -> Sine {
		Sine {
			accum: 0.0f32,
			freq: 440.0f32 / sample_rate,
			vol: 0.0f32,
			vol_: 0.0f32,
		}
	}
}

impl Instrument for Sine {
	fn cv(&mut self, freq: f32, vol: f32) {
		self.vol_ = vol;
		self.freq = freq;
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
}
