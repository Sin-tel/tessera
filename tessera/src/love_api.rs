use crate::State;
use crate::keycodes::love2d_key_to_keycode;
use femtovg::ImageFlags;
use femtovg::ImageId;
use femtovg::{Color, Paint, Path};
use mlua::Variadic;
use mlua::prelude::*;
use winit::dpi::LogicalPosition;
use winit::event::MouseButton;
use winit::window::{CursorGrabMode, CursorIcon};

fn to_f32(value: &LuaValue) -> LuaResult<f32> {
	match value {
		LuaValue::Number(n) => Ok(*n as f32),
		LuaValue::Integer(i) => Ok(*i as f32),
		LuaValue::Nil => Err(LuaError::RuntimeError("missing value".into())),
		_ => Err(LuaError::RuntimeError("expected number".into())),
	}
}

fn opt_f32(value: &LuaValue) -> Option<f32> {
	match value {
		LuaValue::Number(n) => Some(*n as f32),
		LuaValue::Integer(i) => Some(*i as f32),
		_ => None,
	}
}

fn parse_color(args: LuaMultiValue) -> LuaResult<Color> {
	if let Some(LuaValue::Table(table)) = args.get(0) {
		let r = to_f32(&table.get(1)?)?;
		let g = to_f32(&table.get(2)?)?;
		let b = to_f32(&table.get(3)?)?;
		let a = opt_f32(&table.get(4)?).unwrap_or(1.0);
		Ok(Color::rgbaf(r, g, b, a))
	} else {
		let r = to_f32(
			args.get(0)
				.ok_or_else(|| LuaError::RuntimeError("missing r".into()))?,
		)?;
		let g = to_f32(
			args.get(1)
				.ok_or_else(|| LuaError::RuntimeError("missing g".into()))?,
		)?;
		let b = to_f32(
			args.get(2)
				.ok_or_else(|| LuaError::RuntimeError("missing b".into()))?,
		)?;
		let a = args.get(3).and_then(opt_f32).unwrap_or(1.0);
		Ok(Color::rgbaf(r, g, b, a))
	}
}

#[derive(Clone, Copy)]
pub struct Image {
	id: ImageId,
	width: usize,
	height: usize,
}

impl LuaUserData for Image {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("getHeight", |_lua, this, ()| Ok(this.height));
		methods.add_method("getWidth", |_lua, this, ()| Ok(this.width));
	}
}

impl FromLua for Image {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		if let LuaValue::UserData(ref ud) = value
			&& let Ok(img_ref) = ud.borrow::<Image>()
		{
			return Ok(*img_ref);
		}

		Err(LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "Image".to_string(),
			message: Some("Expected an Image object".to_string()),
		})
	}
}

// Font shim
#[derive(Clone)]
pub struct Font {
	pub name: String,
	pub size: f32,
}

impl LuaUserData for Font {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("getHeight", |lua, _this, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			Ok(state.font.size * 1.2)
		});
		methods.add_method("getWidth", |lua, _this, text: String| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			let width = state.text_engine.measure_width(&text, state.font.size);
			Ok(width)
		});
	}
}

impl FromLua for Font {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		if let LuaValue::UserData(ref ud) = value
			&& let Ok(font_ref) = ud.borrow::<Font>()
		{
			return Ok(font_ref.clone());
		}

		Err(LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "Font".to_string(),
			message: Some("Expected an Font object".to_string()),
		})
	}
}

pub struct Graphics;

impl LuaUserData for Graphics {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		// Resources
		methods.add_function("newFont", |_, (name, size): (String, f32)| Ok(Font { name, size }));

