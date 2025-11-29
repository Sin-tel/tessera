#![windows_subsystem = "windows"]

use femtovg::{Canvas, Color};
use mlua::prelude::*;
use std::fs;
use tessera::app::State;
use tessera::app::{INIT_HEIGHT, INIT_WIDTH};
use tessera::log::init_logging;
use winit::{
	event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
	event_loop::EventLoop,
	window::Window,
};

use tessera::api::create_lua;
use tessera::api::graphics::Font;
use tessera::api::keycodes::keycode_to_str;
use tessera::opengl::Renderer;
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

fn main() {
	let (canvas, event_loop, demo_surface, window) = setup_window();

	if let Err(e) = run(canvas, event_loop, demo_surface, window) {
		println!("{e}");
	}
}

fn run(
	canvas: Canvas<Renderer>,
	event_loop: EventLoop<()>,
	mut surface: Surface,
	window: Window,
) -> LuaResult<()> {
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

	// Get main callbacks
	let tessera: LuaTable = lua.globals().get("tessera")?;
	let tessera_load: LuaFunction = tessera.get("load").unwrap();
	let tessera_update: LuaFunction = tessera.get("update").unwrap();
	let tessera_draw: LuaFunction = tessera.get("draw").unwrap();
	let tessera_keypressed: LuaFunction = tessera.get("keypressed").unwrap();
	let tessera_keyreleased: LuaFunction = tessera.get("keyreleased").unwrap();
	let tessera_mousepressed: LuaFunction = tessera.get("mousepressed").unwrap();
	let tessera_mousereleased: LuaFunction = tessera.get("mousereleased").unwrap();
	let tessera_mousemoved: LuaFunction = tessera.get("mousemoved").unwrap();
	let tessera_wheelmoved: LuaFunction = tessera.get("wheelmoved").unwrap();
	let tessera_resize: LuaFunction = tessera.get("resize").unwrap();
	let tessera_quit: LuaFunction = tessera.get("quit").unwrap();

	let _start = std::time::Instant::now();
	let mut last_update = std::time::Instant::now();

	wrap_call(&tessera_load, ());

	event_loop
		.run(move |event, target| {
			target.set_control_flow(winit::event_loop::ControlFlow::Poll);

			match event {
				Event::WindowEvent { event, .. } => match event {
					WindowEvent::RedrawRequested => {
						render_start(&lua);
						wrap_call(&tessera_draw, ());
						render_end(&surface, &lua);
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
								wrap_call(&tessera_keypressed, (key, key, repeat));
							},
							ElementState::Released => {
								wrap_call(&tessera_keyreleased, (key, key, repeat));
							},
						}
					},
					WindowEvent::CursorMoved { position, .. } => {
						let mut app_state = lua.app_data_mut::<State>().unwrap();
						app_state.mouse_position = position.into();
						// tessera.mousemoved gets called in DeviceEvent::MouseMotion
					},
					WindowEvent::Resized(physical_size) => {
						let (w, h) = physical_size.into();
						surface.resize(w, h);

						// Minimizing window should not send (0, 0)
						if w > 0 && h > 0 {
							let mut app_state = lua.app_data_mut::<State>().unwrap();
							app_state.window_size = (w, h);
							drop(app_state);
							wrap_call(&tessera_resize, (w, h));
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
						let (x, y) = lua.app_data_ref::<State>().unwrap().mouse_position;
						match state {
							ElementState::Pressed => {
								wrap_call(&tessera_mousepressed, (x, y, button_number));
							},
							ElementState::Released => {
								wrap_call(&tessera_mousereleased, (x, y, button_number));
							},
						}
					},
					WindowEvent::MouseWheel { delta, .. } => {
						let (x, y) = match delta {
							MouseScrollDelta::LineDelta(x, y) => (x, y),
							MouseScrollDelta::PixelDelta(d) => d.into(),
						};
						wrap_call(&tessera_wheelmoved, (x, y));
					},
					WindowEvent::CloseRequested => target.exit(),
					_ => {},
				},
				Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
					let (x, y) = lua.app_data_ref::<State>().unwrap().mouse_position;
					let (dx, dy) = delta;
					wrap_call(&tessera_mousemoved, (x, y, dx, dy));
				},
				Event::AboutToWait => {
					let mut accum = 0.0;
					loop {
						let now = std::time::Instant::now();
						let dt = (now - last_update).as_secs_f64();
						last_update = now;

						wrap_call(&tessera_update, dt);

						accum += dt;
						if accum >= 1.0 / 60.0 {
							break;
						}
						std::thread::sleep(std::time::Duration::from_micros(2000));
					}

					let app_state = lua.app_data_mut::<State>().unwrap();
					if app_state.exit {
						target.exit();
					}

					app_state.window.request_redraw();
				},
				Event::LoopExiting => {
					wrap_call(&tessera_quit, ());
				},
				_ => {},
			}
		})
		.unwrap();

	Ok(())
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
	surface.present(&mut state.canvas);
}
