mod audio;
pub mod graphics;
pub mod keycodes;
mod midi;
mod mouse;

use crate::api::audio::Audio;
use crate::api::graphics::Graphics;
use crate::api::midi::Midi;
use crate::api::mouse::Mouse;
use crate::app::State;
use mlua::prelude::*;

pub fn create_lua() -> LuaResult<Lua> {
	#[cfg(debug_assertions)]
	let lua = unsafe {
		Lua::unsafe_new_with(
			mlua::StdLib::DEBUG | mlua::StdLib::ALL_SAFE,
			mlua::LuaOptions::default(),
		)
	};

	#[cfg(not(debug_assertions))]
	let lua = Lua::new();

	// tessera.graphics
	let tessera = lua.create_table()?;
	let graphics = Graphics {};
	tessera.set("graphics", graphics)?;

	// tessera.mouse
	let mouse = Mouse {};
	tessera.set("mouse", mouse)?;

	// tessera.midi
	let midi = Midi {};
	tessera.set("midi", midi)?;

	// tessera.audio
	let audio = Audio {};
	tessera.set("audio", audio)?;

	// tessera.event
	let event = lua.create_table()?;
	let quit = lua.create_function(|lua, ()| {
		let mut state = lua.app_data_mut::<State>().unwrap();
		state.exit = true;
		Ok(())
	})?;
	event.set("quit", quit)?;
	tessera.set("event", event)?;

	// tessera.timer
	let timer = lua.create_table()?;
	let get_time = lua.create_function(|lua, ()| {
		let state = lua.app_data_ref::<State>().unwrap();
		let time = (std::time::Instant::now() - state.start_time).as_secs_f64();
		Ok(time)
	})?;
	timer.set("get_time", get_time)?;

	let sleep = lua.create_function(|_, time: f64| {
		std::thread::sleep(std::time::Duration::from_secs_f64(time));
		Ok(())
	})?;
	timer.set("sleep", sleep)?;

	tessera.set("timer", timer)?;

	lua.globals().set("tessera", tessera)?;

	Ok(lua)
}
