use crate::dsp::simper::Filter;
use crate::dsp::*;
use crate::effect::*;

#[derive(Debug)]
pub struct Equalizer {
	tracks: [Track; 2],

	low_cutoff: f32,
	low_gain: f32,
	low_cut: bool,

	bell1_cutoff: f32,
	bell1_q: f32,
	bell1_gain: f32,

	bell2_cutoff: f32,
	bell2_q: f32,
	bell2_gain: f32,

	high_cut: bool,
	high_cutoff: f32,
	high_gain: f32,
}

#[derive(Debug)]
struct Track {
	low: Filter,
	bell1: Filter,
	bell2: Filter,
	high: Filter,
}

impl Track {
	fn new(sample_rate: f32) -> Self {
		Self {
			low: Filter::new(sample_rate),
			bell1: Filter::new(sample_rate),
			bell2: Filter::new(sample_rate),
			high: Filter::new(sample_rate),
		}
	}
}

impl Effect for Equalizer {
	fn new(sample_rate: f32) -> Self {
		Equalizer {
			tracks: [Track::new(sample_rate), Track::new(sample_rate)],
			low_cutoff: 1.,
			low_gain: 1.,
			low_cut: false,

			bell1_cutoff: 1.,
			bell1_q: 1.,
			bell1_gain: 1.,

			bell2_cutoff: 1.,
			bell2_q: 1.,
			bell2_gain: 1.,

			high_cut: false,
			high_cutoff: 1.,
			high_gain: 1.,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			if self.low_cut {
				track.low.set_highpass(self.low_cutoff, BUTTERWORTH_Q);
			} else {
				track.low.set_lowshelf(self.low_cutoff, BUTTERWORTH_Q, self.low_gain);
			}
			track.bell1.set_bell(self.bell1_cutoff, self.bell1_q, self.bell1_gain);
			track.bell2.set_bell(self.bell2_cutoff, self.bell2_q, self.bell2_gain);
			if self.high_cut {
				track.high.set_lowpass(self.high_cutoff, BUTTERWORTH_Q);
			} else {
				track
					.high
					.set_highshelf(self.high_cutoff, BUTTERWORTH_Q, self.high_gain);
			}

			for sample in buf.iter_mut() {
				let mut s = *sample;
				s = track.low.process(s);
				s = track.bell1.process(s);
				s = track.bell2.process(s);
				s = track.high.process(s);

				*sample = s;
			}
		}
	}
	fn flush(&mut self) {}

	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => {
				self.low_gain = value;
				self.low_cut = value < -23.;
			},
			1 => self.bell1_gain = value,
			2 => self.bell2_gain = value,
			3 => {
				self.high_gain = value;
				self.high_cut = value < -23.;
			},
			4 => self.low_cutoff = value,
			5 => self.bell1_cutoff = value,
			6 => self.bell2_cutoff = value,
			7 => self.high_cutoff = value,
			8 => self.bell1_q = value,
			9 => self.bell2_q = value,
			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}
