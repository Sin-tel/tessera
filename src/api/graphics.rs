use crate::app::State;
use crate::text::Font;
use crate::text::Rect;
use cosmic_text::Align;
use femtovg::{Color, Paint, Path};
use mlua::Variadic;
use mlua::prelude::*;

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
	let graphics = lua.create_table()?;

	graphics.set(
		"set_font_main",
		lua.create_function(|lua, ()| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			state.font = Font::Inter;
			Ok(())
		})?,
	)?;

	graphics.set(
		"set_font_notes",
		lua.create_function(|lua, ()| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			state.font = Font::Notes;
			Ok(())
		})?,
	)?;

	graphics.set(
		"set_font_size",
		lua.create_function(|lua, font_size: f32| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			state.font_size = font_size;
			Ok(())
		})?,
	)?;

	graphics.set(
		"draw",
		lua.create_function(|lua, (image, x, y): (usize, f32, f32)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			let image_id = state.image_ids[image];

			let info = state.canvas.image_info(image_id).unwrap();
			let w = info.width() as f32;
			let h = info.height() as f32;

			let mut path = Path::new();
			path.rect(x, y, w, h);

			let paint = Paint::image_tint(image_id, x, y, w, h, 0.0, state.current_color);
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
		"set_color_f",
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
		"label",
		lua.create_function(
			|lua, (text, x, y, w, h, align): (String, f32, f32, f32, f32, Option<u32>)| {
				let state = &mut *lua.app_data_mut::<State>().unwrap();

				let paint = Paint::color(state.current_color);

				let align = align.map(|s| match s {
					0 => Align::Left,
					1 => Align::Center,
					2 => Align::Right,
					e => panic!("Invalid align mode {e}"),
				});

				let rect = Rect(x, y, w, h);

				state.text_engine.draw_label(
					&mut state.canvas,
					&text,
					rect,
					align,
					&paint,
					state.font,
					state.font_size,
				);

				Ok(())
			},
		)?,
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

	graphics.set("ALIGN_LEFT", 0)?;
	graphics.set("ALIGN_CENTER", 1)?;
	graphics.set("ALIGN_RIGHT", 2)?;

	Ok(graphics)
}
