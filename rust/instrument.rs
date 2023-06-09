use crate::defs::*;
use crate::device::Param;

pub mod sine;
pub mod wavetable;

pub trait Instrument: Param {
	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn cv(&mut self, pitch: f32, vel: f32);
	fn process(&mut self, buffer: &mut [&mut [f32]; 2]);
	fn note(&mut self, pitch: f32, vel: f32, id: usize);
}
