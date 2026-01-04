use crate::worker::load_and_resample;

const GAIN: f32 = 0.2;

pub struct Metronome {
	sample_down: [Vec<f32>; 2],
	sample_up: [Vec<f32>; 2],

	active: bool,
	accent: bool,
	position: usize,
}

impl Metronome {
	pub fn new(sample_rate: f32) -> Self {
		let load = |path| load_and_resample(path, sample_rate).expect("Metronome sample missing.");
		Self {
			sample_down: load("metronome/down.wav"),
			sample_up: load("metronome/up.wav"),
			active: false,
			accent: true,
			position: 0,
		}
	}

	pub fn trigger(&mut self, accent: bool) {
		self.active = true;
		self.position = 0;
		self.accent = accent;
	}

	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		if !self.active {
			return;
		}
		let sample = if self.accent { &self.sample_down } else { &self.sample_up };
		let len = sample[0].len();

		let [bl, br] = buffer;
		for (l, r) in bl.iter_mut().zip(br.iter_mut()) {
			if self.position >= len {
				self.active = false;
				break;
			}
			*l += sample[0][self.position] * GAIN;
			*r += sample[1][self.position] * GAIN;
			self.position += 1;
		}
	}
}
