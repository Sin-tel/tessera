use crate::app::State;
use mlua::prelude::*;
use winit::dpi::LogicalPosition;
use winit::window::{CursorGrabMode, CursorIcon};

pub struct Mouse;

impl LuaUserData for Mouse {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_function("getPosition", |lua, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			Ok(state.mouse_position)
		});

		// love.mouse.setCursor(cursor)
		// Option<String> handles the case where Lua calls setCursor() to reset default.
		methods.add_function("setCursor", |lua, cursor_name: Option<String>| {
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

			state.window.set_cursor_icon(icon);
			Ok(())
		});

		// love.mouse.setRelativeMode(bool)
		methods.add_function("setRelativeMode", |lua, enable: bool| {
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

		// love.mouse.setVisible(bool)
		methods.add_function("setVisible", |lua, visible: bool| {
			let state = lua.app_data_ref::<State>().unwrap();
			state.window.set_cursor_visible(visible);
			Ok(())
		});

		// love.mouse.setPosition(x, y)
		methods.add_function("setPosition", |lua, (x, y): (f32, f32)| {
			let state = lua.app_data_ref::<State>().unwrap();
			state.window.set_cursor_position(LogicalPosition::new(x, y)).unwrap();
			Ok(())
		});
	}
}
