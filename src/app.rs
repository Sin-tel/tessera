use crate::audio::AUDIO_PANIC;
use crate::context::AudioContext;
use crate::log::log_warn;
use crate::text::Font;
use femtovg::{Canvas, Color};
use std::sync::atomic::Ordering;
use std::time::Instant;
use winit::window::Window;

use crate::opengl::Renderer;
use crate::text::TextEngine;

pub const INIT_WIDTH: u32 = 1280;
pub const INIT_HEIGHT: u32 = 720;

pub struct State {
	pub current_color: Color,
	pub line_width: f32,
	pub font: Font,
	pub font_size: f32,
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
	pub fn new(canvas: Canvas<Renderer>, window: Window) -> Self {
		State {
			current_color: Color::white(),
			mouse_position: (0., 0.),
			window_size: (INIT_WIDTH, INIT_HEIGHT),
			line_width: 1.5,
			font: Font::Inter,
			font_size: 14.,
			text_engine: TextEngine::new(),
			exit: false,
			start_time: std::time::Instant::now(),
			transform_stack: Vec::new(),
			current_scissor: None,
			audio: None,
			canvas,
			window,
		}
	}

	pub fn check_audio_status(&mut self) {
		if self.audio.is_some() && AUDIO_PANIC.load(Ordering::Relaxed) {
			log_warn!("Killing backend!");
			AUDIO_PANIC.store(false, Ordering::Relaxed);
			self.audio = None;
		}
	}
}
