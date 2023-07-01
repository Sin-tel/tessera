use crate::dsp::resample::{Downsampler19, Downsampler51, Upsampler19};
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
	prev2: f32,
	upsampler: Upsampler19,
	upsampler2: Upsampler19,
	downsampler: Downsampler51,
	downsampler2: Downsampler19,
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
			// 1st order ADAA
			1 => {
				for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
					for sample in s.iter_mut() {
						let x = *sample * self.gain;

						*sample = adaa(x, track.prev) * 0.5;
						track.prev = x;
					}
				}
			}
			// 2x oversample
			2 => {
				for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
					for sample in s.iter_mut() {
						let x = *sample * self.gain;

						let (u1, u2) = track.upsampler.process(x);

						let out = track.downsampler.process(clip(u1), clip(u2));

						*sample = out * 0.5;
					}
				}
			}
			// 4x oversample
			3 => {
				for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
					for sample in s.iter_mut() {
						let x = *sample * self.gain;

						let (u1, u2) = track.upsampler.process(x);
						let (t1, t2) = track.upsampler2.process(u1);
						let (t3, t4) = track.upsampler2.process(u2);

						let out = track.downsampler.process(
							track.downsampler2.process(clip(t1), clip(t2)),
							track.downsampler2.process(clip(t3), clip(t4)),
						);

						*sample = out * 0.5;
					}
				}
			}
			// 2x oversample + ADAA
			4 => {
				for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
					for sample in s.iter_mut() {
						let x = *sample * self.gain;

						let (u1, u2) = track.upsampler.process(x);

						let res1 = adaa(u1, track.prev);
						let res2 = adaa(u2, u1);
						track.prev = u2;

						let out = track.downsampler.process(res1, res2);

						*sample = out * 0.5;
					}
				}
			}
			// 2nd order ADAA
			5 => {
				for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
					for sample in s.iter_mut() {
						let x = *sample * self.gain;

						*sample = adaa2(x, track.prev, track.prev2) * 0.5;
						track.prev2 = track.prev;
						track.prev = x;
					}
				}
			}
			// naive mode
			_ => {
				for (s, _) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
					for sample in s.iter_mut() {
						let x = *sample * self.gain;
						*sample = clip(x) * 0.5;
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

// x0 current sample
// x1 previous sample
fn adaa(x0: f32, x1: f32) -> f32 {
	let diff = x0 - x1;
	if diff.abs() < 1e-3 {
		clip(0.5 * (x0 + x1))
	} else {
		(clip_ad(x0) - clip_ad(x1)) / diff
	}
}

// x0 current sample
// x1 previous sample
// x2 2 samples before
fn adaa2(x0: f32, x1: f32, x2: f32) -> f32 {
	let diff = x0 - x2;

	if diff.abs() < 0.01 {
		let x_bar = 0.5 * (x0 + x2);

		let delta = x_bar - x1;
		if delta.abs() < 0.03 {
			return clip(0.5 * (x_bar + x1));
		} else {
			return 2.0 * (clip_ad(x_bar) + (clip_ad2(x1) - clip_ad2(x_bar)) / delta) / delta;
		}
	}

	2.0 * (adaa2_diff(x0, x1) - adaa2_diff(x1, x2)) / diff
}

fn adaa2_diff(x0: f32, x1: f32) -> f32 {
	let diff = x0 - x1;
	if diff.abs() < 0.03 {
		clip_ad(0.5 * (x0 + x1))
	} else {
		(clip_ad2(x0) - clip_ad2(x1)) / diff
	}
}

fn clip(x: f32) -> f32 {
	x.clamp(-1.0, 1.0)
}

// antiderivative
fn clip_ad(x: f32) -> f32 {
	let x1 = x + 1.0;
	let xm1 = x - 1.0;
	0.25 * (x1.abs() * x1 - xm1.abs() * xm1 - 2.0)
}

fn clip_ad2(x: f32) -> f32 {
	let x1 = x + 1.0;
	let xm1 = x - 1.0;
	(1.0 / 12.0) * (x1.abs() * x1 * x1 - xm1.abs() * xm1 * xm1 - 6.0 * x)
}
