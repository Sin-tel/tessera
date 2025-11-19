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

use bit_mask_ring_buf::BitMaskRB;

pub const COEF_19: [f32; 5] =
	[1.6543841e-04, -5.942591e-3, 3.6400687e-02, -1.3846028e-01, 6.078056e-1];

pub const COEF_31: [f32; 8] = [
	-9.926305e-5,
	1.2845506e-03,
	-5.469064e-3,
	1.604118e-2,
	-3.845655e-2,
	8.3076164e-02,
	-1.8245567e-01,
	6.261118e-1,
];

pub const COEF_51: [f32; 13] = [
	1.3438646e-06,
	-3.0550054e-05,
	1.7178609e-04,
	-6.266282e-4,
	1.7876519e-03,
	-4.317118e-3,
	9.225773e-3,
	-1.7981611e-02,
	3.2782678e-02,
	-5.7463527e-02,
	1.0096023e-01,
	-1.9530381e-01,
	6.307933e-1,
];

#[derive(Debug)]
pub struct Downsampler<const N: usize> {
	buf1: BitMaskRB<f32>,
	buf2: BitMaskRB<f32>,
	pos: isize,
	coef: [f32; N],
}

impl<const N: usize> Downsampler<N> {
	pub fn new(coef_arr: &[f32]) -> Self {
		assert_eq!(coef_arr.len(), N, "Coefficient array size mismatch");

		let mut coef = [0.0; N];
		coef[..N].copy_from_slice(&coef_arr[..N]);

		Self {
			buf1: BitMaskRB::<f32>::new(2 * N, 0.0),
			buf2: BitMaskRB::<f32>::new(2 * N, 0.0),
			pos: 0,
			coef,
		}
	}

	pub fn process(&mut self, s1: f32, s2: f32) -> f32 {
		self.pos = self.buf1.constrain(self.pos + 1);
		self.buf1[self.pos] = s1;
		self.buf2[self.pos] = s2;

		let k = N * 2 - 1;

		let s1 = self
			.coef
			.iter()
			.enumerate()
			.map(|(i, &c)| {
				c * (self.buf1[self.pos - i as isize] + self.buf1[self.pos - (k - i) as isize])
			})
			.sum::<f32>();

		let s2 = self.buf1[self.pos + 1 - (N as isize)];
		0.5 * (s1 + s2)
	}

	pub fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
		assert_eq!(output.len() * 2, input.len(), "Output must be twice the size of input.");

		for (i, chunk) in input.chunks_exact(2).enumerate() {
			output[i] = self.process(chunk[0], chunk[1]);
		}
	}
}

#[derive(Debug)]
pub struct Upsampler<const N: usize> {
	buf: BitMaskRB<f32>,
	pos: isize,
	coef: [f32; N],
}

impl<const N: usize> Upsampler<N> {
	pub fn new(coef_arr: &[f32]) -> Self {
		assert_eq!(coef_arr.len(), N, "Coefficient array size mismatch");

		let mut coef = [0.0; N];
		coef[..N].copy_from_slice(&coef_arr[..N]);
		Self { buf: BitMaskRB::<f32>::new(16, 0.0), pos: 0, coef }
	}

	pub fn process(&mut self, s: f32) -> (f32, f32) {
		self.pos = self.buf.constrain(self.pos + 1);
		self.buf[self.pos] = s;

		let k = N * 2 - 1;
		let s1 = self
			.coef
			.iter()
			.enumerate()
			.map(|(i, &c)| {
				c * (self.buf[self.pos - i as isize] + self.buf[self.pos - (k - i) as isize])
			})
			.sum::<f32>();

		let s2 = self.buf[self.pos - 7];

		(s1, s2)
	}

	pub fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
		assert_eq!(input.len() * 2, output.len(), "Input must be twice the size of output.");

		for (i, chunk) in output.chunks_exact_mut(2).enumerate() {
			let (even, odd) = self.process(input[i]);
			chunk[0] = even;
			chunk[1] = odd;
		}
	}
}
