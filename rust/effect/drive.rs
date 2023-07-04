use crate::dsp::resample::{Downsampler51, Upsampler19};
use crate::dsp::DcKiller;
use crate::effect::Effect;

// TODO: store previous sample eval of antiderivative
// TODO: add some shelving / lowpass / higpass shaping
// TODO: dry/wet, delay compensation?

#[derive(Debug, Default)]
pub struct Drive {
	tracks: [Track; 2],
	gain: f32,
	oversample_mode: usize,
	bias: f32,
	hard: bool,
}

#[derive(Debug, Default)]
struct Track {
	prev: f32,
	upsampler: Upsampler19,
	downsampler: Downsampler51,
	dc_killer: DcKiller,
}

impl Track {
	fn new(sample_rate: f32) -> Self {
		Self {
			dc_killer: DcKiller::new(sample_rate),
			..Default::default()
		}
	}
}

impl Effect for Drive {
	fn new(sample_rate: f32) -> Self {
		Drive {
			gain: 1.0,
			tracks: [Track::new(sample_rate), Track::new(sample_rate)],
			..Default::default()
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		match self.oversample_mode {
			// 1st order ADAA
			0 => {
				if self.hard {
					for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
						for sample in s.iter_mut() {
							let x = (*sample + self.bias) * self.gain;

							let out = adaa_hard(x, track.prev);

							*sample = track.dc_killer.process(out);
							track.prev = x;
						}
					}
				} else {
					for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
						for sample in s.iter_mut() {
							let x = (*sample + self.bias) * self.gain;

							let out = adaa_soft(x, track.prev);

							*sample = track.dc_killer.process(out);
							track.prev = x;
						}
					}
				}
			}
			// 2x oversample + ADAA
			1 => {
				if self.hard {
					for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
						for sample in s.iter_mut() {
							let x = (*sample + self.bias) * self.gain;

							let (u1, u2) = track.upsampler.process(x);

							let res1 = adaa_hard(u1, track.prev);
							let res2 = adaa_hard(u2, u1);
							track.prev = u2;

							let out = track.downsampler.process(res1, res2);

							*sample = track.dc_killer.process(out);
						}
					}
				} else {
					for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
						for sample in s.iter_mut() {
							let x = (*sample + self.bias) * self.gain;

							let (u1, u2) = track.upsampler.process(x);

							let res1 = adaa_soft(u1, track.prev);
							let res2 = adaa_soft(u2, u1);
							track.prev = u2;

							let out = track.downsampler.process(res1, res2);

							*sample = track.dc_killer.process(out);
						}
					}
				}
			}
			// naive (not used)
			_ => {
				if self.hard {
					for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
						for sample in s.iter_mut() {
							let x = (*sample + self.bias) * self.gain;
							*sample = track.dc_killer.process(clip_hard(x));
						}
					}
				} else {
					for (s, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
						for sample in s.iter_mut() {
							let x = (*sample + self.bias) * self.gain;
							*sample = track.dc_killer.process(clip_soft(x));
						}
					}
				}
			}
		}
	}
	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.hard = value > 0.5,
			1 => self.gain = value,
			2 => self.bias = value,
			3 => self.oversample_mode = value as usize,
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}

// This is pretty high because we do finite difference on f32
const ADAA_TOLERANCE: f32 = 1e-3;

// x0 current sample
// x1 previous sample
fn adaa_soft(x0: f32, x1: f32) -> f32 {
	let diff = x0 - x1;
	if diff.abs() < ADAA_TOLERANCE {
		clip_soft(0.5 * (x0 + x1))
	} else {
		(clip_soft_ad(x0) - clip_soft_ad(x1)) / diff
	}
}

fn adaa_hard(x0: f32, x1: f32) -> f32 {
	let diff = x0 - x1;
	if diff.abs() < ADAA_TOLERANCE {
		clip_hard(0.5 * (x0 + x1))
	} else {
		(clip_hard_ad(x0) - clip_hard_ad(x1)) / diff
	}
}

fn clip_soft(x: f32) -> f32 {
	let x = x.clamp(-16. / 9., 16. / 9.);
	(65536. * x) / (256. + 27. * x * x).powi(2)
}

fn clip_hard(x: f32) -> f32 {
	let x = x.clamp(-5. / 4., 5. / 4.);
	x - (256. / 3125.) * x.powi(5)
}

// antiderivatives
fn clip_soft_ad(x: f32) -> f32 {
	let x = x.abs();
	if x < (16. / 9.) {
		let a = x * x;
		(128. * a) / (27. * a + 256.)
	} else {
		x - (16. / 27.)
	}
}

fn clip_hard_ad(x: f32) -> f32 {
	let x = x.abs();
	if x < (5. / 4.) {
		let a = x * x;
		a * (-256. * a * a + 9375.) / 18750.
	} else {
		x - (25. / 48.)
	}
}
