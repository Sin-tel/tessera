// FIR 2x resamplers, windowed sinc
// coefficients obtained from fir_calc.py
// upsamplers have a gain of 2 to compensate average power
//
// 19 taps:
// kaiser window, beta = 8
// 31 taps:
// kaiser window, beta = 8
// 51 taps
// kaiser window, beta = 12

// TODO: double buffer so there is always a contiguous segment to iterate over
// TODO: simd
// see: https://jatinchowdhury18.medium.com/fast-fir-filtering-798d5d773838

use bit_mask_ring_buf::BMRingBuf;

#[derive(Debug)]
pub struct Downsampler19 {
	buf1: BMRingBuf<f32>,
	buf2: BMRingBuf<f32>,
	pos: isize,
}

#[derive(Debug)]
pub struct Downsampler31 {
	buf1: BMRingBuf<f32>,
	buf2: BMRingBuf<f32>,
	pos: isize,
}

#[derive(Debug)]
pub struct Downsampler51 {
	buf1: BMRingBuf<f32>,
	buf2: BMRingBuf<f32>,
	pos: isize,
}

#[derive(Debug)]
pub struct Upsampler19 {
	buf: BMRingBuf<f32>,
	pos: isize,
}

#[derive(Debug)]
pub struct Upsampler31 {
	buf: BMRingBuf<f32>,
	pos: isize,
}

impl Downsampler19 {
	pub fn new() -> Self {
		Self {
			buf1: BMRingBuf::<f32>::from_len(16),
			buf2: BMRingBuf::<f32>::from_len(16),
			pos: 0,
		}
	}

	#[rustfmt::skip]
	pub fn process(&mut self, s1: f32, s2: f32) -> f32 {
		self.pos = self.buf1.constrain(self.pos + 1);
		self.buf1[self.pos] = s1;
		self.buf2[self.pos] = s2;

		let mut s = 0.0;
		s += 8.2719205e-5 * (self.buf2[self.pos    ] + self.buf2[self.pos - 9]);
		s -= 2.9712955e-3 * (self.buf2[self.pos - 1] + self.buf2[self.pos - 8]);
		s += 1.8200343e-2 * (self.buf2[self.pos - 2] + self.buf2[self.pos - 7]);
		s -= 6.923014e-2  * (self.buf2[self.pos - 3] + self.buf2[self.pos - 6]);
		s += 3.039028e-1  * (self.buf2[self.pos - 4] + self.buf2[self.pos - 5]);

		s + 0.5 * self.buf1[self.pos - 4]
	}
}

impl Downsampler31 {
	pub fn new() -> Self {
		Self {
			buf1: BMRingBuf::<f32>::from_len(16),
			buf2: BMRingBuf::<f32>::from_len(16),
			pos: 0,
		}
	}

	#[rustfmt::skip]
	pub fn process(&mut self, s1: f32, s2: f32) -> f32 {
		self.pos = self.buf1.constrain(self.pos + 1);
		self.buf1[self.pos] = s1;
		self.buf2[self.pos] = s2;

		let mut s = 0.0;
		s -= 4.9631526e-5 * (self.buf2[self.pos    ] + self.buf2[self.pos - 15]);
		s += 6.422753e-4  * (self.buf2[self.pos - 1] + self.buf2[self.pos - 14]);
		s -= 2.734532e-3  * (self.buf2[self.pos - 2] + self.buf2[self.pos - 13]);
		s += 8.02059e-3   * (self.buf2[self.pos - 3] + self.buf2[self.pos - 12]);
		s -= 1.9228276e-2 * (self.buf2[self.pos - 4] + self.buf2[self.pos - 11]);
		s += 4.1538082e-2 * (self.buf2[self.pos - 5] + self.buf2[self.pos - 10]);
		s -= 9.122784e-2  * (self.buf2[self.pos - 6] + self.buf2[self.pos -  9]);
		s += 3.130559e-1  * (self.buf2[self.pos - 7] + self.buf2[self.pos -  8]);

		s + 0.5 * self.buf1[self.pos - 7]
	}
}

impl Downsampler51 {
	pub fn new() -> Self {
		Self {
			buf1: BMRingBuf::<f32>::from_len(32),
			buf2: BMRingBuf::<f32>::from_len(32),
			pos: 0,
		}
	}

