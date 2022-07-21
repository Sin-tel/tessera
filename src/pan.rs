// use crate::defs::*;
use crate::dsp::delayline::DelayLine;
use crate::dsp::simper::Filter;
use crate::dsp::Smoothed;

// interaural time difference, 660 μs
const ITD: f32 = 0.00066;
// head filter at 4 kHz
const HEAD_CUTOFF: f32 = 4000.0;
const HEAD_Q: f32 = 0.4;

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
		Track {
			gain: Smoothed::new(1.0, 100.0, sample_rate),
			delay: Smoothed::new(0.0, 100.0, sample_rate),
			filter,
			delayline: DelayLine::new(sample_rate, ITD),
		}
	}
}

#[derive(Debug)]
pub struct Pan {
	tracks: [Track; 2],
}

impl Pan {
	pub fn new(sample_rate: f32) -> Self {
		Pan {
			tracks: [Track::new(sample_rate), Track::new(sample_rate)],
		}
	}

	pub fn set(&mut self, gain: f32, pan: f32) {
		self.tracks[0].delay.set((ITD * pan).max(0.0));
		self.tracks[1].delay.set((-ITD * pan).max(0.0));

		let lshelf = -1.5 * pan * (pan + 3.0);
		let rshelf = -1.5 * pan * (pan - 3.0);
		self.tracks[0]
			.filter
			.set_highshelf(HEAD_CUTOFF, HEAD_Q, lshelf);
		self.tracks[1]
			.filter
			.set_highshelf(HEAD_CUTOFF, HEAD_Q, rshelf);

		let lgain = -0.084 * pan * (pan + 2.53) + 1.0;
		let rgain = -0.084 * pan * (pan - 2.53) + 1.0;
		self.tracks[0].gain.set(lgain * gain);
		self.tracks[1].gain.set(rgain * gain);
	}

	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			for sample in s.iter_mut() {
				track.gain.update();
				track.delay.update();

				let input = sample.clone();

				// delay
				let mut s = track.delayline.go_back_cubic(track.delay.value);
				// head shadow filter
				s = track.filter.process(s);
				// volume difference
				s *= track.gain.value;

				*sample = s;

				track.delayline.push(input);
			}
		}
	}
}
