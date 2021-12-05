use crate::defs::*;

pub mod sine;

pub trait Instrument {
	// pub params: Vec<Param>,

	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn cv(&mut self, pitch: f32, vel: f32);
	fn process(&mut self, buffer: &mut [StereoSample]);
	fn note_on(&mut self, pitch: f32, vel: f32);
	fn set_param(&mut self, index: usize, val: f32);
}
