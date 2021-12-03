pub const TWO_PI: f32 = 2f32 * std::f32::consts::PI;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StereoSample {
	pub l: f32,
	pub r: f32,
}

#[repr(C)]
#[derive(Debug)]
pub struct C_AudioBuffer {
	pub ptr: *mut f64,
	pub len: usize,
	pub cap: usize,
}

// pub type AudioBuffer = Vec<StereoSample>;
