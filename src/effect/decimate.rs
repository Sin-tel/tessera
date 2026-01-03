use crate::dsp::lerp;
use crate::dsp::simper::Filter;
use crate::dsp::smooth::Smooth;
use crate::effect::Effect;
use crate::log::log_warn;
use crate::worker::RequestData;
use halfband::iir::{Downsampler8, Upsampler8}; // Assuming these structs exist

// 4-pole butterworth Q
//  1 / 2 * cos(  pi/8)
//  1 / 2 * cos(3*pi/8)
const Q1: f32 = 0.541196100146;
const Q2: f32 = 1.30656296488;

#[derive(Debug)]
struct Track {
	upsampler: Upsampler8,
	downsampler: Downsampler8,
	pre_filters: [Filter; 2],
	post_filters: [Filter; 2],
	s3: f32,
	s2: f32,
	s1: f32,

	accum: f32,
	y: f32,
	prev_x: f32,
	jitter_val: f32,

	// parameters
	rate: Smooth,
	pre_filter_enable: bool,
}

impl Track {
	fn new(sample_rate: f32) -> Self {
		let mut t = Self {
			upsampler: Upsampler8::default(),
			downsampler: Downsampler8::default(),
			pre_filters: [Filter::new(sample_rate), Filter::new(sample_rate)],
			post_filters: [Filter::new(sample_rate), Filter::new(sample_rate)],
			s3: 0.0,
			s2: 0.0,
			s1: 0.0,
			accum: 0.0,
			y: 0.0,
			prev_x: 0.0,
			jitter_val: 1.0,
			rate: Smooth::new(4000.0, 100.0, sample_rate),
			pre_filter_enable: false,
		};

		t.pre_filters[0].set_lowpass(2000.0, Q1);
		t.pre_filters[1].set_lowpass(2000.0, Q2);
		t.post_filters[0].set_lowpass(2000.0, Q1);
		t.post_filters[1].set_lowpass(2000.0, Q2);

		t
	}

	// Resampling logic
	// Instead of doing a naive S&H, do linear interpolation when an integer boundary is crossed
	#[inline]
	fn tick(&mut self, x: f32, target_rate: f32, jitter_amount: f32, inv_2sr: f32) -> f32 {
		let step = target_rate * inv_2sr * self.jitter_val;

		self.accum += step;

		let mut out = self.y;

		while self.accum >= 1.0 {
			self.accum -= 1.0;

			let a = self.accum / step;

			// linear interpolation on input
			// note: inverted, we need to look back and find position where boundary was crossed
			let next_y = lerp(x, self.prev_x, a);

			// linear interpolation on output
			out = lerp(self.y, next_y, a);

			self.y = next_y;

			if jitter_amount > 0.0 {
				let noise = fastrand::f32() - 0.5;
				self.jitter_val = 1.0 + (noise * jitter_amount);
			} else {
				self.jitter_val = 1.0;
			}
		}

		self.prev_x = x;
		out
	}

	fn update_filters(&mut self, rate: f32, filter: f32) {
		let nyquist = rate * 0.5;
		self.pre_filters[0].set_lowpass(rate * 0.5, Q1);
		self.pre_filters[1].set_lowpass(rate * 0.5, Q2);

		let cutoff = lerp(20_000.0, nyquist, filter);
		self.post_filters[0].set_lowpass(cutoff, Q1);
		self.post_filters[1].set_lowpass(cutoff, Q2);
	}
}

pub struct Decimate {
	tracks: [Track; 2],
	inv_2sr: f32,

	// parameters
	jitter: f32,
	filter: f32,
}

impl Effect for Decimate {
	fn new(sample_rate: f32) -> Self {
		Self {
			tracks: [Track::new(sample_rate), Track::new(sample_rate)],
			inv_2sr: 1.0 / (2.0 * sample_rate),
			jitter: 0.0,
			filter: 0.0,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			// update filters once per block
			let current_rate = track.rate.target();
			track.update_filters(current_rate, self.filter);

			for sample in buf.iter_mut() {
				let mut s = *sample;
				let rate = track.rate.process();

				if track.pre_filter_enable {
					s = track.pre_filters[0].process(s);
					s = track.pre_filters[1].process(s);
				}

				let [u1, u2] = track.upsampler.process(s);
				let t1 = track.tick(u1, rate, self.jitter, self.inv_2sr);
				let t2 = track.tick(u2, rate, self.jitter, self.inv_2sr);
				let mut out = track.downsampler.process(t1, t2);

				out = track.post_filters[0].process(out);
				out = track.post_filters[1].process(out);

				*sample = out;
			}
		}
	}

	fn flush(&mut self) {}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.tracks.iter_mut().for_each(|t| t.rate.set(value)),
			1 => self.jitter = 0.1 * value * value,
			2 => self.filter = 1.0 - (1.0 - value).powi(2),
			3 => self.tracks.iter_mut().for_each(|t| t.pre_filter_enable = value > 0.5),
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
