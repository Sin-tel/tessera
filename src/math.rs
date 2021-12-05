// /// Raw transmutation to `u32`.
// ///
// /// Transmutes the given `f32` into it's raw memory representation.
// /// Similar to `f32::to_bits` but even more raw.
// #[inline]
// fn to_bits(x: f32) -> u32 {
//     unsafe { ::std::mem::transmute::<f32, u32>(x) }
// }

// /// Raw transmutation from `u32`.
// ///
// /// Converts the given `u32` containing the float's raw memory representation into the `f32` type.
// /// Similar to `f32::from_bits` but even more raw.
// #[inline]
// fn from_bits(x: u32) -> f32 {
//     unsafe { ::std::mem::transmute::<u32, f32>(x) }
// }

// /// Raises 2 to a floating point power.
// #[inline]
// pub fn pow2(p: f32) -> f32 {
//     let clipp = if p < -126.0 { -126.0_f32 } else { p };
//     let v = ((1 << 23) as f32 * (clipp + 126.94269504_f32)) as u32;
//     from_bits(v)
// }

#[inline]
pub fn pitch_to_f(p: f32, sample_rate: f32) -> f32 {
	// pow2((p-49.0)/12.0) * 440.0 / sample_rate
	(2.0_f32).powf((p - 49.0) / 12.0) * 440.0 / sample_rate
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
	a * (1.0 - t) + b * t
}

#[derive(Debug)]
pub struct Smoothed {
	pub value: f32,
	inner: f32,
	f: f32,
}

impl Smoothed {
	pub fn new(value: f32, f: f32) -> Self {
		Smoothed {
			inner: value,
			value,
			f,
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
		Smoothed {
			inner: 0.0,
			value: 0.0,
			f: 0.001,
		}
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
	pub fn new(value: f32, attack: f32, release: f32) -> Self {
		SmoothedEnv {
			inner: value,
			value,
			attack,
			release,
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
		SmoothedEnv {
			inner: 0.0,
			value: 0.0,
			attack: 0.002,
			release: 0.0005,
		}
	}
}
