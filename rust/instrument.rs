pub mod analog;
pub mod epiano;
pub mod fm;
pub mod polysine;
pub mod sine;
pub mod wavetable;

use crate::instrument::{
	analog::Analog, epiano::Epiano, fm::Fm, polysine::Polysine, sine::Sine, wavetable::Wavetable,
};
use crate::log::log_warn;

// list of instruments
pub fn new(sample_rate: f32, instrument_number: usize) -> Box<dyn Instrument + Send> {
	match instrument_number {
		0 => Box::new(Sine::new(sample_rate)),
		1 => Box::new(Wavetable::new(sample_rate)),
		2 => Box::new(Analog::new(sample_rate)),
		3 => Box::new(Fm::new(sample_rate)),
		4 => Box::new(Polysine::new(sample_rate)),
		5 => Box::new(Epiano::new(sample_rate)),
		_ => {
			log_warn!("Instrument with number {instrument_number} not found. Returning default.");
			Box::new(Sine::new(sample_rate))
		},
	}
}

pub trait Instrument {
	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn cv(&mut self, pitch: f32, pres: f32, id: usize);
	fn process(&mut self, buffer: &mut [&mut [f32]; 2]);
	fn note(&mut self, pitch: f32, vel: f32, id: usize);
	fn set_parameter(&mut self, index: usize, val: f32);
}
