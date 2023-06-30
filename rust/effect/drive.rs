use crate::effect::Effect;

#[derive(Debug, Default)]
pub struct Drive {
	tracks: [Track; 2],
	gain: f32,
	mode: usize,
}

#[derive(Debug, Default)]
struct Track {
	prev: f32,
}

impl Effect for Drive {
	fn new(_sample_rate: f32) -> Self {
		Drive {
			gain: 1.0,
			..Default::default()
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		match self.mode {
			1 => {
				for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
					for sample in s.iter_mut() {
						let x = *sample * self.gain;

						let diff = x - track.prev;
						let res = if diff.abs() < 1e-7 {
							0.5 * (x - track.prev)
						} else {
							(clip_ad(x) - clip_ad(track.prev)) / diff
						};
						*sample = res * 0.5;
						track.prev = x;
					}
				}
			}
			// naive mode
			_ => {
				for b in buffer.iter_mut() {
					for s in b.iter_mut() {
						let x = *s * self.gain;
						*s = clip(x) * 0.5;
					}
				}
			}
		}
	}
	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.gain = value,
			1 => self.mode = value as usize,
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}

fn clip(x: f32) -> f32 {
	x.clamp(-1.0, 1.0)
}

// antiderivative
fn clip_ad(x: f32) -> f32 {
	0.25 * ((x + 1.0).powi(2) * (x + 1.0).signum() - (x - 1.0).powi(2) * (x - 1.0).signum() - 2.0)
}
