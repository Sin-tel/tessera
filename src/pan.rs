use crate::defs::*;
use crate::dsp::delayline::DelayLine;
use crate::dsp::simper::Filter;
use crate::dsp::Smoothed;

// interaural time difference, 660 μs
const ITD: f32 = 0.00066;
// head filter at 4 kHz
const HEAD_CUTOFF: f32 = 4000.0;
const HEAD_Q: f32 = 0.4;

#[derive(Debug)]
pub struct Pan {
	gain: Smoothed<Stereo>,
	pan: Smoothed<Mono>,
	lfilter: Filter,
	rfilter: Filter,
	delayline: DelayLine<Stereo>,
}

impl Pan {
	pub fn new(sample_rate: f32) -> Pan {
		let mut lfilter = Filter::new(sample_rate);
		lfilter.set_highshelf(HEAD_CUTOFF, HEAD_Q, 0.0);
		let mut rfilter = Filter::new(sample_rate);
		rfilter.set_highshelf(HEAD_CUTOFF, HEAD_Q, 0.0);
		Pan {
			gain: Smoothed::<Stereo>::new(Stereo([1.0; 2]), 100.0, sample_rate),
			pan: Smoothed::new(Mono(0.0), 100.0, sample_rate),
			lfilter,
			rfilter,
			delayline: DelayLine::<Stereo>::new(sample_rate, ITD),
		}
	}

	pub fn set(&mut self, gain: f32, pan: f32) {
		self.pan.set(Mono(pan));

		let lshelf = -1.5 * pan * (pan + 3.0);
		let rshelf = -1.5 * pan * (pan - 3.0);
		self.lfilter.set_highshelf(HEAD_CUTOFF, HEAD_Q, lshelf);
		self.rfilter.set_highshelf(HEAD_CUTOFF, HEAD_Q, rshelf);

		let lgain = -0.084 * pan * (pan + 2.53) + 1.0;
		let rgain = -0.084 * pan * (pan - 2.53) + 1.0;
		self.gain.set(Stereo([lgain, rgain]) * gain);
	}

	pub fn process(&mut self, buffer: &mut [Stereo]) {
		for sample in buffer.iter_mut() {
			self.gain.update();
			self.pan.update();

			let input = sample.clone();

			let p = self.pan.value;

			// delay
			let mut l = self.ldelayline.go_back_cubic((ITD * p).max(0.0));
			let mut r = self.rdelayline.go_back_cubic((-ITD * p).max(0.0));

			// head shadow filter
			l = self.lfilter.process(l);
			r = self.rfilter.process(r);

			// volume difference
			l *= self.lgain.value;
			r *= self.rgain.value;

			*sample = Stereo([l, r]);

			self.ldelayline.push(input.0[0]);
			self.rdelayline.push(input.0[1]);
		}
	}
}
