use std::f32::consts::PI;

pub mod delayline;
pub mod env;
pub mod simper;
pub mod skf;

pub const TWO_PI: f32 = std::f32::consts::TAU;

const C5_HZ: f32 = 523.2511;

// pitch in semitones to Hz
// tuning to C5 = 523.2511 instead of A4 = 440
pub fn pitch_to_hz(p: f32) -> f32 {
	(2.0_f32).powf((p - 72.0) / 12.0) * C5_HZ
}

// inverse of above
pub fn hz_to_pitch(f: f32) -> f32 {
	12.0 * (f / C5_HZ).log2() + 72.0
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
	a + (b - a) * t
}

pub fn from_db(x: f32) -> f32 {
	(10.0f32).powf(x / 20.0)
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

#[derive(Debug, Default)]
pub struct DcKiller {
	z: f32,
}

impl DcKiller {
	// TODO: add new with sample_rate
	pub fn process(&mut self, s: f32) -> f32 {
		// 10Hz at 44.1kHz sample rate
		self.z += (s - self.z) * 0.0014;

		s - self.z
	}
}
