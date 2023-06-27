// FIR 2x resamplers, windowed sinc
//
// 19 taps:
// kaiser window, beta = 6
// 31 taps:
// kaiser window, beta = 8

use bit_mask_ring_buf::BMRingBuf;

#[derive(Debug)]
pub struct Downsampler {
	buf: BMRingBuf<f32>,
	pos: isize,
}

impl Default for Downsampler {
	fn default() -> Self {
		Self::new()
	}
}

impl Downsampler {
	pub fn new() -> Self {
		Self {
			buf: BMRingBuf::<f32>::from_len(32),
			pos: 0,
		}
	}

	#[rustfmt::skip]
	pub fn process_19(&mut self, s1: f32, s2: f32) -> f32 {
		self.pos = self.buf.constrain(self.pos + 1);
		self.buf[self.pos] = s1;
		self.pos = self.buf.constrain(self.pos + 1);
		self.buf[self.pos] = s2;

		let mut s = 0.0;
		s += 0.0005260367 * (self.buf[self.pos    ] + self.buf[self.pos - 18]);
		s -= 0.0062802667 * (self.buf[self.pos - 2] + self.buf[self.pos - 16]);
		s += 0.025537502  * (self.buf[self.pos - 4] + self.buf[self.pos - 14]);
		s -= 0.077656515  * (self.buf[self.pos - 6] + self.buf[self.pos - 12]);
		s += 0.30770457   * (self.buf[self.pos - 8] + self.buf[self.pos - 10]);

		s + 0.5 * self.buf[self.pos - 9]
	}

	#[rustfmt::skip]
	pub fn process_31(&mut self, s1: f32, s2: f32) -> f32 {
		self.pos = self.buf.constrain(self.pos + 1);
		self.buf[self.pos] = s1;
		self.pos = self.buf.constrain(self.pos + 1);
		self.buf[self.pos] = s2;

		let mut s = 0.0;
		s -= 4.9631526e-5 * (self.buf[self.pos     ] + self.buf[self.pos - 30]);
		s += 6.422753e-4  * (self.buf[self.pos -  2] + self.buf[self.pos - 28]);
		s -= 2.734532e-3  * (self.buf[self.pos -  4] + self.buf[self.pos - 26]);
		s += 8.02059e-3   * (self.buf[self.pos -  6] + self.buf[self.pos - 24]);
		s -= 1.9228276e-2 * (self.buf[self.pos -  8] + self.buf[self.pos - 22]);
		s += 4.1538082e-2 * (self.buf[self.pos - 10] + self.buf[self.pos - 20]);
		s -= 9.122784e-2  * (self.buf[self.pos - 12] + self.buf[self.pos - 18]);
		s += 3.130559e-1  * (self.buf[self.pos - 14] + self.buf[self.pos - 16]);

		s + 0.5 * self.buf[self.pos - 15]
	}
}

#[derive(Debug)]
pub struct Upsampler {
	buf: BMRingBuf<f32>,
	pos: isize,
}

impl Default for Upsampler {
	fn default() -> Self {
		Self::new()
	}
}

impl Upsampler {
	pub fn new() -> Self {
		Self {
			buf: BMRingBuf::<f32>::from_len(32),
			pos: 0,
		}
	}

	#[rustfmt::skip]
	pub fn process_19(&mut self, s: f32) -> (f32, f32) {
		self.pos = self.buf.constrain(self.pos + 1);
		self.buf[self.pos] = s;

		let mut s1 = 0.0;
		s1 += 0.0005260367 * (self.buf[self.pos    ] + self.buf[self.pos - 9]);
		s1 -= 0.0062802667 * (self.buf[self.pos - 1] + self.buf[self.pos - 8]);
		s1 += 0.025537502  * (self.buf[self.pos - 2] + self.buf[self.pos - 7]);
		s1 -= 0.077656515  * (self.buf[self.pos - 3] + self.buf[self.pos - 6]);
		s1 += 0.30770457   * (self.buf[self.pos - 4] + self.buf[self.pos - 5]);

		let s2 = 0.5 * self.buf[self.pos - 4];

		(s1, s2)
	}

	#[rustfmt::skip]
	pub fn process_31(&mut self, s: f32) -> (f32, f32) {
		self.pos = self.buf.constrain(self.pos + 1);
		self.buf[self.pos] = s;

		let mut s1 = 0.0;
		s1 -= 4.9631526e-5 * (self.buf[self.pos    ] + self.buf[self.pos - 15]);
		s1 += 6.422753e-4  * (self.buf[self.pos - 1] + self.buf[self.pos - 14]);
		s1 -= 2.734532e-3  * (self.buf[self.pos - 2] + self.buf[self.pos - 13]);
		s1 += 8.02059e-3   * (self.buf[self.pos - 3] + self.buf[self.pos - 12]);
		s1 -= 1.9228276e-2 * (self.buf[self.pos - 4] + self.buf[self.pos - 11]);
		s1 += 4.1538082e-2 * (self.buf[self.pos - 5] + self.buf[self.pos - 10]);
		s1 -= 9.122784e-2  * (self.buf[self.pos - 6] + self.buf[self.pos -  9]);
		s1 += 3.130559e-1  * (self.buf[self.pos - 7] + self.buf[self.pos -  8]);

		let s2 = 0.5 * self.buf[self.pos - 7];

		(s1, s2)
	}
}
