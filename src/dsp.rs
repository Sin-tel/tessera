use crate::defs::*;

pub mod delayline;
pub mod simper;

pub fn pitch_to_f(p: f32, sample_rate: f32) -> f32 {
	// tuning to C4 = 261.63 instead of A4 = 440
	(2.0_f32).powf((p - 60.0) / 12.0) * 261.625_58 / sample_rate
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
	a * (1.0 - t) + b * t
}

pub fn from_db(x: f32) -> f32 {
	(10.0f32).powf(x / 20.0)
}

// pub fn softclip(x: f32) -> f32 {
// 	let s = x.clamp(-3.0, 3.0);
// 	s * (27.0 + s * s) / (27.0 + 9.0 * s * s)
// }
pub fn softclip<T>(x: T) -> T
where
	T: Sample,
{
	let s = x.map(|x| x.clamp(-3.0, 3.0));
	// let s = x;
	s * (s * s + 27.0) / (s * s * 9.0 + 27.0)
}

pub fn softclip_cubic(x: f32) -> f32 {
	let s = x.clamp(-1.5, 1.5);
	s * (1.0 - (4. / 27.) * s * s)
}

pub fn pow2_fast(p: f32) -> f32 {
	let offset = if p < 0.0 { 1.0_f32 } else { 0.0_f32 };
	let clipp = if p < -126.0 { -126.0_f32 } else { p };
	let w = clipp as i32;
	let z = clipp - (w as f32) + offset;
	let v = ((1 << 23) as f32
		* (clipp + 121.274_055_f32 + 27.728_024_f32 / (4.842_525_5_f32 - z) - 1.490_129_1_f32 * z))
		as u32;
	f32::from_bits(v)
}

#[derive(Debug)]
pub struct Smoothed {
	pub value: f32,
	inner: f32,
	f: f32,
}

impl Smoothed {
	pub fn new(value: f32, f: f32, sample_rate: f32) -> Self {
		Smoothed {
			inner: value,
			value,
			f: f / sample_rate,
		}
	}
	pub fn update(&mut self) {
		self.value = lerp(self.value, self.inner, self.f);
	}

	pub fn set(&mut self, v: f32) {
		self.inner = v;
	}

	pub fn set_hard(&mut self, v: f32) {
		self.inner = v;
		self.value = v;
	}
}

impl Default for Smoothed {
	fn default() -> Self {
		Smoothed::new(0.0, 0.5, 1.0)
	}
}

#[derive(Debug)]
pub struct SmoothedEnv {
	pub value: f32,
	inner: f32,
	attack: f32,
	release: f32,
}

impl SmoothedEnv {
	pub fn new(value: f32, attack: f32, release: f32, sample_rate: f32) -> Self {
		SmoothedEnv {
			inner: value,
			value,
			attack: attack / sample_rate,
			release: release / sample_rate,
		}
	}

	pub fn update(&mut self) {
		self.value = lerp(
			self.value,
			self.inner,
			if self.inner > self.value {
				self.attack
			} else {
				self.release
			},
		);
	}

	pub fn set(&mut self, v: f32) {
		self.inner = v;
	}

	pub fn set_hard(&mut self, v: f32) {
		self.inner = v;
		self.value = v;
	}
}

impl Default for SmoothedEnv {
	fn default() -> Self {
		SmoothedEnv::new(0.0, 0.5, 0.1, 1.0)
	}
}
