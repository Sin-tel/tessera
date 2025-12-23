#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use femtovg::Color;
use mlua::prelude::*;
use std::error::Error;
use std::fs;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::DeviceId;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::WindowId;

use tessera::api::Hooks;
use tessera::api::create_lua;
use tessera::api::icon::load_icons;
use tessera::api::image::load_images;
use tessera::api::keycodes::keycode_to_str;
use tessera::app::State;
use tessera::audio;
use tessera::context::LuaMessage;
use tessera::embed::Script;
use tessera::log::*;
use tessera::midi;
use tessera::opengl::Surface;
use tessera::opengl::WindowSurface;
use tessera::opengl::setup_window;

fn wrap_call<T: IntoLuaMulti>(status: &mut Status, lua_fn: &LuaFunction, args: T) {
	if let Status::Error(_) = status {
		return;
	}
	if let Err(e) = lua_fn.call::<()>(args) {
		log_error!("{e}");
		*status = Status::Error(e.to_string())
	} else {
		*status = Status::Running
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	// Make sure output folder exists before we do anything
	fs::create_dir_all("./out")?;

	if std::env::args().any(|arg| arg == "--test-run") {
		test_run()?;
		return Ok(());
	}

	if let Err(e) = run() {
		log_error!("{e}");
	}
	Ok(())
}

fn do_main(lua: &Lua) -> Result<(), Box<dyn Error>> {
	let lua_main = &*Script::get("main.lua").ok_or("main.lua not found.")?.data;
	lua.load(lua_main).set_name("@lua/main.lua").exec()?;

	Ok(())
}

fn test_run() -> Result<(), Box<dyn Error>> {
	// Basic checks for CI without initializing any graphics or audio

	let (lua_tx, _lua_rx) = mpsc::sync_channel::<LuaMessage>(256);
	init_logging(lua_tx.clone());

	let lua = create_lua(1.0)?;

	do_main(&lua)?;

	let hooks = Hooks::new(&lua)?;

	let hosts = audio::get_hosts();
	log_info!("Available hosts: {:?}", hosts);

	for host in hosts {
		if let Ok(devices) = audio::get_output_devices(&host) {
			log_info!("Available devices ({}): {:?}", host, devices);
		} else {
			log_warn!("No devices");
		}
	}

	if let Some(midi_input) = midi::open_midi() {
		log_info!("Available midi ports: {:?}", midi::port_names(&midi_input));
	} else {
		log_warn!("Midi failed to init");
	}

	hooks.load.call::<()>(true)?;
	hooks.quit.call::<()>(())?;
	Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
	let (lua_tx, lua_rx) = mpsc::sync_channel::<LuaMessage>(256);
	init_logging(lua_tx.clone());

	let (canvas, event_loop, surface, window) = setup_window();

	// We check scale factor before loading lua, and assume it doesn't change for simplicity
	let scale_factor = window.scale_factor();

	let lua = create_lua(scale_factor)?;
	let state = State::new(canvas, window, lua_tx, lua_rx, scale_factor as f32);

	lua.set_app_data(state);

	#[cfg(not(debug_assertions))]
	tessera::app::spawn_update_check();

	do_main(&lua)?;
	load_images(&lua)?;
	load_icons(&lua)?;

	let hooks = Hooks::new(&lua)?;

	let mut app = App::new(lua, surface, hooks);

	wrap_call(&mut app.status, &app.hooks.load, ());

	if let Err(e) = event_loop.run_app(&mut app) {
		log_error!("{e}");
	}
	Ok(())
}

#[derive(Debug, Clone)]
enum Status {
	Running,
	Error(String),
}

struct App {
	lua: Lua,
	surface: Surface,
	last_update: Instant,
	hooks: Hooks,
	status: Status,
}

impl App {
	pub fn new(lua: Lua, surface: Surface, hooks: Hooks) -> Self {
		let last_update = Instant::now();
		Self { lua, surface, last_update, hooks, status: Status::Running }
	}
}

impl ApplicationHandler for App {
	fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
		// Only show window after initializing state to prevent blank screen
		let app_state = self.lua.app_data_mut::<State>().unwrap();
		#[cfg(not(debug_assertions))]
		app_state.window.set_maximized(true);
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
				match &self.status {
					Status::Running => {
						wrap_call(&mut self.status, &self.hooks.draw, ());
					},
					Status::Error(msg) => {
						if let Err(e) = self.hooks.draw_error.call::<()>(msg.clone()) {
							// If we can't even display the error message then just panic
							log_error!("{e}");
							panic!("Error in draw_err");
						}
					},
				}
				render_end(&self.surface, &self.lua);
			},
			WindowEvent::KeyboardInput {
				event:
					winit::event::KeyEvent {
						physical_key: PhysicalKey::Code(keycode),
						logical_key,
						state,
						repeat,
						..
					},
				..
			} => {
				let key = keycode_to_str(keycode);
				let key_str: Option<String> = match logical_key {
					Key::Character(s) => Some(s.into()),
					Key::Named(NamedKey::Space) => Some(" ".to_string()),
					_ => None,
				};

				match state {
					ElementState::Pressed => {
						if let Status::Error(_) = self.status
							&& keycode == KeyCode::Escape
						{
							event_loop.exit();
						}
						wrap_call(&mut self.status, &self.hooks.keypressed, (key, key_str, repeat));
					},
					ElementState::Released => {
						wrap_call(
							&mut self.status,
							&self.hooks.keyreleased,
							(key, key_str, repeat),
						);
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
					wrap_call(&mut self.status, &self.hooks.resize, (w, h));
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
						wrap_call(
							&mut self.status,
							&self.hooks.mousepressed,
							(x, y, button_number),
						);
					},
					ElementState::Released => {
						wrap_call(
							&mut self.status,
							&self.hooks.mousereleased,
							(x, y, button_number),
						);
					},
				}
			},
			WindowEvent::MouseWheel { delta, .. } => {
				let (x, y) = match delta {
					MouseScrollDelta::LineDelta(x, y) => (x, y),
					MouseScrollDelta::PixelDelta(d) => (d.x as f32 / 14., d.y as f32 / 14.),
				};
				wrap_call(&mut self.status, &self.hooks.wheelmoved, (x, y));
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
		#[allow(clippy::single_match)]
		match event {
			DeviceEvent::MouseMotion { delta } => {
				let (x, y) = self.lua.app_data_ref::<State>().unwrap().mouse_position;
				let (dx, dy) = delta;
				wrap_call(&mut self.status, &self.hooks.mousemoved, (x, y, dx, dy));
			},
			_ => {},
		}
	}

	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		let start = Instant::now();
		loop {
			self.lua.app_data_mut::<State>().unwrap().check_audio_status();

			let now = Instant::now();
			let dt = (now - self.last_update).as_secs_f64();
			self.last_update = now;

			wrap_call(&mut self.status, &self.hooks.update, dt);

			let accum = (Instant::now() - start).as_secs_f64() + 0.004;
			if accum >= 1.0 / 60.0 {
				break;
			}
			std::thread::sleep(Duration::from_micros(2000));
		}

		let app_state = self.lua.app_data_mut::<State>().unwrap();
		if app_state.exit {
			event_loop.exit();
		}

		app_state.window.request_redraw();
	}

	fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
		wrap_call(&mut self.status, &self.hooks.quit, ());
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

	state.canvas.clear_rect(0, 0, size.width, size.height, Color::black());
}

fn render_end(surface: &Surface, lua: &Lua) {
	let mut state = lua.app_data_mut::<State>().unwrap();
	if !state.transform_stack.is_empty() {
		log_error!("Transform stack should be empty.");
	}

	state.canvas.reset_scissor();
	state.current_scissor = None;

	state.window.pre_present_notify();
	surface.present(&mut state.canvas);
}
