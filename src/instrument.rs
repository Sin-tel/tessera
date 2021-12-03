use crate::defs::*;

pub mod sine;

pub trait Instrument {
	fn cv(&mut self, freq: f32, vol: f32);
	fn process(&mut self, buffer: &mut [StereoSample]);
}
