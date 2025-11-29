#![windows_subsystem = "windows"]

use femtovg::Color;
use mlua::prelude::*;
use std::fs;
use tessera::app::State;
use tessera::app::{INIT_HEIGHT, INIT_WIDTH};
use tessera::log::init_logging;
use winit::application::ApplicationHandler;
use winit::event::DeviceId;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use tessera::api::create_lua;
use tessera::api::graphics::Font;
use tessera::api::keycodes::keycode_to_str;
use tessera::opengl::Surface;
use tessera::opengl::WindowSurface;
use tessera::opengl::setup_window;
use tessera::text::TextEngine;

fn wrap_call<T: IntoLuaMulti>(lua_fn: &LuaFunction, args: T) {
	if let Err(e) = lua_fn.call::<()>(args) {
		// For now we just panic
		panic!("{e}");
		// log_error!("{e}");
		// println!("{e}");
	}
}

fn main() -> LuaResult<()> {
	let (canvas, event_loop, surface, window) = setup_window();

	let lua = create_lua()?;
	lua.set_app_data(State {
		current_color: Color::white(),
		background_color: Color::black(),
		mouse_position: (0., 0.),
		window_size: (INIT_WIDTH, INIT_HEIGHT),
		line_width: 1.5,
		font: Font { name: "Inter".to_string(), size: 14. },
		text_engine: TextEngine::new(),
		exit: false,
		start_time: std::time::Instant::now(),
		transform_stack: Vec::new(),
		current_scissor: None,
		audio: None,
		canvas,
		window,
	});

	// set working directory so 'require' works
	lua.load("package.path = package.path .. ';lua/?.lua;'").exec()?;

	init_logging();

	let lua_main = fs::read_to_string("lua/main.lua").unwrap();
	lua.load(lua_main).set_name("@lua/main.lua").exec()?;

	let tessera: LuaTable = lua.globals().get("tessera")?;
	let load: LuaFunction = tessera.get("load").unwrap();
	let update: LuaFunction = tessera.get("update").unwrap();
	let draw: LuaFunction = tessera.get("draw").unwrap();
	let keypressed: LuaFunction = tessera.get("keypressed").unwrap();
	let keyreleased: LuaFunction = tessera.get("keyreleased").unwrap();
	let mousepressed: LuaFunction = tessera.get("mousepressed").unwrap();
	let mousereleased: LuaFunction = tessera.get("mousereleased").unwrap();
	let mousemoved: LuaFunction = tessera.get("mousemoved").unwrap();
	let wheelmoved: LuaFunction = tessera.get("wheelmoved").unwrap();
	let resize: LuaFunction = tessera.get("resize").unwrap();
	let quit: LuaFunction = tessera.get("quit").unwrap();

	wrap_call(&load, ());

	let last_update = std::time::Instant::now();
	let mut app = App {
		lua,
		surface,
		last_update,
		update,
		draw,
		keypressed,
		keyreleased,
		mousepressed,
		mousereleased,
		mousemoved,
		wheelmoved,
		resize,
		quit,
	};

	event_loop.run_app(&mut app).unwrap();
	Ok(())
}

struct App {
	lua: Lua,
	surface: Surface,
	last_update: std::time::Instant,
	update: LuaFunction,
	draw: LuaFunction,
	keypressed: LuaFunction,
	keyreleased: LuaFunction,
	mousepressed: LuaFunction,
	mousereleased: LuaFunction,
	mousemoved: LuaFunction,
	wheelmoved: LuaFunction,
	resize: LuaFunction,
	quit: LuaFunction,
}

