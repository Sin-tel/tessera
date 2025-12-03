mod audio;
pub mod graphics;
pub mod image;
pub mod keycodes;
mod midi;
mod mouse;
pub mod project;

use crate::app::State;
use crate::log::log_warn;
use mlua::prelude::*;
use std::sync::mpsc;

pub fn create_lua() -> LuaResult<Lua> {
	// #[cfg(debug_assertions)]
	let lua = unsafe {
		Lua::unsafe_new_with(
			mlua::StdLib::DEBUG | mlua::StdLib::ALL_SAFE,
			mlua::LuaOptions::default(),
		)
	};

	// #[cfg(not(debug_assertions))]
	// let lua = Lua::new();

	// main tessera table
	let tessera = lua.create_table()?;

	// tessera.graphics
	tessera.set("graphics", graphics::create(&lua)?)?;

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
