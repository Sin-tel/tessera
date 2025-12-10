use crate::app::State;
use crate::app::{DEFAULT_FONT_SIZE, DEFAULT_LINE_WIDTH};
use crate::text::Font;
use crate::text::Rect;
use cosmic_text::Align;
use femtovg::{Color, LineCap, LineJoin, Paint, Path};
use mlua::Variadic;
use mlua::prelude::*;

pub fn create(lua: &Lua, scale_factor: f64) -> LuaResult<LuaTable> {
	let graphics = lua.create_table()?;

	graphics.set("scale_factor", scale_factor)?;

	// Draw functions

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
		lua.create_function(|lua, font_size: Option<f32>| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			state.font_size = font_size.unwrap_or(DEFAULT_FONT_SIZE * state.scale_factor);
			// round to quarter increments to avoid spamming the cache
			state.font_size = (state.font_size * 4.0).round() / 4.0;
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

	graphics.set(
		"draw_path",
		lua.create_function(|lua, (path_id, x, y): (usize, f32, f32)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			let transform = state.canvas.transform();
			state.canvas.translate(x, y);
			state.canvas.scale(state.scale_factor, state.scale_factor);

			let path = &state.paths[path_id];

			let paint = Paint::color(state.current_color);
			state.canvas.fill_path(&path, &paint);

			state.canvas.reset_transform();
			state.canvas.set_transform(&transform);

			Ok(())
		})?,
	)?;

	//

	graphics.set(
		"get_dimensions",
		lua.create_function(|lua, ()| {
			// This function runs on load so it needs to work without any state attached
			if let Some(state) = lua.app_data_ref::<State>() {
				Ok(state.window_size)
			} else {
				Ok((400, 400))
			}
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
		"get_color_hsv",
		lua.create_function(|lua, (h, s, v): (f64, f32, f32)| {
			use okhsl::*;
			let Rgb { r, g, b } = oklab_to_srgb_f32(okhsv_to_oklab(Okhsv { h, s, v }));
			let t = lua.create_table()?;
			t.set(1, r)?;
			t.set(2, g)?;
			t.set(3, b)?;

			// Ok((r, g, b))
			Ok(t)
		})?,
	)?;

	graphics.set(
		"set_line_width",
		lua.create_function(|lua, line_width: Option<f32>| {
			let mut state = lua.app_data_mut::<State>().unwrap();

			state.line_width = line_width.unwrap_or(DEFAULT_LINE_WIDTH * state.scale_factor);
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
		"text",
		lua.create_function(|lua, (text, x, y): (String, f32, f32)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			let paint = Paint::color(state.current_color);

			state.text_engine.draw_text(
				&mut state.canvas,
				&text,
				x,
				y,
				&paint,
				state.font,
				state.font_size,
			);

			Ok(())
		})?,
	)?;

	graphics.set(
		"measure_width",
		lua.create_function(|lua, text: String| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			Ok(state.text_engine.measure_width(&text, state.font, state.font_size))
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

	graphics.set(
		"polyline",
		lua.create_function(|lua, (x, y): (Vec<f32>, Vec<f32>)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			if x.len() < 2 {
				return Ok(());
			}
			assert!(x.len() == y.len());

			let mut path = Path::new();
			path.move_to(x[0], y[0]);

			for (&x, &y) in x.iter().zip(y.iter()).skip(1) {
				path.line_to(x, y);
			}

			let mut paint = Paint::color(state.current_color);
			paint.set_line_width(state.line_width);
			paint.set_line_cap(LineCap::Round);
			paint.set_line_join(LineJoin::Round);

			state.canvas.stroke_path(&path, &paint);
			Ok(())
		})?,
	)?;

	graphics.set(
		"polyline_w",
		lua.create_function(|lua, (lx, ly, lw): (Vec<f32>, Vec<f32>, Vec<f32>)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			if lx.len() < 2 {
				return Ok(());
			}
			assert!(lx.len() == ly.len());
			assert!(lx.len() == lw.len());

			let n = lx.len();
			let mut path = Path::new();

			let mut bottom_verts: Vec<(f32, f32)> = Vec::with_capacity(n);

			// first point
			let (nx, ny) = get_normal(lx[0], ly[0], lx[1], ly[1]);

			let (x, y, w) = (lx[0], ly[0], lw[0]);
			path.move_to(x + nx * w, y + ny * w);
			bottom_verts.push((x - nx * w, y - ny * w));

			// middle points
			for i in 1..n - 1 {
				let (nx, ny) = get_normal(lx[i - 1], ly[i - 1], lx[i + 1], ly[i + 1]);
				let (x, y, w) = (lx[i], ly[i], lw[i]);

				path.line_to(x + nx * w, y + ny * w);
				bottom_verts.push((x - nx * w, y - ny * w));
			}

			// last point
			let (nx, ny) = get_normal(lx[n - 2], ly[n - 2], lx[n - 1], ly[n - 1]);
			let (x, y, w) = (lx[n - 1], ly[n - 1], lw[n - 1]);

			path.line_to(x + nx * w, y + ny * w);
			bottom_verts.push((x - nx * w, y - ny * w));

			// draw bottom verts in reverse
			for (bx, by) in bottom_verts.iter().rev() {
				path.line_to(*bx, *by);
			}
			path.close();

			// draw
			let paint = Paint::color(state.current_color);
			state.canvas.fill_path(&path, &paint);
			Ok(())
		})?,
	)?;

	graphics.set(
		"verts",
		lua.create_function(|lua, (lx, ly, w): (Vec<f32>, Vec<f32>, f32)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			if lx.len() < 2 {
				return Ok(());
			}
			assert!(lx.len() == ly.len());

			let mut path = Path::new();

			for (&x, &y) in lx.iter().zip(ly.iter()) {
				path.circle(x, y, w);
			}

			// draw
			let paint = Paint::color(state.current_color);
			state.canvas.fill_path(&path, &paint);
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

	graphics.set(
		"draw_spectrum",
		lua.create_function(|lua, (w, h): (f32, f32)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			let Some(audio_ctx) = &mut state.audio else { return Ok(()) };

			let color = state.current_color;
			let sample_rate = audio_ctx.sample_rate as f32;
			audio_ctx
				.scope
				.draw_spectrum(w, h, sample_rate, color, &mut state.canvas);
			Ok(())
		})?,
	)?;

	graphics.set(
		"draw_scope",
		lua.create_function(|lua, (w, h): (f32, f32)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			let Some(audio_ctx) = &mut state.audio else { return Ok(()) };

			let color = state.current_color;
			audio_ctx.scope.draw_scope(w, h, color, &mut state.canvas);

			Ok(())
		})?,
	)?;

	graphics.set(
		"draw_debug_atlas",
		lua.create_function(|lua, ()| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			state.text_engine.draw_debug_atlas(&mut state.canvas);

			Ok(())
		})?,
	)?;

	graphics.set("ALIGN_LEFT", 0)?;
	graphics.set("ALIGN_CENTER", 1)?;
	graphics.set("ALIGN_RIGHT", 2)?;

	Ok(graphics)
}

fn get_normal(x1: f32, y1: f32, x2: f32, y2: f32) -> (f32, f32) {
	let dx = x2 - x1;
	let dy = y2 - y1;
	let len = (dx * dx + dy * dy).sqrt();
	if len > 0.0001 { (-dy / len, dx / len) } else { (0.0, 1.0) }
}
