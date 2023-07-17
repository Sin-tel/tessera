use crate::dsp::from_db;
use crate::dsp::onepole::OnePole;
use crate::effect::Effect;

// TODO: better gain matching

// logarithmically spaced bands
const FREQ_1: f32 = 328.56;
const FREQ_2: f32 = 929.30;
const FREQ_3: f32 = 2628.45;
const FREQ_4: f32 = 7434.40;

#[derive(Debug)]
pub struct Tilt {
	tracks: [Track; 2],
	gain: f32,
}

#[derive(Debug)]
struct Track {
	filter1: OnePole,
	filter2: OnePole,
	filter3: OnePole,
	filter4: OnePole,
}

impl Track {
	fn new(sample_rate: f32) -> Self {
		Self {
			filter1: OnePole::new(sample_rate),
			filter2: OnePole::new(sample_rate),
			filter3: OnePole::new(sample_rate),
			filter4: OnePole::new(sample_rate),
		}
	}
}

impl Effect for Tilt {
	fn new(sample_rate: f32) -> Self {
		Tilt {
			tracks: [Track::new(sample_rate), Track::new(sample_rate)],
			gain: 1.,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			for sample in buf.iter_mut() {
				let mut s = *sample;
				s = track.filter1.process(s);
				s = track.filter2.process(s);
				s = track.filter3.process(s);
				s = track.filter4.process(s);
				*sample = s * self.gain;
			}
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => {
				// empirical correction factor
				let slope = value * 1.58;
				for track in self.tracks.iter_mut() {
					track.filter1.set_tilt(FREQ_1, slope);
					track.filter2.set_tilt(FREQ_2, slope);
					track.filter3.set_tilt(FREQ_3, slope);
					track.filter4.set_tilt(FREQ_4, slope);
				}
				self.gain = from_db(-1.5 * value.abs())
			}
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}
