use crate::dsp::*;
use bit_mask_ring_buf::BMRingBuf;

// TODO: we probably want to avoid the multiply by sample_rate in every call
//       and have the caller take care of that

#[derive(Debug)]
pub struct DelayLine {
	buf: BMRingBuf<f32>,
	sample_rate: f32,
	pos: isize,
	h: [f32; 4],
	time_prev: f32,
}

impl DelayLine {
	pub fn new(sample_rate: f32, len: f32) -> Self {
		Self {
			// + 4 is so that we have a bit of room for doing cubic interpolation etc.
			buf: BMRingBuf::<f32>::from_len((len * sample_rate) as usize + 4),
			sample_rate,
			pos: 0,
			h: [0.0; 4],
			time_prev: 0.,
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

	pub fn push(&mut self, s: f32) {
		self.pos += 1;
		*self.buf.at_mut(&mut self.pos) = s;
	}

	#[must_use]
	pub fn go_back_int(&self, time: f32) -> f32 {
		let d_int = (time * self.sample_rate).floor() as isize;
		self.buf[self.pos - d_int]
	}

	#[must_use]
	pub fn allpass(&mut self, s: f32, k_ap: f32, time: f32) -> f32 {
		let d = self.go_back_int(time);
		let v = s - k_ap * d;
		self.push(v);
		k_ap * v + d
	}

	// pub fn go_back_int_s(&mut self, samples: isize) -> f32 {
	// 	self.buf[self.pos - samples]
	// }

	#[must_use]
	pub fn go_back_linear(&self, time: f32) -> f32 {
		let delay = time * self.sample_rate;
		let (d_int, frac) = make_isize_frac(delay);

		lerp(
			self.buf[self.pos - d_int],
			self.buf[self.pos - d_int - 1],
			frac,
		)
	}

	// lagrange polynomial
	fn calc_coeff(&mut self, dm1: f32) {
		let d = dm1 + 1.;
		let dm2 = dm1 - 1.;
		let dm3 = dm1 - 2.;
		self.h[0] = (-1. / 6.) * dm1 * dm2 * dm3;
		self.h[1] = 0.5 * d * dm2 * dm3;
		self.h[2] = -0.5 * d * dm1 * dm3;
		self.h[3] = (1. / 6.) * d * dm1 * dm2;
	}

	#[must_use]
	pub fn go_back_cubic(&mut self, time: f32) -> f32 {
		let delay = time * self.sample_rate;
		let (d_int, dm1) = make_isize_frac(delay);

		// self.calc_coeff(dm1);

		// TODO: check if this actually improves performance
		//       maybe we should have static hint
		if self.time_prev != dm1 {
			self.calc_coeff(dm1);
		}
		self.time_prev = dm1;

		let mut sum = 0.0f32;
		for (i, h) in self.h.iter().enumerate() {
			sum += self.buf[self.pos - d_int - (i as isize)] * h;
		}

		sum
	}
}
