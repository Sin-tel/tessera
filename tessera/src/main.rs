mod keycodes;
mod love_api;
mod love_config;
mod opengl;
mod text;

use femtovg::{Canvas, Color};
use mlua::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path;
use std::time::Instant;
use winit::keyboard::KeyCode;
use winit::{
	event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
	event_loop::EventLoop,
	window::Window,
};

use keycodes::keycode_to_love2d_key;
use love_api::Font;
use love_api::create_love_env;
use love_config::Config;
use opengl::Renderer;
use opengl::Surface;
use opengl::WindowSurface;
use opengl::start;
use text::TextEngine;

const FAST_UPDATE: bool = false;

struct Timer {
	start: Instant,
	last_update: Instant,
	last_update_fps: Instant,
	fps: f64,
	frames: usize,
}

impl Timer {
	fn new() -> Self {
		Self {
			start: std::time::Instant::now(),
			last_update: std::time::Instant::now(),
			last_update_fps: std::time::Instant::now(),
			fps: 0.,
			frames: 0,
		}
	}

	fn step(&mut self) -> f64 {
		self.frames += 1;
		let now = std::time::Instant::now();
		let dt = (now - self.last_update).as_secs_f64();
		self.last_update = now;

		let time_since_last = (now - self.last_update_fps).as_secs_f64();
		if time_since_last > 1.0 {
			self.fps = (self.frames as f64 / time_since_last).round();
			self.last_update_fps = now;
			self.frames = 0;
		};

		dt
	}

	fn get_time(&self) -> f64 {
		return (std::time::Instant::now() - self.start).as_secs_f64();
	}
}

pub struct State {
	current_color: Color,
	background_color: Color,
	line_width: f32,
	font: Font,
	text_engine: TextEngine,
	keys_down: HashSet<KeyCode>,
	mouse_down: HashSet<MouseButton>,
	mouse_position: (f32, f32),
	window_size: (u32, u32),
	exit: bool,
	timer: Timer,
	src_dir: String,
	transform_stack: Vec<femtovg::Transform2D>,
	current_scissor: Option<(f32, f32, f32, f32)>,
	canvas: Canvas<Renderer>,
	window: Window,
}

fn main() -> LuaResult<()> {
	// unsafe {
	//     std::env::set_var("RUST_BACKTRACE", "1");
	// }

	let src_dir = path::absolute("../lua").unwrap();

	let config = Config::read(src_dir)?;

	start(config);
	Ok(())
}

fn do_nothing(lua: &Lua) -> LuaFunction {
	lua.create_function(|_, ()| Ok(())).unwrap()
}

fn wrap_call<T: IntoLuaMulti>(lua_fn: &LuaFunction, args: T) {
	if let Err(e) = lua_fn.call::<()>(args) {
		// For now we just panic
		panic!("{}", e);
	}
}

