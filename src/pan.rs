use crate::defs::*;
use crate::dsp::delayline::*;
use crate::dsp::simper::*;
use crate::dsp::*;

// inter aural delay
const IAD: f32 = 0.00066;
// const IAD: f32 = 0.1;
const HEAD_CUTOFF: f32 = 4000.0;
const HEAD_Q: f32 = 0.4;

#[derive(Debug)]
pub struct Pan {
	lgain: Smoothed,
	rgain: Smoothed,
	pan: Smoothed,
	lfilter: Filter,
	rfilter: Filter,
	ldelayline: DelayLine,
	rdelayline: DelayLine,
}

impl Pan {
	pub fn new(sample_rate: f32) -> Pan {
		let mut lfilter = Filter::new(sample_rate);
		lfilter.set_highshelf(HEAD_CUTOFF, HEAD_Q, 0.0);
		let mut rfilter = Filter::new(sample_rate);
		rfilter.set_highshelf(HEAD_CUTOFF, HEAD_Q, 0.0);
		Pan {
			lgain: Smoothed::new(1.0, 100.0 / sample_rate),
			rgain: Smoothed::new(1.0, 100.0 / sample_rate),
			pan: Smoothed::new(0.0, 100.0 / sample_rate),
			lfilter,
			rfilter,
			ldelayline: DelayLine::new(sample_rate, IAD),
			rdelayline: DelayLine::new(sample_rate, IAD),
		}
	}

	pub fn set(&mut self, gain: f32, pan: f32) {
		self.pan.set(pan);

		let lshelf = -1.5 * pan * (pan + 3.0);
		let rshelf = -1.5 * pan * (pan - 3.0);
		self.lfilter.set_highshelf(HEAD_CUTOFF, HEAD_Q, lshelf);
		self.rfilter.set_highshelf(HEAD_CUTOFF, HEAD_Q, rshelf);

		let lgain = -0.084 * pan * (pan + 2.53) + 1.0;
		let rgain = -0.084 * pan * (pan - 2.53) + 1.0;
		self.lgain.set(lgain * gain);
		self.rgain.set(rgain * gain);
	}

	pub fn process(&mut self, buffer: &mut [StereoSample]) {
		for sample in buffer.iter_mut() {
			self.lgain.update();
			self.rgain.update();
			self.pan.update();

			let l_in = sample.l;
			let r_in = sample.r;

			let p = self.pan.value;

			// delay
			let mut l = self.ldelayline.go_back_cubic((IAD * p).max(0.0));
			let mut r = self.rdelayline.go_back_cubic((-IAD * p).max(0.0));

			// head shadow filter
			l = self.lfilter.process(l);
			r = self.rfilter.process(r);

			// volume difference
			l *= self.lgain.value;
			r *= self.rgain.value;

			sample.l = l;
			sample.r = r;

			self.ldelayline.push(l_in);
			self.rdelayline.push(r_in);
		}
	}
}
