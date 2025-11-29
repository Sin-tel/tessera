use crate::context::AudioContext;
use crate::log::log_error;
use femtovg::{Canvas, Color};
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
	pub fn audio_mut(&mut self) -> Option<&mut AudioContext> {
		if let Some(ud) = &self.audio
			&& ud.m_render.is_poisoned()
		{
			log_error!("Lock was poisoned. Killing backend.");
			self.audio = None;
		}
		self.audio.as_mut()
	}
}
