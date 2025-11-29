use crate::audio::AUDIO_PANIC;
use crate::context::AudioContext;
use crate::log::log_warn;
use femtovg::{Canvas, Color};
use std::sync::atomic::Ordering;
use std::time::Instant;
use winit::window::Window;

use crate::api::graphics::Font;
use crate::opengl::Renderer;
use crate::text::TextEngine;

pub const INIT_WIDTH: u32 = 1280;
pub const INIT_HEIGHT: u32 = 720;

pub struct State {
	pub current_color: Color,
	pub background_color: Color,
	pub line_width: f32,
	pub font: Font,
	pub text_engine: TextEngine,
	pub mouse_position: (f32, f32),
	pub window_size: (u32, u32),
	pub exit: bool,
	pub start_time: Instant,
	pub transform_stack: Vec<femtovg::Transform2D>,
	pub current_scissor: Option<(f32, f32, f32, f32)>,
	pub audio: Option<AudioContext>,
	pub canvas: Canvas<Renderer>,
	pub window: Window,
}

impl State {
	pub fn check_audio_status(&mut self) {
		if self.audio.is_some() && AUDIO_PANIC.load(Ordering::Relaxed) {
			log_warn!("Killing backend!");
			AUDIO_PANIC.store(false, Ordering::Relaxed);
			self.audio = None;
		}
	}
}
