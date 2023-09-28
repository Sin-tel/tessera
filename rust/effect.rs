pub mod convolve;
pub mod delay;
pub mod drive;
pub mod equalizer;
pub mod gain;
pub mod pan;
pub mod reverb;
pub mod testfilter;
pub mod tilt;

use crate::effect;
use crate::effect::{
	convolve::Convolve, delay::Delay, drive::Drive, equalizer::Equalizer, gain::Gain, pan::Pan,
	reverb::Reverb, testfilter::TestFilter, tilt::Tilt,
};

// list of effects
pub fn new(sample_rate: f32, effect_number: usize) -> Box<dyn Effect + Send> {
	match effect_number {
		0 => Box::new(Pan::new(sample_rate)),
		1 => Box::new(Gain::new(sample_rate)),
		2 => Box::new(Drive::new(sample_rate)),
		3 => Box::new(Delay::new(sample_rate)),
		4 => Box::new(Reverb::new(sample_rate)),
		5 => Box::new(TestFilter::new(sample_rate)),
		6 => Box::new(Equalizer::new(sample_rate)),
		7 => Box::new(Tilt::new(sample_rate)),
		8 => Box::new(Convolve::new(sample_rate)),
		_ => {
			eprintln!("Effect with number {effect_number} not found. Returning default.");
			Box::new(Gain::new(sample_rate))
		}
	}
}

pub trait Effect {
	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn process(&mut self, buffer: &mut [&mut [f32]; 2]);
	fn set_parameter(&mut self, index: usize, val: f32);
}

pub struct Bypass {
	pub effect: Box<dyn Effect + Send>,
	pub bypassed: bool,
}

impl Bypass {
	pub fn new(sample_rate: f32, index: usize) -> Self {
		Bypass {
			effect: effect::new(sample_rate, index),
			bypassed: false,
		}
	}
	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		if !self.bypassed {
			self.effect.process(buffer);
		}
	}
}
