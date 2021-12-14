// use std::ops::AddAssign;
// use std::ops::Add;

pub const TWO_PI: f32 = 2f32 * std::f32::consts::PI;
pub const MAX_BUF_SIZE: usize = 1024;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct StereoSample {
	pub l: f32,
	pub r: f32,
}

// impl AddAssign for StereoSample {
//     fn add_assign(&mut self, other: Self) {
//         *self = Self {
//             l: self.l + other.l,
//             r: self.r + other.r,
//         };
//     }
// }

// impl Add for StereoSample {
//     type Output = Self;

//     fn add(self, other: Self) -> Self {
//         Self {
//             l: self.l + other.l,
//             r: self.r + other.r,
//         }
//     }
// }


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
	NoteOn(usize, CV),
	SetParam(usize, usize, usize, f32),
	Pan(usize, f32, f32),
}

#[derive(Debug)]
pub struct CV {
	pub pitch: f32,
	pub vel: f32,
}

#[repr(C)]
#[derive(Debug)]
pub enum LuaMessage {
	Test(),
	Cpu(f32),
	Meter(f32, f32),
}