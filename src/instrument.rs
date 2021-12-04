use crate::defs::*;

pub mod sine;

pub trait Instrument {
	// pub params: Vec<Param>,

	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn cv(&mut self, freq: f32, vol: f32);
	fn process(&mut self, buffer: &mut [StereoSample]);
	fn set_param(&mut self, index: usize, val: f32);
}
