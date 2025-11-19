use crate::audio::MAX_BUF_SIZE;
use crate::dsp::onepole::OnePole;
use crate::dsp::resample_fir::{Downsampler, Upsampler, COEF_19, COEF_51};
use crate::dsp::smooth::SmoothBuffer;
use crate::dsp::*;
use crate::effect::*;

// TODO: store previous sample eval of antiderivative
// TODO: add proper interpolation for gain and bias
// TODO: Delay compensation in dry path

#[derive(Debug)]
pub struct Drive {
	tracks: [Track; 2],
	gain: SmoothBuffer,
	post_gain: SmoothBuffer,
	gain_comp: f32,
	oversample: bool,
	bias: f32,
	hard: bool,
	balance: f32,
}

#[derive(Debug)]
struct Track {
	prev: f32,
	upsampler: Upsampler<{ COEF_19.len() }>,
	downsampler: Downsampler<{ COEF_51.len() }>,
	dc_killer: DcKiller,
	pre_filter: OnePole,
	post_filter: OnePole,
	buffer: [f32; MAX_BUF_SIZE],
}

impl Track {
	fn new(sample_rate: f32) -> Self {
		Self {
			prev: 0.,
			upsampler: Upsampler::<{ COEF_19.len() }>::new(&COEF_19),
			downsampler: Downsampler::<{ COEF_51.len() }>::new(&COEF_51),
			dc_killer: DcKiller::new(sample_rate),
			pre_filter: OnePole::new(sample_rate),
			post_filter: OnePole::new(sample_rate),
			buffer: [0.; MAX_BUF_SIZE],
		}
	}
}

impl Effect for Drive {
	fn new(sample_rate: f32) -> Self {
		Drive {
			tracks: [Track::new(sample_rate), Track::new(sample_rate)],
			gain: SmoothBuffer::new(),
			post_gain: SmoothBuffer::new(),
			gain_comp: 0.,
			oversample: false,
			bias: 0.,
			hard: false,
			balance: 0.,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let n = buffer[0].len();
		self.gain.process_buffer(n);
		self.post_gain.process_buffer(n);

		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			// track.buffer.clone_from_slice(buf);
			buf.iter()
				.zip(track.buffer.iter_mut())
				.for_each(|(src, dst)| *dst = *src);
			for (i, sample) in buf.iter_mut().enumerate() {
				let s = track.pre_filter.process(*sample);
				*sample = s * self.gain.get(i) + self.bias;
			}
		}

		if self.oversample {
			// 2x oversample + ADAA
			if self.hard {
				for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
					for sample in buf.iter_mut() {
						let (u1, u2) = track.upsampler.process(*sample);

						let res1 = adaa_hard(u1, track.prev);
						let res2 = adaa_hard(u2, u1);
						track.prev = u2;

						*sample = track.downsampler.process(res1, res2);
					}
				}
			} else {
				for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
					for sample in buf.iter_mut() {
						let (u1, u2) = track.upsampler.process(*sample);

						let res1 = adaa_soft(u1, track.prev);
						let res2 = adaa_soft(u2, u1);
						track.prev = u2;

						*sample = track.downsampler.process(res1, res2);
					}
				}
			}
		} else {
			// 1st order ADAA
			if self.hard {
				for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
					for sample in buf.iter_mut() {
						let x = *sample;
						*sample = adaa_hard(x, track.prev);
						track.prev = x;
					}
				}
			} else {
				for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
					for sample in buf.iter_mut() {
						let x = *sample;
						*sample = adaa_soft(x, track.prev);
						track.prev = x;
					}
				}
			}
		}
		// naive (not used)
		// _ => {
		// 	if self.hard {
		// 		for buf in buffer.iter_mut() {
		// 			for sample in buf.iter_mut() {
		// 				*sample = clip_hard(*sample);
		// 			}
		// 		}
		// 	} else {
		// 		for buf in buffer.iter_mut() {
		// 			for sample in buf.iter_mut() {
		// 				*sample = clip_soft(*sample);
		// 			}
		// 		}
		// 	}
		// },

		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			for (i, (sample, dry)) in buf.iter_mut().zip(track.buffer).enumerate() {
				let s = track.post_filter.process(*sample - self.bias) * self.post_gain.get(i);
				*sample = track.dc_killer.process(lerp(dry, s, self.balance));
			}
		}
	}
	fn flush(&mut self) {}

	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.balance = value,
			1 => self.hard = value > 0.5,
			2 => {
				let gain = from_db(value);
				self.gain.set(gain);
				self.post_gain.set(self.gain_comp / gain);
			},
			3 => {
				self.gain_comp = from_db(value);
				self.post_gain.set(self.gain_comp / self.gain.target());
			},
			4 => self.bias = value,
			5 => self.tracks.iter_mut().for_each(|v| {
				v.pre_filter.set_tilt(700., -value);
				v.post_filter.set_tilt(700., value);
			}),
			6 => self.oversample = value > 0.5,
			_ => log_warn!("Parameter with index {index} not found"),
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
