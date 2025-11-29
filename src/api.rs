pub mod backend;
pub mod graphics;
pub mod keycodes;
mod mouse;

use crate::State;
use crate::api::graphics::Graphics;
use crate::api::mouse::Mouse;
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

	// love.graphics
	let love = lua.create_table()?;
	let graphics = Graphics {};
	love.set("graphics", graphics)?;

	// love.mouse
	let mouse = Mouse {};
	love.set("mouse", mouse)?;

	// love.event
	let event = lua.create_table()?;
	let quit = lua.create_function(|lua, ()| {
		let mut state = lua.app_data_mut::<State>().unwrap();
		state.exit = true;
		Ok(())
	})?;
	event.set("quit", quit)?;
	love.set("event", event)?;

	// love.timer
	let timer = lua.create_table()?;
	let get_time = lua.create_function(|lua, ()| {
		let state = lua.app_data_ref::<State>().unwrap();
		let time = (std::time::Instant::now() - state.start_time).as_secs_f64();
		Ok(time)
	})?;
	timer.set("getTime", get_time)?;

	let sleep = lua.create_function(|_, time: f64| {
		std::thread::sleep(std::time::Duration::from_secs_f64(time));
		Ok(())
	})?;
	timer.set("sleep", sleep)?;

	love.set("timer", timer)?;

	lua.globals().set("love", love)?;

	Ok(lua)
}