	#[rustfmt::skip]
	pub fn process(&mut self, s1: f32, s2: f32) -> f32 {
		self.pos = self.buf1.constrain(self.pos + 1);
		self.buf1[self.pos] = s1;
		self.buf2[self.pos] = s2;

		let mut s = 0.0;
		s += 6.719323e-7  * (self.buf2[self.pos     ] + self.buf2[self.pos - 25]);
		s -= 1.5275027e-5 * (self.buf2[self.pos -  1] + self.buf2[self.pos - 24]);
		s += 8.589304e-5  * (self.buf2[self.pos -  2] + self.buf2[self.pos - 23]);
		s -= 3.133141e-4  * (self.buf2[self.pos -  3] + self.buf2[self.pos - 22]);
		s += 8.9382596e-4 * (self.buf2[self.pos -  4] + self.buf2[self.pos - 21]);
		s -= 2.158559e-3  * (self.buf2[self.pos -  5] + self.buf2[self.pos - 20]);
		s += 4.6128863e-3 * (self.buf2[self.pos -  6] + self.buf2[self.pos - 19]);
		s -= 8.990806e-3  * (self.buf2[self.pos -  7] + self.buf2[self.pos - 18]);
		s += 1.6391339e-2 * (self.buf2[self.pos -  8] + self.buf2[self.pos - 17]);
		s -= 2.8731763e-2 * (self.buf2[self.pos -  9] + self.buf2[self.pos - 16]);
		s += 5.0480116e-2 * (self.buf2[self.pos - 10] + self.buf2[self.pos - 15]);
		s -= 9.765191e-2  * (self.buf2[self.pos - 11] + self.buf2[self.pos - 14]);
		s += 3.1539664e-1 * (self.buf2[self.pos - 12] + self.buf2[self.pos - 13]);

		s + 0.5 * self.buf1[self.pos - 12]
	}
}

impl Upsampler19 {
	pub fn new() -> Self {
		Self {
			buf: BMRingBuf::<f32>::from_len(16),
			pos: 0,
		}
	}

	#[rustfmt::skip]
	pub fn process(&mut self, s: f32) -> (f32, f32) {
		self.pos = self.buf.constrain(self.pos + 1);
		self.buf[self.pos] = s;

		let mut s1 = 0.0;
		s1 += 1.6543841e-4 * (self.buf[self.pos    ] + self.buf[self.pos - 9]);
		s1 -= 5.942591e-3  * (self.buf[self.pos - 1] + self.buf[self.pos - 8]);
		s1 += 3.6400687e-2 * (self.buf[self.pos - 2] + self.buf[self.pos - 7]);
		s1 -= 1.3846028e-1 * (self.buf[self.pos - 3] + self.buf[self.pos - 6]);
		s1 += 6.078056e-1  * (self.buf[self.pos - 4] + self.buf[self.pos - 5]);

		let s2 = self.buf[self.pos - 4];

		(s1, s2)
	}
}

impl Upsampler31 {
	pub fn new() -> Self {
		Self {
			buf: BMRingBuf::<f32>::from_len(16),
			pos: 0,
		}
	}

	#[rustfmt::skip]
	pub fn process(&mut self, s: f32) -> (f32, f32) {
		self.pos = self.buf.constrain(self.pos + 1);
		self.buf[self.pos] = s;

		let mut s1 = 0.0;
		s1 -= 9.926305e-5  * (self.buf[self.pos    ] + self.buf[self.pos - 15]);
		s1 += 1.2845506e-3 * (self.buf[self.pos - 1] + self.buf[self.pos - 14]);
		s1 -= 5.469064e-3  * (self.buf[self.pos - 2] + self.buf[self.pos - 13]);
		s1 += 1.604118e-2  * (self.buf[self.pos - 3] + self.buf[self.pos - 12]);
		s1 -= 3.845655e-2  * (self.buf[self.pos - 4] + self.buf[self.pos - 11]);
		s1 += 8.3076164e-2 * (self.buf[self.pos - 5] + self.buf[self.pos - 10]);
		s1 -= 1.8245567e-1 * (self.buf[self.pos - 6] + self.buf[self.pos -  9]);
		s1 += 6.261118e-1  * (self.buf[self.pos - 7] + self.buf[self.pos -  8]);

		let s2 = self.buf[self.pos - 7];

		(s1, s2)
	}
}

impl Default for Downsampler19 {
	fn default() -> Self {
		Self::new()
	}
}

impl Default for Downsampler31 {
	fn default() -> Self {
		Self::new()
	}
}

impl Default for Downsampler51 {
	fn default() -> Self {
		Self::new()
	}
}

impl Default for Upsampler19 {
	fn default() -> Self {
		Self::new()
	}
}

impl Default for Upsampler31 {
	fn default() -> Self {
		Self::new()
	}
}
