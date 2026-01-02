mod analog;
mod epiano;
mod fm;
mod pluck;
mod polysine;
mod sampler;
mod sine;
mod wavetable;

use crate::instrument::{
	analog::Analog, epiano::Epiano, fm::Fm, pluck::Pluck, polysine::Polysine, sampler::Sampler,
	sine::Sine, wavetable::Wavetable,
};
use crate::log::log_warn;
use crate::worker::RequestData;
use crate::worker::ResponseData;

// list of instruments
pub fn new(sample_rate: f32, name: &str) -> Box<dyn Instrument + Send> {
	match name {
		"analog" => Box::new(Analog::new(sample_rate)),
		"epiano" => Box::new(Epiano::new(sample_rate)),
		"fm" => Box::new(Fm::new(sample_rate)),
		"pluck" => Box::new(Pluck::new(sample_rate)),
		"polysine" => Box::new(Polysine::new(sample_rate)),
		"sampler" => Box::new(Sampler::new(sample_rate)),
		"sine" => Box::new(Sine::new(sample_rate)),
		"wavetable" => Box::new(Wavetable::new(sample_rate)),
		_ => {
			log_warn!("Instrument with name \"{name}\" not found. Returning default.");
			Box::new(Sine::new(sample_rate))
		},
	}
}

pub trait Instrument {
	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn process(&mut self, buffer: &mut [&mut [f32]; 2]);
	fn note_on(&mut self, pitch: f32, vel: f32, id: usize);
	fn note_off(&mut self, id: usize);
	fn pitch(&mut self, pitch: f32, id: usize);
	fn pressure(&mut self, pressure: f32, id: usize);
	fn set_parameter(&mut self, index: usize, val: f32) -> Option<RequestData>;
	fn flush(&mut self);
	fn voice_count(&self) -> usize;
	#[must_use]
	fn receive_data(&mut self, _data: ResponseData) -> Option<Box<dyn std::any::Any + Send>> {
		log_warn!("Effect received data with no handler");
		None
	}
}
