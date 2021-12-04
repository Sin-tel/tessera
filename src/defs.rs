pub const TWO_PI: f32 = 2f32 * std::f32::consts::PI;
pub const MAX_BUF_SIZE: usize = 1024;

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

// Message struct to pass to audio thread
// Should not contain any boxed values (for now)
#[derive(Debug)]
pub enum AudioMessage {
	CV(usize, CV),
	Add,
	SetParam(usize, usize, f32),
}

#[derive(Debug)]
pub struct CV {
	pub freq: f32,
	pub vol: f32,
}

#[derive(Debug)]
pub enum LuaMessage {
	Test(f32),
}
