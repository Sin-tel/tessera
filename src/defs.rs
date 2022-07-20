pub const TWO_PI: f32 = std::f32::consts::TAU;
pub const MAX_BUF_SIZE: usize = 1024;

pub type Stereo = [f32; 2];

#[repr(C)]
#[derive(Debug)]
pub struct C_AudioBuffer {
	pub ptr: *mut f64,
	pub len: usize,
	pub cap: usize,
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
