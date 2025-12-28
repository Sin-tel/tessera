use crate::dsp::*;
use bit_mask_ring_buf::BitMaskRB;

// TODO: we probably want to avoid the multiply by sample_rate in every call
//       and have the caller take care of that

#[derive(Debug)]
pub struct DelayLine {
	buf: BitMaskRB<f32>,
	sample_rate: f32,
	pos: isize,
}

impl DelayLine {
	pub fn new(sample_rate: f32, len: f32) -> Self {
		Self {
			// + 4 is so that we have a bit of room for doing cubic interpolation etc.
			buf: BitMaskRB::<f32>::new((len * sample_rate) as usize + 4, 0.0),
			sample_rate,
			pos: 0,
		}
	}

	pub fn new_samples(sample_rate: f32, len: usize) -> Self {
		Self { buf: BitMaskRB::<f32>::new(len, 0.0), sample_rate, pos: 0 }
	}

	pub fn push(&mut self, s: f32) {
		*self.buf.get_mut(self.pos) = s;
		self.pos = self.buf.constrain(self.pos - 1);
	}

	#[must_use]
	pub fn go_back_int(&self, time: f32) -> f32 {
		let delay = (time * self.sample_rate).max(1.);
		assert!(delay < self.buf.len().get() as f32);
		let d_int = delay.floor() as isize;

		self.buf[self.pos + d_int]
	}

	#[must_use]
	pub fn go_back_int_s(&mut self, samples: isize) -> f32 {
		self.buf[self.pos + samples]
	}

	#[must_use]
	pub fn allpass(&mut self, s: f32, k_ap: f32, time: f32) -> f32 {
		let d = self.go_back_int(time);
		let v = s - k_ap * d;
		self.push(v);
		k_ap * v + d
	}

	#[must_use]
	pub fn go_back_linear(&self, time: f32) -> f32 {
		let delay = (time * self.sample_rate).max(1.);
		assert!(delay < self.buf.len().get() as f32);

		let (d_int, frac) = make_isize_frac(delay);
		let read = self.pos + d_int;

		lerp(self.buf[read], self.buf[read + 1], frac)
	}

	#[must_use]
	pub fn go_back_cubic(&mut self, time: f32) -> f32 {
		let delay = (time * self.sample_rate).max(2.);
		assert!(delay < self.buf.len().get() as f32);

		let (d_int, frac) = make_isize_frac(delay);
		let read = self.pos + d_int;

		let y0 = self.buf[read - 1];
		let y1 = self.buf[read];
		let y2 = self.buf[read + 1];
		let y3 = self.buf[read + 2];

		lagrange4(y0, y1, y2, y3, frac)
	}

	pub fn flush(&mut self) {
		for k in self.buf.raw_data_mut() {
			*k = 0.0;
		}
	}
}
