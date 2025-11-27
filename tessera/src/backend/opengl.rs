use crate::Config;
use std::num::NonZeroU32;

use super::{WindowSurface, run};

use femtovg::{Canvas, renderer::OpenGl};
use glutin::{
	config::ConfigTemplateBuilder,
	context::{ContextApi, ContextAttributesBuilder},
	display::GetGlDisplay,
	prelude::*,
	surface::SurfaceAttributesBuilder,
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use winit::{event_loop::EventLoop, window::WindowBuilder};

pub struct Surface {
	context: glutin::context::PossiblyCurrentContext,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl WindowSurface for Surface {
	type Renderer = OpenGl;

	fn resize(&mut self, width: u32, height: u32) {
		if width > 0 && height > 0 {
			self.surface.resize(
				&self.context,
				width.try_into().unwrap(),
				height.try_into().unwrap(),
			);
		}
	}
	fn present(&self, canvas: &mut femtovg::Canvas<Self::Renderer>) {
		canvas.flush_to_surface(&());
		self.surface.swap_buffers(&self.context).unwrap();
	}
}

pub fn start_opengl(config: Config) {
	let event_loop = EventLoop::new().unwrap();

	let (canvas, window, context, surface) = {
		let window_builder = WindowBuilder::new()
			.with_inner_size(winit::dpi::PhysicalSize::new(config.width, config.height))
			.with_resizable(config.resizeable)
			.with_title(config.title.clone());

		let template = ConfigTemplateBuilder::new().with_alpha_size(8);

		let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

		let (window, gl_config) = display_builder
			.build(&event_loop, template, |configs| {
				// Find the config with the maximum number of samples, so our triangle will
				// be smooth.
				configs
					.reduce(|accum, config| {
						let transparency_check = config.supports_transparency().unwrap_or(false)
							& !accum.supports_transparency().unwrap_or(false);

						if transparency_check || config.num_samples() < accum.num_samples() {
							config
						} else {
							accum
						}
					})
					.unwrap()
			})
			.unwrap();

		let window = window.unwrap();

		let raw_window_handle = Some(window.raw_window_handle());

		let gl_display = gl_config.display();

		let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);
		let fallback_context_attributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::Gles(None))
			.build(raw_window_handle);
		let mut not_current_gl_context = Some(unsafe {
			gl_display
				.create_context(&gl_config, &context_attributes)
				.unwrap_or_else(|_| {
					gl_display
						.create_context(&gl_config, &fallback_context_attributes)
						.expect("failed to create context")
				})
		});

		let (width, height): (u32, u32) = window.inner_size().into();
		let raw_window_handle = window.raw_window_handle();
		let attrs = SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
			raw_window_handle,
			NonZeroU32::new(width).unwrap(),
			NonZeroU32::new(height).unwrap(),
		);

		let surface =
			unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };

		let gl_context = not_current_gl_context.take().unwrap().make_current(&surface).unwrap();

		let renderer =
			unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s).cast()) }
				.expect("Cannot create renderer");

		let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
		canvas.set_size(width, height, window.scale_factor() as f32);

		(canvas, window, gl_context, surface)
	};

	let demo_surface = Surface { context, surface };

	println!("Running openGL backend.");
	if let Err(e) = run(canvas, event_loop, demo_surface, window, config) {
		println!("{}", e);
	}
}
