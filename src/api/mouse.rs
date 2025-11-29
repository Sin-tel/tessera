use crate::app::State;
use mlua::prelude::*;
use winit::dpi::LogicalPosition;
use winit::window::{CursorGrabMode, CursorIcon};

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
	let mouse = lua.create_table()?;

	mouse.set(
		"get_position",
		lua.create_function(|lua, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			Ok(state.mouse_position)
		})?,
	)?;

	// tessera.mouse.set_cursor(cursor)
	// Option<String> handles the case where Lua calls set_cursor() to reset default.
	mouse.set(
		"set_cursor",
		lua.create_function(|lua, cursor_name: Option<String>| {
			let state = lua.app_data_ref::<State>().unwrap();

			// If nil/None passed, default to Arrow. Otherwise parse the string.
			let icon_name = cursor_name.as_deref().unwrap_or("default");

			let icon = match icon_name {
				"default" => CursorIcon::Default,
				"v" => CursorIcon::EwResize,
				"h" => CursorIcon::NsResize,
				"wait" => CursorIcon::Wait,
				"text" => CursorIcon::Text,
				"move" => CursorIcon::Move,
				"grab" => CursorIcon::Grab,
				"crosshair" => CursorIcon::Crosshair,
				"progress" => CursorIcon::Progress,
				"help" => CursorIcon::Help,
				"no" => CursorIcon::NotAllowed,
				"hand" => CursorIcon::Pointer,
				"nwse" => CursorIcon::NwseResize,
				"nesw" => CursorIcon::NeswResize,
				e => panic!("Unknown cursor \"{e}\""),
			};

			state.window.set_cursor(icon);
			Ok(())
		})?,
	)?;

	// tessera.mouse.set_relative_mode(bool)
	mouse.set(
		"set_relative_mode",
		lua.create_function(|lua, enable: bool| {
			let state = lua.app_data_ref::<State>().unwrap();

			if enable {
				// If Locked fails, fallback to Confined
				state
					.window
					.set_cursor_grab(CursorGrabMode::Locked)
					.or_else(|_e| state.window.set_cursor_grab(CursorGrabMode::Confined))
					.unwrap();

				// Hide the cursor in relative mode
				state.window.set_cursor_visible(false);

				Ok(())
			} else {
				state.window.set_cursor_grab(CursorGrabMode::None).unwrap();
				state.window.set_cursor_visible(true);
				Ok(())
			}
		})?,
	)?;

	// tessera.mouse.set_visible(bool)
	mouse.set(
		"set_visible",
		lua.create_function(|lua, visible: bool| {
			let state = lua.app_data_ref::<State>().unwrap();
			state.window.set_cursor_visible(visible);
			Ok(())
		})?,
	)?;

	// tessera.mouse.set_position(x, y)
	mouse.set(
		"set_position",
		lua.create_function(|lua, (x, y): (f32, f32)| {
			let state = lua.app_data_ref::<State>().unwrap();
			state.window.set_cursor_position(LogicalPosition::new(x, y)).unwrap();
			Ok(())
		})?,
	)?;

	Ok(mouse)
}