		methods.add_function("getFont", |lua, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			Ok(state.font.clone())
		});

		methods.add_function("setFont", |lua, font: Font| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			state.font = font;
			Ok(())
		});

		// Image

		// love.graphics.newImage(filename)
		methods.add_function("newImage", |lua, filename: String| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			let image_id = state.canvas.load_image_file(&filename, ImageFlags::empty()).unwrap();

			let info = state.canvas.image_info(image_id).unwrap();
			Ok(Image { id: image_id, width: info.width(), height: info.height() })
		});

		methods.add_function("draw", |lua, (image, x, y): (Image, f32, f32)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			let w = image.width as f32;
			let h = image.height as f32;

			let mut path = Path::new();
			path.rect(x, y, w, h);

			let paint = Paint::image_tint(image.id, x, y, w, h, 0.0, state.current_color);
			state.canvas.fill_path(&path, &paint);

			Ok(())
		});

		//

		methods.add_function("getDimensions", |lua, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			Ok(state.window_size)
		});

		methods.add_function("setBackgroundColor", |lua, (r, g, b): (f32, f32, f32)| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			state.background_color = Color::rgbf(r, g, b);
			Ok(())
		});

		methods.add_function("setColor", |lua, args: LuaMultiValue| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			state.current_color = parse_color(args)?;
			Ok(())
		});

		methods.add_function("setLineWidth", |lua, w: f32| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			state.line_width = w + 0.5;
			Ok(())
		});

		// Scissor
		methods.add_function("getScissor", |lua, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			// Return current scissor rect, or nil if none
			if let Some((x, y, w, h)) = state.current_scissor {
				lua.pack_multi((x, y, w, h))
			} else {
				lua.pack_multi(())
			}
		});

		methods.add_function("setScissor", |lua, args: LuaMultiValue| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			if args.is_empty() {
				state.canvas.reset_scissor();
				state.current_scissor = None;
			} else {
				let x = to_f32(args.get(0).unwrap())?;
				let y = to_f32(args.get(1).unwrap())?;
				let w = to_f32(args.get(2).unwrap())?;
				let h = to_f32(args.get(3).unwrap())?;

				let (sx, sy) = state.canvas.transform().inverse().transform_point(x, y);

				state.canvas.scissor(sx, sy, w, h);
				state.current_scissor = Some((x, y, w, h));
			}
			Ok(())
		});

		methods.add_function("intersectScissor", |lua, (x, y, w, h): (f32, f32, f32, f32)| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			let (new_x, new_y, new_w, new_h) = if let Some((sx, sy, sw, sh)) = state.current_scissor
			{
				// Intersect with existing scissor
				let x1 = x.max(sx);
				let y1 = y.max(sy);
				let x2 = (x + w).min(sx + sw);
				let y2 = (y + h).min(sy + sh);
				(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
			} else {
				(x, y, w, h)
			};

			let (sx, sy) = state.canvas.transform().inverse().transform_point(new_x, new_y);

			state.canvas.scissor(sx, sy, new_w, new_h);
			state.current_scissor = Some((new_x, new_y, new_w, new_h));
			Ok(())
		});

		// Draw functions
		methods.add_function(
			"rectangle",
			|lua, (mode, x, y, w, h, r): (String, f32, f32, f32, f32, Option<f32>)| {
				let mut state = lua.app_data_mut::<State>().unwrap();

				let mut path = Path::new();
				match r {
					Some(r) => path.rounded_rect(x, y, w, h, r),
					None => path.rect(x, y, w, h),
				}

				let mut paint = Paint::color(state.current_color);
				paint.set_line_width(state.line_width);

				match mode.as_str() {
					"fill" => {
						state.canvas.fill_path(&path, &paint);
					},
					"line" => {
						state.canvas.stroke_path(&path, &paint);
					},
					m => panic!("Invalid draw mode {m}, expected one of: 'line', 'fill'"),
				}

				Ok(())
			},
		);

		methods.add_function("line", |lua, (x1, y1, x2, y2): (f32, f32, f32, f32)| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			let mut path = Path::new();
			path.move_to(x1, y1);
			path.line_to(x2, y2);

			let mut paint = Paint::color(state.current_color);
			paint.set_line_width(state.line_width);

			state.canvas.stroke_path(&path, &paint);
			Ok(())
		});

		methods.add_function("print", |lua, (text, x, y): (String, f32, f32)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			let paint = Paint::color(state.current_color).with_font_size(state.font.size);

			state
				.text_engine
				.draw_text(&mut state.canvas, &text, x, y, &paint, &state.font.name);

			Ok(())
		});

		methods.add_function("circle", |lua, (mode, x, y, r): (String, f32, f32, f32)| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			let mut path = Path::new();
			path.circle(x, y, r);

			let mut paint = Paint::color(state.current_color);

			match mode.as_str() {
				"fill" => {
					state.canvas.fill_path(&path, &paint);
				},
				"line" => {
					paint.set_line_width(1.0);
					state.canvas.stroke_path(&path, &paint);
				},
				m => panic!("Invalid draw mode {m}, expected one of: 'line', 'fill'"),
			}

			Ok(())
		});

		methods.add_function("ellipse", |lua, (mode, x, y, w, h): (String, f32, f32, f32, f32)| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			let mut path = Path::new();
			path.ellipse(x, y, w, h);

			let mut paint = Paint::color(state.current_color);

			match mode.as_str() {
				"fill" => {
					state.canvas.fill_path(&path, &paint);
				},
				"line" => {
					paint.set_line_width(1.0);
					state.canvas.stroke_path(&path, &paint);
				},
				m => panic!("Invalid draw mode {m}, expected one of: 'line', 'fill'"),
			}

			Ok(())
		});

		methods.add_function("polygon", |lua, (mode, points): (String, Variadic<f32>)| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			if points.len() < 4 || points.len() % 2 != 0 {
				return Err(LuaError::RuntimeError(
					"Polygon requires at least 2 points (x, y) pairs.".into(),
				));
			}

			let mut path = Path::new();

			path.move_to(points[0], points[1]);
			for chunk in points[2..].chunks_exact(2) {
				path.line_to(chunk[0], chunk[1]);
			}
			path.close();

			let mut paint = Paint::color(state.current_color);
			match mode.as_str() {
				"fill" => {
					state.canvas.fill_path(&path, &paint);
				},
				"line" => {
					paint.set_line_width(state.line_width);
					state.canvas.stroke_path(&path, &paint);
				},
				m => panic!("Invalid draw mode {m}, expected one of: 'line', 'fill'"),
			}

			Ok(())
		});

		methods.add_function("clear", |lua, ()| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			let size = state.window.inner_size();

			let bg_color = state.background_color;
			state.canvas.clear_rect(0, 0, size.width, size.height, bg_color);
			Ok(())
		});

		// Transform
		methods.add_function("origin", |lua, ()| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			state.canvas.reset_transform();
			Ok(())
		});

		methods.add_function("push", |lua, ()| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			let current = state.canvas.transform();
			state.transform_stack.push(current);
			Ok(())
		});

		methods.add_function("pop", |lua, ()| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			let transform = state
				.transform_stack
				.pop()
				.expect("Transform stack should never be empty");
			state.canvas.reset_transform();
			state.canvas.set_transform(&transform);
			Ok(())
		});

		methods.add_function("translate", |lua, (x, y): (f32, f32)| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			// This should compose with current transform
			state.canvas.translate(x, y);
			Ok(())
		});

		methods.add_function("rotate", |lua, angle: f32| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			// This should also compose with current transform
			state.canvas.rotate(angle);
			Ok(())
		});

		methods.add_function("transformPoint", |lua, (x, y): (f32, f32)| {
			let state = lua.app_data_ref::<State>().unwrap();
			let (sx, sy) = state.canvas.transform().transform_point(x, y);

			Ok((sx, sy))
		});
	}
}

