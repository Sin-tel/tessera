use std::f32::consts::PI;

pub mod delayline;
pub mod env;
pub mod onepole;
pub mod resample;
pub mod simper;
pub mod skf;
pub mod smooth;

pub const TWO_PI: f32 = std::f32::consts::TAU;

// reference frequency
// we use C5 instead of A4 = 440
pub const C5_HZ: f32 = 523.2511;

// log(10) / (20 * log(2))
pub const DECIBEL_FACTOR: f32 = 0.1660964;

// https://stackoverflow.com/questions/65554112/fast-double-exp2-function-in-c
pub fn pow2_cheap(x: f32) -> f32 {
	const A: f32 = (1 << 23) as f32;
	let w = x.floor();
	let z = x - w;

	// rational pow2(x)-1 approximation in [0, 1], order (2,1)
	let approx = -5.725942 - 0.49013227 * z + 27.728334 / (4.842578 - z);

	let resi = (A * (w + 127.0 + approx)) as u32;
	f32::from_bits(resi)
}

pub fn log2_cheap(x: f32) -> f32 {
	const MASK: u32 = !(255 << 23);
	const A: u32 = 127 << 23;

	let u = x.to_bits();
	let w = ((u as i32 >> 23) - 128) as f32;

	let z = f32::from_bits((u & MASK) + A);

	// TODO: optimize minimax error
	// rational log2(x)+1 approximation in [1, 2], order (2,1)
	let approx = 2.848016 + 0.240372 * z - 3.65164 / (0.74883 + z);

	w + approx
}

// pitch in semitones to Hz
// tuning to C5 = 523.2511 instead of A4 = 440
pub fn pitch_to_hz(p: f32) -> f32 {
	// 2.0_f32.powf((p - 72.0) / 12.0). * C5_HZ
	pow2_cheap((p - 72.0) / 12.0) * C5_HZ
}

// inverse of above
pub fn hz_to_pitch(f: f32) -> f32 {
	// 12.0 * (f / C5_HZ).log2() + 72.0
	12.0 * log2_cheap(f / C5_HZ) + 72.0
}

#[inline(always)]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
	a + (b - a) * t
}

// TODO: check if these two correctly handle -Inf and 0 respectively
pub fn from_db(x: f32) -> f32 {
	// (10.0f32).powf(x / 20.0)
	pow2_cheap(x * DECIBEL_FACTOR)
}

pub fn to_db(x: f32) -> f32 {
	log2_cheap(x) / DECIBEL_FACTOR
}

// Cheap tanh-ish distortion
pub fn softclip(x: f32) -> f32 {
	let s = x.clamp(-3.0, 3.0);
	s * (27.0 + s * s) / (27.0 + 9.0 * s * s)
}

pub fn softclip_cubic(x: f32) -> f32 {
	let s = x.clamp(-1.5, 1.5);
	s * (1.0 - (4. / 27.) * s * s)
}

// branchless approximation of sin(2*pi*x)
pub fn sin_cheap(x: f32) -> f32 {
	let x = x - x.floor();
	let a = f32::from(x > 0.5);
	let b = 2.0 * x - 1.0 - 2.0 * a;
	(2.0 * a - 1.0) * (x * b + a) / (0.25 * x * b + 0.15625 + 0.25 * a)
	// (TWO_PI * x).sin()
}

pub fn make_usize_frac(x: f32) -> (usize, f32) {
	let x_int = x.floor();
	let x_frac = x - x_int;

	(x_int as usize, x_frac)
}

pub fn make_isize_frac(x: f32) -> (isize, f32) {
	let x_int = x.floor();
	let x_frac = x - x_int;

	(x_int as isize, x_frac)
}

// Padé approximant of tan(pi*x)
// Less than 1c error < 13kHz
pub fn prewarp(f: f32) -> f32 {
	let x = f.min(0.49);
	let a = x * x;
	x * (PI.powi(3) * a - 15.0 * PI) / (6.0 * PI.powi(2) * a - 15.0)
}

// milliseconds (time to reach 10^-2) to time constant
pub fn time_constant(t: f32, sample_rate: f32) -> f32 {
	// - 1000 * ln(0.01) / ln(2)
	const T_LOG2: f32 = 6643.856;
	// - 1000 * ln(0.01)
	const T_LN: f32 = 4605.1704;

	assert!(t > 0.);

	let denom = sample_rate * t;
	if denom < 1000. {
		1.0 - pow2_cheap(-T_LOG2 / denom)
	} else {
		// 1 - exp(-x) ~ x for small values
		T_LN / denom
	}
}

pub fn time_constant_linear(t: f32, sample_rate: f32) -> f32 {
	assert!(t > 0.);
	1000. / (sample_rate * t)
}

// This is a 'naive' implementation of a one-pole highpass at 10Hz
// Since cutoff << sample rate it works fine
#[derive(Debug)]
pub struct DcKiller {
	z: f32,
	f: f32,
}

impl DcKiller {
	pub fn new(sample_rate: f32) -> Self {
		Self {
			z: 0.,

			// approximation:
			// 1 - exp( -2*pi*10 / f_s) ~ 2*pi*10 / f_s
			f: TWO_PI * 10. / sample_rate,
		}
	}
	pub fn process(&mut self, s: f32) -> f32 {
		let y_hp = s - self.z;
		self.z += y_hp * self.f;
		y_hp
	}
}
