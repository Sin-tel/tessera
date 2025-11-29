use crate::app::State;
use femtovg::ImageFlags;
use femtovg::ImageId;
use femtovg::{Color, Paint, Path};
use mlua::Variadic;
use mlua::prelude::*;

#[derive(Clone, Copy)]
pub struct Image {
	id: ImageId,
	width: usize,
	height: usize,
}

impl LuaUserData for Image {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("get_height", |_lua, this, ()| Ok(this.height));
		methods.add_method("get_width", |_lua, this, ()| Ok(this.width));
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
		methods.add_method("get_height", |lua, _this, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			Ok(state.font.size * 1.2)
		});
		methods.add_method("get_width", |lua, _this, text: String| {
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

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
	let graphics = lua.create_table()?;

	// Resources
	graphics.set(
		"new_font",
		lua.create_function(|_, (name, size): (String, f32)| Ok(Font { name, size }))?,
	)?;

	graphics.set(
		"get_font",
		lua.create_function(|lua, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			Ok(state.font.clone())
		})?,
	)?;

	graphics.set(
		"set_font",
		lua.create_function(|lua, font: Font| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			state.font = font;
			Ok(())
		})?,
	)?;

	// Image
	graphics.set(
		"new_image",
		lua.create_function(|lua, filename: String| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			let image_id = state.canvas.load_image_file(&filename, ImageFlags::empty()).unwrap();

			let info = state.canvas.image_info(image_id).unwrap();
			Ok(Image { id: image_id, width: info.width(), height: info.height() })
		})?,
	)?;

	graphics.set(
		"draw",
		lua.create_function(|lua, (image, x, y): (Image, f32, f32)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			let w = image.width as f32;
			let h = image.height as f32;

			let mut path = Path::new();
			path.rect(x, y, w, h);

			let paint = Paint::image_tint(image.id, x, y, w, h, 0.0, state.current_color);
			state.canvas.fill_path(&path, &paint);

			Ok(())
		})?,
	)?;

	//

	graphics.set(
		"get_dimensions",
		lua.create_function(|lua, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			Ok(state.window_size)
		})?,
	)?;

	graphics.set(
		"set_color",
		lua.create_function(|lua, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			let a = a.unwrap_or(1.0);
			state.current_color = Color::rgbaf(r, g, b, a);
			Ok(())
		})?,
	)?;

	graphics.set(
		"set_line_width",
		lua.create_function(|lua, w: f32| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			state.line_width = w + 0.5;
			Ok(())
		})?,
	)?;

	// Scissor
	graphics.set(
		"get_scissor",
		lua.create_function(|lua, ()| {
			let state = lua.app_data_ref::<State>().unwrap();
			// Return current scissor rect, or nil if none
			if let Some((x, y, w, h)) = state.current_scissor {
				lua.pack_multi((x, y, w, h))
			} else {
				lua.pack_multi(())
			}
		})?,
	)?;

	graphics.set(
		"reset_scissor",
		lua.create_function(|lua, ()| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			state.canvas.reset_scissor();
			state.current_scissor = None;
			Ok(())
		})?,
	)?;

	graphics.set(
		"set_scissor",
		lua.create_function(|lua, (x, y, w, h): (f32, f32, f32, f32)| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			let (sx, sy) = state.canvas.transform().inverse().transform_point(x, y);

			state.canvas.scissor(sx, sy, w, h);
			state.current_scissor = Some((x, y, w, h));
			Ok(())
		})?,
	)?;

	graphics.set(
		"intersect_scissor",
		lua.create_function(|lua, (x, y, w, h): (f32, f32, f32, f32)| {
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
		})?,
	)?;

	// Draw functions
	graphics.set(
		"rectangle",
		lua.create_function(
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
		)?,
	)?;

	graphics.set(
		"line",
		lua.create_function(|lua, (x1, y1, x2, y2): (f32, f32, f32, f32)| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			let mut path = Path::new();
			path.move_to(x1, y1);
			path.line_to(x2, y2);

			let mut paint = Paint::color(state.current_color);
			paint.set_line_width(state.line_width);

			state.canvas.stroke_path(&path, &paint);
			Ok(())
		})?,
	)?;

	graphics.set(
		"print",
		lua.create_function(|lua, (text, x, y): (String, f32, f32)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			let paint = Paint::color(state.current_color).with_font_size(state.font.size);

			state
				.text_engine
				.draw_text(&mut state.canvas, &text, x, y, &paint, &state.font.name);

			Ok(())
		})?,
	)?;

	graphics.set(
		"circle",
		lua.create_function(|lua, (mode, x, y, r): (String, f32, f32, f32)| {
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
		})?,
	)?;

	graphics.set(
		"ellipse",
		lua.create_function(|lua, (mode, x, y, w, h): (String, f32, f32, f32, f32)| {
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
		})?,
	)?;

	graphics.set(
		"polygon",
		lua.create_function(|lua, (mode, points): (String, Variadic<f32>)| {
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
		})?,
	)?;

	// Transform
	graphics.set(
		"push",
		lua.create_function(|lua, ()| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			let current = state.canvas.transform();
			state.transform_stack.push(current);
			Ok(())
		})?,
	)?;

	graphics.set(
		"pop",
		lua.create_function(|lua, ()| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			let transform = state
				.transform_stack
				.pop()
				.expect("Transform stack should never be empty");
			state.canvas.reset_transform();
			state.canvas.set_transform(&transform);
			Ok(())
		})?,
	)?;

	graphics.set(
		"translate",
		lua.create_function(|lua, (x, y): (f32, f32)| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			// This should compose with current transform
			state.canvas.translate(x, y);
			Ok(())
		})?,
	)?;

	graphics.set(
		"rotate",
		lua.create_function(|lua, angle: f32| {
			let mut state = lua.app_data_mut::<State>().unwrap();
			// This should also compose with current transform
			state.canvas.rotate(angle);
			Ok(())
		})?,
	)?;

	graphics.set(
		"transform_point",
		lua.create_function(|lua, (x, y): (f32, f32)| {
			let state = lua.app_data_ref::<State>().unwrap();
			let (sx, sy) = state.canvas.transform().transform_point(x, y);

			Ok((sx, sy))
		})?,
	)?;

	Ok(graphics)
}
