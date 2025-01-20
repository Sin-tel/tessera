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

use crate::effect;
use crate::effect::{
	compressor::Compressor, convolve::Convolve, delay::Delay, drive::Drive, equalizer::Equalizer,
	gain::Gain, pan::Pan, reverb::Reverb, testfilter::TestFilter, tilt::Tilt, wide::Wide,
};
use crate::log::log_warn;

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
	pub bypassed: bool,
}

impl Bypass {
	pub fn new(sample_rate: f32, name: &str) -> Self {
		Bypass { effect: effect::new(sample_rate, name), bypassed: false }
	}
	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		if !self.bypassed {
			self.effect.process(buffer);
		}
	}
}