fn run(
	canvas: Canvas<Renderer>,
	event_loop: EventLoop<()>,
	mut surface: Surface,
	window: Window,
	config: Config,
) -> LuaResult<()> {
	let lua = create_love_env()?;

	lua.set_app_data(State {
		current_color: Color::white(),
		background_color: Color::black(),
		keys_down: HashSet::new(),
		mouse_down: HashSet::new(),
		mouse_position: (0., 0.),
		window_size: (config.width, config.height),
		line_width: 1.5,
		font: Font { name: "Inter".to_string(), size: 14. },
		text_engine: TextEngine::new(),
		exit: false,
		timer: Timer::new(),
		src_dir: config.src_dir.display().to_string(),
		transform_stack: Vec::new(),
		current_scissor: None,
		canvas,
		window,
	});

	// set working directory so 'require' works
	std::env::set_current_dir(&config.src_dir).unwrap();

	let lua_main = fs::read_to_string(config.src_dir.join("main.lua")).unwrap();

	lua.load(lua_main).exec()?;

	// Get main callbacks
	let love: LuaTable = lua.globals().get("love")?;
	let love_load: LuaFunction = love.get("load").unwrap_or(do_nothing(&lua));
	let love_update: LuaFunction = love.get("update").unwrap_or(do_nothing(&lua));
	let love_draw: LuaFunction = love.get("draw").unwrap_or(do_nothing(&lua));
	let love_keypressed: LuaFunction = love.get("keypressed").unwrap_or(do_nothing(&lua));
	let love_keyreleased: LuaFunction = love.get("keyreleased").unwrap_or(do_nothing(&lua));
	let love_mousepressed: LuaFunction = love.get("mousepressed").unwrap_or(do_nothing(&lua));
	let love_mousereleased: LuaFunction = love.get("mousereleased").unwrap_or(do_nothing(&lua));
	let love_mousemoved: LuaFunction = love.get("mousemoved").unwrap_or(do_nothing(&lua));
	let love_wheelmoved: LuaFunction = love.get("wheelmoved").unwrap_or(do_nothing(&lua));
	let love_resize: LuaFunction = love.get("resize").unwrap_or(do_nothing(&lua));

	wrap_call(&love_load, ());

	event_loop
		.run(move |event, target| {
			target.set_control_flow(winit::event_loop::ControlFlow::Poll);

			match event {
				Event::WindowEvent { event, .. } => match event {
					WindowEvent::RedrawRequested => {
						if !FAST_UPDATE {
							let dt = step_timer(&lua);
							wrap_call(&love_update, dt);
						}
						render_start(&lua);
						wrap_call(&love_draw, ());
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
						let key = keycode_to_love2d_key(keycode);
						let mut app_state = lua.app_data_mut::<State>().unwrap();
						match state {
							ElementState::Pressed => {
								app_state.keys_down.insert(keycode);
								drop(app_state);
								wrap_call(&love_keypressed, (key, key, repeat))
							},
							ElementState::Released => {
								app_state.keys_down.remove(&keycode);
								drop(app_state);
								wrap_call(&love_keyreleased, (key, key, repeat))
							},
						};
					},
					WindowEvent::CursorMoved { position, .. } => {
						let mut app_state = lua.app_data_mut::<State>().unwrap();
						app_state.mouse_position = position.into();
						// love.mousemoved gets called in DeviceEvent::MouseMotion
					},
					WindowEvent::Resized(physical_size) => {
						let (w, h) = physical_size.into();
						surface.resize(w, h);

						// Minimizing window should not send (0, 0)
						if w > 0 && h > 0 {
							let mut app_state = lua.app_data_mut::<State>().unwrap();
							app_state.window_size = (w, h);
							drop(app_state);
							wrap_call(&love_resize, (w, h))
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
								let mut app_state = lua.app_data_mut::<State>().unwrap();
								app_state.mouse_down.insert(button);
								drop(app_state);

								wrap_call(&love_mousepressed, (x, y, button_number))
							},
							ElementState::Released => {
								let mut app_state = lua.app_data_mut::<State>().unwrap();
								app_state.mouse_down.remove(&button);
								drop(app_state);

								wrap_call(&love_mousereleased, (x, y, button_number))
							},
						};
					},
					WindowEvent::MouseWheel { delta, .. } => {
						let (x, y) = match delta {
							MouseScrollDelta::LineDelta(x, y) => (x, y),
							MouseScrollDelta::PixelDelta(d) => d.into(),
						};
						wrap_call(&love_wheelmoved, (x, y))
					},
					WindowEvent::CloseRequested => target.exit(),
					_ => {},
				},
				Event::DeviceEvent { event, .. } => match event {
					DeviceEvent::MouseMotion { delta } => {
						let (x, y) = lua.app_data_ref::<State>().unwrap().mouse_position;
						let (dx, dy) = delta;
						wrap_call(&love_mousemoved, (x, y, dx, dy));
					},
					_ => {},
				},
				Event::AboutToWait => {
					if FAST_UPDATE {
						let mut accum = 0.0;
						loop {
							let dt = step_timer(&lua);
							wrap_call(&love_update, dt);

							accum += dt;
							if accum >= 1.0 / 60.0 {
								break;
							}
							std::thread::sleep(std::time::Duration::from_micros(2000));
						}
					}

					let app_state = lua.app_data_mut::<State>().unwrap();
					if app_state.exit {
						target.exit()
					}

					app_state.window.request_redraw();
				},
				_ => {},
			}
		})
		.unwrap();

	Ok(())
}

fn step_timer(lua: &Lua) -> f64 {
	let mut state = lua.app_data_mut::<State>().unwrap();
	state.timer.step()
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
	assert!(state.transform_stack.len() == 0);

	state.canvas.reset_scissor();
	state.current_scissor = None;
	surface.present(&mut state.canvas);
}
