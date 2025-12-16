use crate::dsp::delayline::DelayLine;
use crate::dsp::simper::Filter;
use crate::dsp::smooth::SmoothBuffer;
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
	gain: SmoothBuffer,
	delay_f: Filter,
	delay: f32,
	filter: Filter,
	delayline: DelayLine,
}

impl Track {
	pub fn new(sample_rate: f32) -> Self {
		let mut filter = Filter::new(sample_rate);
		filter.set_highshelf(HEAD_CUTOFF, HEAD_Q, 0.0);

		let mut delay_f = Filter::new(sample_rate);
		delay_f.set_lowpass(2.5, 0.5);

		Track {
			gain: SmoothBuffer::new(1.0, 25.0, sample_rate),
			delay: 0.0,
			delay_f,
			filter,
			delayline: DelayLine::new(sample_rate, ITD),
		}
	}
}

impl Pan {
	fn update_params(&mut self) {
		self.tracks[0].delay = (ITD * self.pan).max(0.0);
		self.tracks[1].delay = (-ITD * self.pan).max(0.0);

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
		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			// delay
			for sample in buf.iter_mut() {
				let delay = track.delay_f.process(track.delay);
				let input = *sample;
				track.delayline.push(input);

				*sample = track.delayline.go_back_cubic(delay);
			}

			// head shadow filter
			track.filter.process_block(buf);

			// // volume difference
			track.gain.process_block(buf.len());
			track.gain.multiply_block(buf);
		}
	}
	fn flush(&mut self) {
		for track in &mut self.tracks {
			track.delay_f.reset_state();
			track.delayline.flush();
		}
	}

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
