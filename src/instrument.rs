use crate::defs::*;
use crate::param::Param;

pub mod sine;

pub trait InstrumentP: Instrument + Param {}
impl<T: Instrument + Param> InstrumentP for T {}

pub trait EffectP: Effect + Param {}
impl<T: Effect + Param> EffectP for T {}

pub trait Instrument {
	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn cv(&mut self, pitch: f32, vel: f32);
	fn process(&mut self, buffer: &mut [StereoSample]);
	fn note_on(&mut self, pitch: f32, vel: f32);
	// fn set_param(&mut self, index: usize, val: f32);
}

pub trait Effect {
	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn process(&mut self, buffer: &mut [StereoSample]);
}
