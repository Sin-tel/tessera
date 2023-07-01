use crate::dsp::delayline::DelayLine;
use crate::dsp::env::Smoothed;
use crate::dsp::simper::Filter;
use crate::effect::Effect;

// TODO: This device is used everywhere and
//       most of the time, parameters don't change,
//       so we should be able to improve performance a lot

// interaural time difference, 660 μs
const ITD: f32 = 0.00066;
// head filter at 4 kHz
const HEAD_CUTOFF: f32 = 4000.0;
const HEAD_Q: f32 = 0.4;

#[derive(Debug)]
pub struct Pan {
	tracks: [Track; 2],
	gain: f32,
	pan: f32,
}

#[derive(Debug)]
struct Track {
	gain: Smoothed,
	delay: Smoothed,
	filter: Filter,
	delayline: DelayLine,
}

impl Track {
	pub fn new(sample_rate: f32) -> Self {
		let mut filter = Filter::new(sample_rate);
		filter.set_highshelf(HEAD_CUTOFF, HEAD_Q, 0.0);
		let mut gain = Smoothed::new(10.0, sample_rate);
		gain.set_immediate(1.0);
		Track {
			gain,
			delay: Smoothed::new(10.0, sample_rate),
			filter,
			delayline: DelayLine::new(sample_rate, ITD),
		}
	}
}

impl Pan {
	fn update_params(&mut self) {
		self.tracks[0].delay.set((ITD * self.pan).max(0.0));
		self.tracks[1].delay.set((-ITD * self.pan).max(0.0));

		let lshelf = -1.5 * self.pan * (self.pan + 3.0);
		let rshelf = -1.5 * self.pan * (self.pan - 3.0);
		self.tracks[0]
			.filter
			.set_highshelf(HEAD_CUTOFF, HEAD_Q, lshelf);
		self.tracks[1]
			.filter
			.set_highshelf(HEAD_CUTOFF, HEAD_Q, rshelf);

		let lgain = -0.084 * self.pan * (self.pan + 2.53) + 1.0;
		let rgain = -0.084 * self.pan * (self.pan - 2.53) + 1.0;
		self.tracks[0].gain.set(lgain * self.gain);
		self.tracks[1].gain.set(rgain * self.gain);
	}
}

impl Effect for Pan {
	fn new(sample_rate: f32) -> Self {
		Pan {
			tracks: [Track::new(sample_rate), Track::new(sample_rate)],
			gain: 1.0,
			pan: 0.0,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		for track in self.tracks.iter_mut() {
			if track.delay.get() < 1e-5 && track.delay.inner() == 0. {
				track.delay.set_immediate(0.);
			}
		}

		for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			for sample in s.iter_mut() {
				let gain = track.gain.process();
				let delay = track.delay.process();
				let input = *sample;
				track.delayline.push(input);

				// delay
				let mut s = track.delayline.go_back_cubic(delay);
				// head shadow filter
				s = track.filter.process(s);
				// volume difference
				s *= gain;

				*sample = s;
			}
		}
	}
	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => {
				self.gain = value;
				self.update_params();
			}
			1 => {
				self.pan = value;
				self.update_params();
			}
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}