pub struct Mouse;

impl LuaUserData for Mouse {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_function("isDown", |lua, button_number: u16| {
			let button = match button_number {
				1 => MouseButton::Left,
				2 => MouseButton::Right,
				3 => MouseButton::Middle,
				4 => MouseButton::Back,
				5 => MouseButton::Forward,
				i => MouseButton::Other(i),
			};
			let state = lua.app_data_ref::<State>().unwrap();
			if state.mouse_down.contains(&button) {
				return Ok(true);
			}
			Ok(false)
		});

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

pub fn create_love_env() -> LuaResult<Lua> {
	// let lua = unsafe { Lua::unsafe_new() };
	let lua = unsafe {
		Lua::unsafe_new_with(
			// mlua::StdLib::FFI | mlua::StdLib::DEBUG | mlua::StdLib::ALL_SAFE,
			mlua::StdLib::ALL,
			mlua::LuaOptions::default(),
		)
	};

	// love.graphics
	let love = lua.create_table()?;
	let graphics = Graphics {};
	love.set("graphics", graphics)?;

	// love.mouse
	let mouse = Mouse {};
	love.set("mouse", mouse)?;

	// love.keyboard
	let keyboard = lua.create_table()?;
	let is_down = lua.create_function(|lua: &Lua, key: String| {
		if let Some(code) = love2d_key_to_keycode(&key) {
			let state = lua.app_data_ref::<State>().unwrap();
			if state.keys_down.contains(&code) {
				return Ok(true);
			}
		}
		Ok(false)
	})?;
	keyboard.set("isDown", is_down)?;
	love.set("keyboard", keyboard)?;

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
	let get_fps = lua.create_function(|lua, ()| {
		let state = lua.app_data_ref::<State>().unwrap();
		Ok(state.timer.fps)
	})?;
	timer.set("getFPS", get_fps)?;
	let get_time = lua.create_function(|lua, ()| {
		let state = lua.app_data_ref::<State>().unwrap();
		Ok(state.timer.get_time())
	})?;
	timer.set("getTime", get_time)?;
	love.set("timer", timer)?;

	// love.filesystem
	let filesystem = lua.create_table()?;
	let get_source = lua.create_function(|lua, ()| {
		let state = lua.app_data_mut::<State>().unwrap();
		Ok(state.lua_dir.clone())
	})?;
	filesystem.set("getSource", get_source)?;
	love.set("filesystem", filesystem)?;

	lua.globals().set("love", love)?;

	Ok(lua)
}
