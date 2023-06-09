pub mod delayline;
pub mod env;
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

pub fn softclip(x: f32) -> f32 {
	let s = x.clamp(-3.0, 3.0);
	s * (27.0 + s * s) / (27.0 + 9.0 * s * s)
}

pub fn softclip_cubic(x: f32) -> f32 {
	let s = x.clamp(-1.5, 1.5);
	s * (1.0 - (4. / 27.) * s * s)
}
