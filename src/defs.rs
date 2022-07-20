use std::ops::{Add, Div, Mul, Sub};

pub const TWO_PI: f32 = std::f32::consts::TAU;
pub const MAX_BUF_SIZE: usize = 1024;

#[derive(Debug, Copy, Clone, Default)]
pub struct Stereo(pub [f32; 2]);

#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
pub struct Mono(pub f32);

pub trait Sample:
	Add<Self, Output = Self>
	+ Add<f32, Output = Self>
	+ Sub<Self, Output = Self>
	+ Sub<f32, Output = Self>
	+ Mul<Self, Output = Self>
	+ Mul<f32, Output = Self>
	+ Div<Self, Output = Self>
	+ Div<f32, Output = Self>
	+ Sized
	+ Copy
	+ Default
{
	fn map<F: Fn(f32) -> f32>(self, f: F) -> Self;
}

impl Sample for Mono {
	fn map<F: Fn(f32) -> f32>(self, f: F) -> Self {
		Mono(f(self.0))
	}
}

impl Add<Mono> for Mono {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Mono(self.0 + other.0)
	}
}

impl Add<f32> for Mono {
	type Output = Self;

	fn add(self, other: f32) -> Self {
		Mono(self.0 + other)
	}
}

impl Sub<Mono> for Mono {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		Mono(self.0 - other.0)
	}
}

impl Sub<f32> for Mono {
	type Output = Self;

	fn sub(self, other: f32) -> Self {
		Mono(self.0 - other)
	}
}

impl Mul<Mono> for Mono {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		Mono(self.0 * other.0)
	}
}

impl Mul<f32> for Mono {
	type Output = Self;

	fn mul(self, other: f32) -> Self {
		Mono(self.0 * other)
	}
}

impl Div<Mono> for Mono {
	type Output = Self;

	fn div(self, other: Self) -> Self {
		Mono(self.0 / other.0)
	}
}

impl Div<f32> for Mono {
	type Output = Self;

	fn div(self, other: f32) -> Self {
		Mono(self.0 / other)
	}
}

////////////////

impl Sample for Stereo {
	fn map<F: Fn(f32) -> f32>(self, f: F) -> Self {
		Stereo([f(self.0[0]), f(self.0[1])])
	}
}

impl Add<Stereo> for Stereo {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Stereo([self.0[0] + other.0[0], self.0[1] + other.0[1]])
	}
}

impl Add<f32> for Stereo {
	type Output = Self;

	fn add(self, other: f32) -> Self {
		Stereo([self.0[0] + other, self.0[1] + other])
	}
}

impl Sub<Stereo> for Stereo {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		Stereo([self.0[0] - other.0[0], self.0[1] - other.0[1]])
	}
}

impl Sub<f32> for Stereo {
	type Output = Self;

	fn sub(self, other: f32) -> Self {
		Stereo([self.0[0] - other, self.0[1] - other])
	}
}

impl Mul<Stereo> for Stereo {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		Stereo([self.0[0] * other.0[0], self.0[1] * other.0[1]])
	}
}

impl Mul<f32> for Stereo {
	type Output = Self;

	fn mul(self, other: f32) -> Self {
		Stereo([self.0[0] * other, self.0[1] * other])
	}
}

impl Div<Stereo> for Stereo {
	type Output = Self;

	fn div(self, other: Self) -> Self {
		Stereo([self.0[0] / other.0[0], self.0[1] / other.0[1]])
	}
}

impl Div<f32> for Stereo {
	type Output = Self;

	fn div(self, other: f32) -> Self {
		Stereo([self.0[0] / other, self.0[1] / other])
	}
}

// Message struct to pass to audio thread
// Should not contain any boxed values
#[derive(Debug)]
pub enum AudioMessage {
	CV(usize, f32, f32),
	Note(usize, f32, f32, usize),
	SetParam(usize, usize, usize, f32),
	Pan(usize, f32, f32),
	Mute(usize, bool),
	// Bypass(usize, usize, bool),
}

#[repr(C)]
#[derive(Debug)]
pub enum LuaMessage {
	Cpu(f32),
	Meter(f32, f32),
}
