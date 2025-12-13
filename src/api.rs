mod audio;
pub mod graphics;
pub mod icon;
pub mod image;
pub mod keycodes;
mod midi;
mod mouse;
pub mod project;

use crate::app::{State, get_version};
use crate::embed::setup_lua_loader;
use crate::log::log_warn;
use mlua::prelude::*;
use std::sync::mpsc;

pub fn create_lua(scale_factor: f64) -> LuaResult<Lua> {
	#[cfg(debug_assertions)]
	let lua = unsafe { Lua::unsafe_new() };

	#[cfg(not(debug_assertions))]
	let lua = Lua::new();

	setup_lua_loader(&lua)?;

	// main tessera table
	let tessera = lua.create_table()?;

	// tessera.graphics
	tessera.set("graphics", graphics::create(&lua, scale_factor)?)?;

	// tessera.mouse
	tessera.set("mouse", mouse::create(&lua)?)?;

	// tessera.midi
	tessera.set("midi", midi::create(&lua)?)?;

	// tessera.audio
	tessera.set("audio", audio::create(&lua)?)?;

	// tessera.project
	tessera.set("project", project::create(&lua)?)?;

	// tessera.exit()
	tessera.set(
		"exit",
		lua.create_function(|lua, ()| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			state.exit = true;
			Ok(())
		})?,
	)?;

	// tessera.version()
	tessera.set(
		"version",
		lua.create_function(|lua, ()| {
			let version = get_version();
			let v = lua.create_table()?;
			v.set("MAJOR", version.major)?;
			v.set("MINOR", version.minor)?;
			v.set("PATCH", version.patch)?;
			Ok(v)
		})?,
	)?;

	// tessera.is_release()
	tessera.set(
		"is_release",
		lua.create_function(|_, ()| {
			#[cfg(debug_assertions)]
			return Ok(false);

			#[cfg(not(debug_assertions))]
			return Ok(true);
		})?,
	)?;

	// tessera.get_time()
	tessera.set(
		"get_time",
		lua.create_function(|lua, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			let time = (std::time::Instant::now() - state.start_time).as_secs_f64();
			Ok(time)
		})?,
	)?;

	// tessera.sleep(time)
	tessera.set(
		"sleep",
		lua.create_function(|_, time: f64| {
			std::thread::sleep(std::time::Duration::from_secs_f64(time));
			Ok(())
		})?,
	)?;

	tessera.set(
		"dialog_poll",
		lua.create_function(|lua: &Lua, ()| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			if let Some(rx) = &state.dialog_rx {
				match rx.try_recv() {
					Ok(f) => {
						state.dialog_rx = None;
						Ok(Some(f))
					},
					_ => Ok(None),
				}
			} else {
				Ok(None)
			}
		})?,
	)?;

	tessera.set(
		"dialog_save",
		lua.create_function(|lua: &Lua, name: String| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			if state.dialog_rx.is_some() {
				log_warn!("Dialog already open!");
				return Ok(false);
			}

			let (tx, rx) = mpsc::channel();
			state.dialog_rx = Some(rx);

			std::thread::spawn(move || {
				let file = rfd::FileDialog::new()
					.add_filter("save", &["sav"])
					.set_file_name(name)
					.set_directory(std::path::absolute("./out").unwrap())
					.save_file();

				tx.send(file).unwrap();
			});

			Ok(true)
		})?,
	)?;

	tessera.set(
		"dialog_open",
		lua.create_function(|lua: &Lua, ()| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			if state.dialog_rx.is_some() {
				log_warn!("Dialog already open!");
				return Ok(false);
			}

			let (tx, rx) = mpsc::channel();
			state.dialog_rx = Some(rx);

			std::thread::spawn(move || {
				let file = rfd::FileDialog::new()
					.add_filter("save", &["sav"])
					.set_directory(std::path::absolute("./out").unwrap())
					.pick_file();

				tx.send(file).unwrap();
			});

			Ok(true)
		})?,
	)?;

	lua.globals().set("tessera", tessera)?;

	Ok(lua)
}

pub struct Hooks {
	pub load: LuaFunction,
	pub update: LuaFunction,
	pub draw: LuaFunction,
	pub draw_error: LuaFunction,
	pub keypressed: LuaFunction,
	pub keyreleased: LuaFunction,
	pub mousepressed: LuaFunction,
	pub mousereleased: LuaFunction,
	pub mousemoved: LuaFunction,
	pub wheelmoved: LuaFunction,
	pub resize: LuaFunction,
	pub quit: LuaFunction,
}

impl Hooks {
	pub fn new(lua: &Lua) -> LuaResult<Self> {
		let tessera: LuaTable = lua.globals().get("tessera").unwrap();
		let load: LuaFunction = tessera.get("load").unwrap();
		let update: LuaFunction = tessera.get("update").unwrap();
		let draw: LuaFunction = tessera.get("draw").unwrap();
		let draw_error: LuaFunction = tessera.get("draw_error").unwrap();
		let keypressed: LuaFunction = tessera.get("keypressed").unwrap();
		let keyreleased: LuaFunction = tessera.get("keyreleased").unwrap();
		let mousepressed: LuaFunction = tessera.get("mousepressed").unwrap();
		let mousereleased: LuaFunction = tessera.get("mousereleased").unwrap();
		let mousemoved: LuaFunction = tessera.get("mousemoved").unwrap();
		let wheelmoved: LuaFunction = tessera.get("wheelmoved").unwrap();
		let resize: LuaFunction = tessera.get("resize").unwrap();
		let quit: LuaFunction = tessera.get("quit").unwrap();

		Ok(Self {
			load,
			update,
			draw,
			draw_error,
			keypressed,
			keyreleased,
			mousepressed,
			mousereleased,
			mousemoved,
			wheelmoved,
			resize,
			quit,
		})
	}
}
