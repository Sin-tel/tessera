use crate::dsp::lerp;
use bit_mask_ring_buf::BMRingBuf;

#[derive(Debug)]
pub struct DelayLine {
	buf: BMRingBuf<f32>,
	sample_rate: f32,
	pos: isize,
}

#[allow(dead_code)]
impl DelayLine {
	pub fn new(sample_rate: f32, len: f32) -> Self {
		Self {
			buf: BMRingBuf::<f32>::from_len((len * sample_rate) as usize),
			sample_rate,
			pos: 0,
		}
	}

	pub fn push(&mut self, s: f32) {
		self.pos = self.buf.constrain(self.pos + 1);
		self.buf[self.pos] = s;
	}

	pub fn go_back_int(&mut self, time: f32) -> f32 {
		let dt = (time * self.sample_rate).round() as isize;
		self.buf[self.pos - dt]
	}

	pub fn go_back_linear(&mut self, time: f32) -> f32 {
		let dt = time * self.sample_rate;
		let fpos = (self.pos as f32) - dt;
		let ipos = fpos as isize;
		let frac = fpos.fract();

		lerp(self.buf[ipos], self.buf[ipos - 1], frac)
	}
}
