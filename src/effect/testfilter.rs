use crate::dsp::onepole::OnePole;
use crate::dsp::simper::Filter;
use crate::effect::*;

#[derive(Debug)]
pub struct TestFilter {
	tracks: [Track; 2],
	cutoff: f32,
	q: f32,
	gain: f32,
	onepole: bool,
}

#[derive(Debug)]
struct Track {
	filter1: OnePole,
	filter2: Filter,
}

impl Track {
	fn new(sample_rate: f32) -> Self {
		Self { filter1: OnePole::new(sample_rate), filter2: Filter::new(sample_rate) }
	}
}

impl Effect for TestFilter {
	fn new(sample_rate: f32) -> Self {
		TestFilter {
			tracks: [Track::new(sample_rate), Track::new(sample_rate)],
			cutoff: 1.,
			q: 1.,
			gain: 1.,
			onepole: false,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		// let f = &self.tracks[0].filter2;
		// dbg!(f.phase_delay(1000.));
		// let a = f.phase_delay2(1000.) - f.phase_delay(1000.);

		if self.onepole {
			for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
				// track.filter1.set_lowpass(self.cutoff);
				track.filter1.set_tilt(self.cutoff, self.gain);
				for sample in buf.iter_mut() {
					*sample = track.filter1.process(*sample);
				}
			}
		} else {
			for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
				track.filter2.set_lowpass(self.cutoff, self.q);
				// track.filter2.set_tilt(self.cutoff, self.q, self.gain);
				for sample in buf.iter_mut() {
					*sample = track.filter2.process(*sample);
				}
			}
		}
	}
	fn flush(&mut self) {}

	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.cutoff = value,
			1 => self.q = value,
			2 => self.gain = value,
			3 => self.onepole = value > 0.5,
			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}
