mod audio;
pub mod graphics;
pub mod image;
pub mod keycodes;
mod midi;
mod mouse;

use crate::app::State;
use mlua::prelude::*;

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

	lua.globals().set("tessera", tessera)?;

	Ok(lua)
}
