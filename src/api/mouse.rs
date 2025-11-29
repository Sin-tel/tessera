use crate::app::State;
use mlua::prelude::*;
use winit::dpi::LogicalPosition;
use winit::window::{CursorGrabMode, CursorIcon};

pub struct Mouse;

impl LuaUserData for Mouse {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_function("get_position", |lua, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			Ok(state.mouse_position)
		});

		// tessera.mouse.set_cursor(cursor)
		// Option<String> handles the case where Lua calls set_cursor() to reset default.
		methods.add_function("set_cursor", |lua, cursor_name: Option<String>| {
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
		});

		// tessera.mouse.set_relative_mode(bool)
		methods.add_function("set_relative_mode", |lua, enable: bool| {
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
		});

		// tessera.mouse.setVisible(bool)
		methods.add_function("set_visible", |lua, visible: bool| {
			let state = lua.app_data_ref::<State>().unwrap();
			state.window.set_cursor_visible(visible);
			Ok(())
		});

		// tessera.mouse.setPosition(x, y)
		methods.add_function("set_position", |lua, (x, y): (f32, f32)| {
			let state = lua.app_data_ref::<State>().unwrap();
			state.window.set_cursor_position(LogicalPosition::new(x, y)).unwrap();
			Ok(())
		});
	}
}
