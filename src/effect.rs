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
use crate::dsp::{MuteState, PeakMeter, time_constant};
use crate::effect;
use crate::effect::{
	compressor::Compressor, convolve::Convolve, delay::Delay, drive::Drive, equalizer::Equalizer,
	gain::Gain, pan::Pan, reverb::Reverb, testfilter::TestFilter, tilt::Tilt, wide::Wide,
};
use crate::log::log_warn;
use crate::meters::MeterHandle;
use crate::worker::{RequestData, ResponseData};

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
	#[must_use]
	fn set_parameter(&mut self, index: usize, val: f32) -> Option<RequestData>;
	fn flush(&mut self) {}
	#[must_use]
	fn receive_data(&mut self, _data: ResponseData) -> Option<Box<dyn std::any::Any + Send>> {
		log_warn!("Effect received data with no handler");
		None
	}
}

pub struct Bypass {
	pub effect: Box<dyn Effect + Send>,

	peak: PeakMeter,
	meter_handle: MeterHandle,

	mute: bool,
	state: MuteState,
	gain: f32,
	smoothing_f: f32,
	dry_temp: [[f32; MAX_BUF_SIZE]; 2],
	wet_gain: [f32; MAX_BUF_SIZE],
}

impl Bypass {
	pub fn new(sample_rate: f32, name: &str, meter_handle: MeterHandle) -> Self {
		Bypass {
			effect: effect::new(sample_rate, name),

			peak: PeakMeter::new(sample_rate),
			meter_handle,

			mute: false,
			state: MuteState::Active,
			gain: 1.0,
			smoothing_f: time_constant(15.0, sample_rate),
			dry_temp: [[0.0; MAX_BUF_SIZE]; 2],
			wet_gain: [0.0; MAX_BUF_SIZE],
		}
	}

	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		match self.state {
			MuteState::Active => {
				self.effect.process(buffer);
				let peak = self.peak.process_block(buffer);
				self.meter_handle.set(peak);
			},
			MuteState::Off => {
				self.meter_handle.set([0., 0.]);
			},
			MuteState::Transition => {
				let target = if self.mute { 0.0 } else { 1.0 };
				let samples = buffer[0].len();
				assert!(samples <= MAX_BUF_SIZE);

				self.dry_temp[0][..samples].copy_from_slice(buffer[0]);
				self.dry_temp[1][..samples].copy_from_slice(buffer[1]);

				for i in 0..samples {
					self.gain += self.smoothing_f * (target - self.gain);

					// Gain is applied twice, so need to take square root
					let wet_gain = self.gain.sqrt();

					self.wet_gain[i] = wet_gain;
					let dry_gain = 1.0 - self.gain;

					// Pre gain
					buffer[0][i] *= wet_gain;
					buffer[1][i] *= wet_gain;

					self.dry_temp[0][i] *= dry_gain;
					self.dry_temp[1][i] *= dry_gain;
				}

				self.effect.process(buffer);

				// Post gain
				for i in 0..samples {
					buffer[0][i] *= self.wet_gain[i];
					buffer[1][i] *= self.wet_gain[i];
				}

				let peak = self.peak.process_block(buffer);
				self.meter_handle.set(peak);

				// Add dry signal back in
				for i in 0..samples {
					buffer[0][i] += self.dry_temp[0][i];
					buffer[1][i] += self.dry_temp[1][i];
				}

				// Check state transition
				if (self.gain - target).abs() < 1e-4 {
					self.gain = target;
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
