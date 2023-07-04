pub mod analog;
pub mod fm;
pub mod polysine;
pub mod sine;
pub mod wavetable;

pub trait Instrument {
	fn new(sample_rate: f32) -> Self
	where
		Self: Sized;
	fn cv(&mut self, pitch: f32, pres: f32, id: usize);
	fn process(&mut self, buffer: &mut [&mut [f32]; 2]);
	fn note(&mut self, pitch: f32, vel: f32, id: usize);
	fn set_parameter(&mut self, index: usize, val: f32);
}
