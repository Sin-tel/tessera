use crate::dsp::delayline::DelayLine;
use crate::dsp::simper::Filter;
use crate::dsp::smooth::SmoothLinear;
use crate::effect::*;

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
	gain: SmoothLinear,
	delay: SmoothLinear,
	filter: Filter,
	delayline: DelayLine,
}

impl Track {
	pub fn new(sample_rate: f32) -> Self {
		let mut filter = Filter::new(sample_rate);
		filter.set_highshelf(HEAD_CUTOFF, HEAD_Q, 0.0);
		let mut gain = SmoothLinear::new(30.0, sample_rate);
		gain.set_immediate(1.0);
		Track {
			gain,
			delay: SmoothLinear::new(30.0, sample_rate),
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
		self.tracks[0].filter.set_highshelf(HEAD_CUTOFF, HEAD_Q, lshelf);
		self.tracks[1].filter.set_highshelf(HEAD_CUTOFF, HEAD_Q, rshelf);

		let lgain = -0.084 * self.pan * (self.pan + 2.53) + 1.0;
		let rgain = -0.084 * self.pan * (self.pan - 2.53) + 1.0;
		self.tracks[0].gain.set(lgain * self.gain);
		self.tracks[1].gain.set(rgain * self.gain);
	}
}

impl Effect for Pan {
	fn new(sample_rate: f32) -> Self {
		Pan { tracks: [Track::new(sample_rate), Track::new(sample_rate)], gain: 1.0, pan: 0.0 }
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		// if self.pan == 0. {
		// 	for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
		// 		for sample in buf.iter_mut() {
		// 			let gain = track.gain.process();
		// 			*sample *= gain;
		// 		}
		// 	}
		// } else {
		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			for sample in buf.iter_mut() {
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
		// }
	}
	fn flush(&mut self) {}

	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => {
				self.gain = value;
				self.update_params();
			},
			1 => {
				self.pan = value;
				self.update_params();
			},
			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}