impl ApplicationHandler for App {
	fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
		// Only show window after initializing state to prevent blank screen
		let app_state = self.lua.app_data_mut::<State>().unwrap();
		app_state.window.set_visible(true);
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		_window_id: WindowId,
		event: WindowEvent,
	) {
		match event {
			WindowEvent::RedrawRequested => {
				render_start(&self.lua);
				wrap_call(&self.draw, ());
				render_end(&self.surface, &self.lua);
			},
			WindowEvent::KeyboardInput {
				event:
					winit::event::KeyEvent {
						physical_key: winit::keyboard::PhysicalKey::Code(keycode),
						state,
						repeat,
						..
					},
				..
			} => {
				let key = keycode_to_str(keycode);
				match state {
					ElementState::Pressed => {
						wrap_call(&self.keypressed, (key, key, repeat));
					},
					ElementState::Released => {
						wrap_call(&self.keyreleased, (key, key, repeat));
					},
				}
			},
			WindowEvent::CursorMoved { position, .. } => {
				let mut app_state = self.lua.app_data_mut::<State>().unwrap();
				app_state.mouse_position = position.into();
				// tessera.mousemoved gets called in DeviceEvent::MouseMotion
			},
			WindowEvent::Resized(physical_size) => {
				let (w, h) = physical_size.into();
				self.surface.resize(w, h);

				// Minimizing window should not send (0, 0)
				if w > 0 && h > 0 {
					let mut app_state = self.lua.app_data_mut::<State>().unwrap();
					app_state.window_size = (w, h);
					drop(app_state);
					wrap_call(&self.resize, (w, h));
				}
			},
			WindowEvent::MouseInput { state, button, .. } => {
				let button_number = match button {
					MouseButton::Left => 1,
					MouseButton::Right => 2,
					MouseButton::Middle => 3,
					MouseButton::Back => 4,
					MouseButton::Forward => 5,
					MouseButton::Other(i) => i,
				};
				let (x, y) = self.lua.app_data_ref::<State>().unwrap().mouse_position;
				match state {
					ElementState::Pressed => {
						wrap_call(&self.mousepressed, (x, y, button_number));
					},
					ElementState::Released => {
						wrap_call(&self.mousereleased, (x, y, button_number));
					},
				}
			},
			WindowEvent::MouseWheel { delta, .. } => {
				let (x, y) = match delta {
					MouseScrollDelta::LineDelta(x, y) => (x, y),
					MouseScrollDelta::PixelDelta(d) => d.into(),
				};
				wrap_call(&self.wheelmoved, (x, y));
			},
			WindowEvent::CloseRequested => event_loop.exit(),
			_ => {},
		}
	}

	fn device_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_device_id: DeviceId,
		event: DeviceEvent,
	) {
		match event {
			DeviceEvent::MouseMotion { delta } => {
				let (x, y) = self.lua.app_data_ref::<State>().unwrap().mouse_position;
				let (dx, dy) = delta;
				wrap_call(&self.mousemoved, (x, y, dx, dy));
			},
			_ => {},
		}
	}

	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		let mut accum = 0.0;
		loop {
			self.lua.app_data_mut::<State>().unwrap().check_audio_status();

			let now = std::time::Instant::now();
			let dt = (now - self.last_update).as_secs_f64();
			self.last_update = now;

			wrap_call(&self.update, dt);

			accum += dt;
			if accum >= 1.0 / 60.0 {
				break;
			}
			std::thread::sleep(std::time::Duration::from_micros(2000));
		}

		let app_state = self.lua.app_data_mut::<State>().unwrap();
		if app_state.exit {
			event_loop.exit();
		}

		app_state.window.request_redraw();

		// if self.request_redraw && !self.wait_cancelled && !self.close_requested {
		// 	self.window.as_ref().unwrap().request_redraw();
		// }
	}

	fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
		wrap_call(&self.quit, ());
	}
}

fn render_start(lua: &Lua) {
	let mut state = lua.app_data_mut::<State>().unwrap();

	state.canvas.reset_transform();
	state.transform_stack.clear();

	// Make sure the canvas has the right size:
	let size = state.window.inner_size();
	let scale_factor = state.window.scale_factor() as f32;
	state.canvas.set_size(size.width, size.height, scale_factor);

	let bg_color = state.background_color;
	state.canvas.clear_rect(0, 0, size.width, size.height, bg_color);
}

fn render_end(surface: &Surface, lua: &Lua) {
	let mut state = lua.app_data_mut::<State>().unwrap();
	assert!(state.transform_stack.is_empty());

	state.canvas.reset_scissor();
	state.current_scissor = None;

	state.window.pre_present_notify();
	surface.present(&mut state.canvas);
}
