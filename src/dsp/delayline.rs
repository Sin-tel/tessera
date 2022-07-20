use crate::defs::*;

use crate::dsp::lerp;
use bit_mask_ring_buf::BMRingBuf;

#[derive(Debug)]
pub struct DelayLine<T>
where
	T: Sample,
{
	buf: BMRingBuf<T>,
	sample_rate: f32,
	pos: isize,
	h: [f32; 4],
}

impl<T: Sample> DelayLine<T> {
	pub fn new(sample_rate: f32, len: f32) -> Self {
		Self {
			buf: BMRingBuf::<T>::from_len((len * sample_rate) as usize),
			sample_rate,
			pos: 0,
			h: [0.0; 4],
		}
	}

	// pub fn new_from_usize(sample_rate: f32, len: usize) -> Self {
	// 	Self {
	// 		buf: BMRingBuf::<f32>::from_len(len),
	// 		sample_rate,
	// 		pos: 0,
	// 		h: [0.0; 4],
	// 	}
	// }

	pub fn push(&mut self, s: T) {
		self.pos = self.buf.constrain(self.pos + 1);
		self.buf[self.pos] = s;
	}

	pub fn go_back_int(&mut self, time: f32) -> T {
		let dt = (time * self.sample_rate).round() as isize;
		self.buf[self.pos - dt]
	}

	// pub fn go_back_int_s(&mut self, samples: isize) -> f32 {
	// 	self.buf[self.pos - samples]
	// }

	pub fn go_back_linear(&mut self, time: f32) -> T {
		let dt = time * self.sample_rate;
		let idt = dt as isize;
		let frac = dt.fract();

		lerp(self.buf[self.pos - idt], self.buf[self.pos - idt - 1], frac)
	}

	fn calc_coeff(&mut self, delay: f32) -> isize {
		let dm1 = delay.fract();
		let d = dm1 + 1.;
		let dm2 = dm1 - 1.;
		let dm3 = dm1 - 2.;
		self.h[0] = (-1. / 6.) * dm1 * dm2 * dm3;
		self.h[1] = 0.5 * d * dm2 * dm3;
		self.h[2] = -0.5 * d * dm1 * dm3;
		self.h[3] = (1. / 6.) * d * dm1 * dm2;

		delay as isize
	}

	pub fn go_back_cubic(&mut self, time: f32) -> T {
		let dt = time * self.sample_rate;
		let idt = self.calc_coeff(dt);

		let mut sum: T = Default::default();

		for (i, h) in self.h.iter().enumerate() {
			sum += self.buf[self.pos - idt - (i as isize)] * (*h);
		}

		sum
	}
}
