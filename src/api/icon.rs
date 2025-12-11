use crate::app::State;
use crate::log_error;
use femtovg::Path;
use mlua::prelude::*;
use svgtypes::{PathParser, PathSegment};

include!("icons_res.rs");

#[rustfmt::skip]
const BUILTIN_ICONS: &[(&str, &str)] = &[
	("armed",      ICON_ARMED),
	("visible",    ICON_VISIBLE),
	("invisible",  ICON_INVISIBLE),
	("lock",       ICON_LOCK),
	("unlock",     ICON_UNLOCK),
	("save",       ICON_SAVE),
];

pub fn parse_svg_path(path_str: &str) -> Path {
	let mut path = Path::new();

	// Current pen position
	let mut cx = 0.0;
	let mut cy = 0.0;

	// Start of the current sub-path
	let mut start_x = 0.0;
	let mut start_y = 0.0;

	for segment in PathParser::from(path_str) {
		match segment {
			Ok(PathSegment::MoveTo { abs, x, y }) => {
				cx = if abs { x } else { cx + x };
				cy = if abs { y } else { cy + y };
				start_x = cx;
				start_y = cy;

				path.move_to(cx as f32, cy as f32);
			},
			Ok(PathSegment::LineTo { abs, x, y }) => {
				cx = if abs { x } else { cx + x };
				cy = if abs { y } else { cy + y };

				path.line_to(cx as f32, cy as f32);
			},
			Ok(PathSegment::HorizontalLineTo { abs, x }) => {
				cx = if abs { x } else { cx + x };

				path.line_to(cx as f32, cy as f32);
			},
			Ok(PathSegment::VerticalLineTo { abs, y }) => {
				cy = if abs { y } else { cy + y };

				path.line_to(cx as f32, cy as f32);
			},
			Ok(PathSegment::CurveTo { abs, x1, y1, x2, y2, x, y }) => {
				let cx1 = if abs { x1 } else { cx + x1 };
				let cy1 = if abs { y1 } else { cy + y1 };
				let cx2 = if abs { x2 } else { cx + x2 };
				let cy2 = if abs { y2 } else { cy + y2 };

				cx = if abs { x } else { cx + x };
				cy = if abs { y } else { cy + y };

				path.bezier_to(
					cx1 as f32, cy1 as f32, cx2 as f32, cy2 as f32, cx as f32, cy as f32,
				);
			},
			Ok(PathSegment::Quadratic { abs, x1, y1, x, y }) => {
				let cx1 = if abs { x1 } else { cx + x1 };
				let cy1 = if abs { y1 } else { cy + y1 };

				cx = if abs { x } else { cx + x };
				cy = if abs { y } else { cy + y };

				path.quad_to(cx1 as f32, cy1 as f32, cx as f32, cy as f32);
			},
			Ok(PathSegment::ClosePath { .. }) => {
				cx = start_x;
				cy = start_y;
				path.close();
			},
			Ok(e) => log_error!("Not supported: {e:?}"),
			Err(e) => panic!("{e}"),
		}
	}

	path
}

pub fn load_icons(lua: &Lua) -> LuaResult<()> {
	let state = &mut *lua.app_data_mut::<State>().unwrap();

	let icon = lua.create_table()?;

	let mut paths = Vec::new();

	for (index, (name, path)) in BUILTIN_ICONS.iter().enumerate() {
		icon.set(*name, index)?;
		paths.push(parse_svg_path(path));
	}

	state.paths = paths;

	let tessera: LuaTable = lua.globals().get("tessera")?;
	tessera.set("icon", icon)?;

	Ok(())
}
