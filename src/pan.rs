use crate::defs::*;
use crate::dsp::*;
use crate::dsp::simper::*;
use crate::dsp::delayline::*;

// inter aural delay
const IAD: f32 = 0.00066;

#[derive(Debug)]
pub struct Pan {
	gain: Smoothed,
	pan: Smoothed,
	lfilter: Filter,
	rfilter: Filter,
	ldelayline: DelayLine,
	rdelayline: DelayLine,
	ldelay: f32,
	rdelay: f32,
}

impl Pan {
	pub fn new(sample_rate: f32) -> Pan {
		let mut lfilter = Filter::new(sample_rate);
		lfilter.set_highshelf(4000.0, 0.4, 0.0);
		let mut rfilter = Filter::new(sample_rate);
		rfilter.set_highshelf(4000.0, 0.4, 0.0);
		Pan {
			gain: Smoothed::new(1.0, 200.0 / sample_rate),
			pan:  Smoothed::new(0.0, 200.0 / sample_rate),
			lfilter,
			rfilter,
			ldelayline: DelayLine::new(sample_rate, IAD),
			rdelayline: DelayLine::new(sample_rate, IAD),
			ldelay: 0.0,
			rdelay: 0.0,
		}
	}

	pub fn set(&mut self, gain: f32, pan: f32) {
		self.gain.set(gain);
		self.pan.set(pan);

		let p = self.pan.value;
		let lgain = -1.5*p*(p+3.0);
		let rgain = -1.5*p*(p-3.0);
		self.lfilter.set_highshelf(4000.0, 0.4, lgain);
		self.rfilter.set_highshelf(4000.0, 0.4, rgain);

		if p < 0.0 {
			self.ldelay = 0.0;
			self.rdelay = IAD * (-p* std::f32::consts::FRAC_PI_2).sin();
		}else {
			self.ldelay = IAD * (p* std::f32::consts::FRAC_PI_2).sin();
			self.rdelay = 0.0;
		}
	}

	pub fn process(&mut self, buffer: &mut [StereoSample]) {
		for sample in buffer.iter_mut() {
			self.gain.update();
			self.pan.update();

			let l_in = sample.l;
			let r_in = sample.r;

			let p = self.pan.value;
			let lgain = from_db(-p*(p+2.0));
			let rgain = from_db(-p*(p-2.0));

			let mut l = self.ldelayline.go_back_int(self.ldelay);
			let mut r = self.rdelayline.go_back_int(self.rdelay);

			l = self.lfilter.process(l);
			r = self.rfilter.process(r);

			sample.l = l*self.gain.value*lgain;
			sample.r = r*self.gain.value*rgain;

			self.ldelayline.push(l_in);
			self.rdelayline.push(r_in);
		}
	}
}
