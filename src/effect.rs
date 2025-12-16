mod compressor;
mod convolve;
mod delay;
mod drive;
mod equalizer;
mod gain;
mod pan;
mod reverb;
mod testfilter;
mod tilt;
mod wide;

use crate::audio::MAX_BUF_SIZE;
use crate::dsp;
use crate::dsp::MuteState;
use crate::dsp::time_constant;
use crate::effect;
use crate::effect::{
	compressor::Compressor, convolve::Convolve, delay::Delay, drive::Drive, equalizer::Equalizer,
	gain::Gain, pan::Pan, reverb::Reverb, testfilter::TestFilter, tilt::Tilt, wide::Wide,
};
use crate::log::log_warn;
use crate::meters::MeterHandle;

// list of effects
pub fn new(sample_rate: f32, name: &str) -> Box<dyn Effect + Send> {
	match name {
		"compressor" => Box::new(Compressor::new(sample_rate)),
		"convolve" => Box::new(Convolve::new(sample_rate)),
		"delay" => Box::new(Delay::new(sample_rate)),
		"drive" => Box::new(Drive::new(sample_rate)),
		"equalizer" => Box::new(Equalizer::new(sample_rate)),
		"gain" => Box::new(Gain::new(sample_rate)),
		"pan" => Box::new(Pan::new(sample_rate)),
		"reverb" => Box::new(Reverb::new(sample_rate)),
		"testfilter" => Box::new(TestFilter::new(sample_rate)),
		"tilt" => Box::new(Tilt::new(sample_rate)),
		"wide" => Box::new(Wide::new(sample_rate)),
		_ => {
			log_warn!("Effect with name \"{name}\" not found. Returning default.");
			Box::new(Gain::new(sample_rate))
		},
	}
}

pub trait Effect {
	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn process(&mut self, buffer: &mut [&mut [f32]; 2]);
	fn set_parameter(&mut self, index: usize, val: f32);
	fn flush(&mut self);
}

pub struct Bypass {
	pub effect: Box<dyn Effect + Send>,
	mute: bool,
	meter_handle: MeterHandle,

	value: f32,
	smoothing_f: f32,

	temp: [[f32; MAX_BUF_SIZE]; 2],
	values: [f32; MAX_BUF_SIZE],

	state: MuteState,
}

impl Bypass {
	pub fn new(sample_rate: f32, name: &str, meter_handle: MeterHandle) -> Self {
		Bypass {
			effect: effect::new(sample_rate, name),
			mute: false,
			meter_handle,
			value: 1.0,
			smoothing_f: time_constant(15.0, sample_rate),
			temp: [[0.0; MAX_BUF_SIZE]; 2],
			values: [0.0; MAX_BUF_SIZE],
			state: MuteState::Active,
		}
	}

	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		match self.state {
			MuteState::Active => {
				self.effect.process(buffer);

				let peak = dsp::peak(buffer);
				self.meter_handle.set(peak);
			},
			MuteState::Off => {
				self.meter_handle.set([0., 0.]);
			},
			MuteState::Transition => {
				let target = if self.mute { 0.0 } else { 1.0 };
				let samples = buffer[0].len();
				assert!(samples <= MAX_BUF_SIZE);

				self.temp[0][..samples].copy_from_slice(buffer[0]);
				self.temp[1][..samples].copy_from_slice(buffer[1]);

				for i in 0..samples {
					self.value += self.smoothing_f * (target - self.value);

					// Gain is applied twice, so need to take square root
					let wet_gain = self.value.sqrt();

					self.values[i] = wet_gain;
					let dry_gain = 1.0 - self.value;

					// Pre gain
					buffer[0][i] *= wet_gain;
					buffer[1][i] *= wet_gain;

					self.temp[0][i] *= dry_gain;
					self.temp[1][i] *= dry_gain;
				}

				self.effect.process(buffer);

				// Post gain
				for i in 0..samples {
					buffer[0][i] *= self.values[i];
					buffer[1][i] *= self.values[i];
				}

				let peak = dsp::peak(buffer);
				self.meter_handle.set(peak);

				// Add dry signal back in
				for i in 0..samples {
					buffer[0][i] += self.temp[0][i];
					buffer[1][i] += self.temp[1][i];
				}

				// Check state transition
				if (self.value - target).abs() < 1e-4 {
					self.value = target;
					if self.mute {
						self.state = MuteState::Off;
						self.flush();
					} else {
						self.state = MuteState::Active;
					}
				}
			},
		}
	}

	pub fn set_mute(&mut self, mute: bool) {
		self.mute = mute;
		self.state = MuteState::Transition;
	}

	pub fn flush(&mut self) {
		self.effect.flush();
		self.meter_handle.set([0., 0.]);
	}
}
