use crate::embed::Asset;
use crate::log_info;
use crate::opengl::Renderer;
use cosmic_text::fontdb;
use cosmic_text::{
	Align, Attrs, Buffer, CacheKey, Fallback, Family, FontSystem, Metrics, Shaping, SubpixelBin,
	SwashCache, Wrap,
};
use femtovg::imgref::{Img, ImgRef};
use femtovg::rgb::RGBA8;
use femtovg::{
	Atlas, Canvas, Color, DrawCommand, GlyphDrawCommands, ImageFlags, ImageId, ImageSource, Paint,
	Quad,
};
use std::collections::HashMap;
use swash::scale::image::Content;
use unicode_script::Script;

const TEXTURE_SIZE: usize = 512;

pub struct FontTexture {
	atlas: Atlas,
	image_id: ImageId,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GlyphStyle {
	Normal,
	Blur,
}

#[derive(Copy, Clone, Debug)]
pub struct RenderedGlyph {
	texture_index: usize,
	width: u32,
	height: u32,
	offset_x: i32,
	offset_y: i32,
	atlas_x: u32,
	atlas_y: u32,
}

pub struct RenderCache {
	swash_cache: SwashCache,
	rendered_glyphs: HashMap<(CacheKey, GlyphStyle), Option<RenderedGlyph>>,
	glyph_textures: Vec<FontTexture>,
}

impl RenderCache {
	pub(crate) fn new() -> Self {
		Self {
			swash_cache: SwashCache::new(),
			rendered_glyphs: HashMap::default(),
			glyph_textures: Vec::default(),
		}
	}

	pub(crate) fn fill_to_cmds(
		&mut self,
		system: &mut FontSystem,
		canvas: &mut Canvas<Renderer>,
		buffer: &Buffer,
		position: (f32, f32),
		style: GlyphStyle,
	) -> GlyphDrawCommands {
		let mut cmd_map = HashMap::new();

		for run in buffer.layout_runs() {
			for glyph in run.glyphs {
				let physical_glyph = glyph.physical((0.0, 0.0), 1.0);

				let mut cache_key = physical_glyph.cache_key;

				let position_x = position.0 + cache_key.x_bin.as_float();
				let position_y = position.1 + cache_key.y_bin.as_float();

				let (position_x, subpixel_x) = SubpixelBin::new(position_x);

				// Hack: Swash uses Y-up so we need to flip the subpixel bin.
				let (position_y, subpixel_y) = SubpixelBin::new(-position_y);
				let position_y = -position_y;

				cache_key.x_bin = subpixel_x;
				cache_key.y_bin = subpixel_y;

				// perform cache lookup for rendered glyph
				let key = (cache_key, style);
				let Some(rendered) = self.rendered_glyphs.entry(key).or_insert_with(|| {
					// resterize glyph
					let mut rendered = self.swash_cache.get_image_uncached(system, cache_key)?;

					let mut content_x = rendered.placement.left;
					let mut content_y = -rendered.placement.top; // Flip Y
					let mut content_w = rendered.placement.width as usize;
					let mut content_h = rendered.placement.height as usize;

					if content_w == 0 || content_h == 0 {
						return None;
					}

					// Apply blur and change size
					if style == GlyphStyle::Blur {
						if rendered.content == Content::Color {
							return None;
						}
						let pad = 3;
						let sigma = 1.2;
						let new_data = blur(&rendered.data, content_w, content_h, pad, sigma);
						rendered.data = new_data;
						content_x -= pad as i32;
						content_y -= pad as i32;
						content_w += 2 * pad;
						content_h += 2 * pad;
					}

					// Check if there is some space
					let mut found = None;
					for (texture_index, glyph_atlas) in self.glyph_textures.iter_mut().enumerate() {
						if let Some((x, y)) = glyph_atlas.atlas.add_rect(content_w, content_h) {
							found = Some((texture_index, x, y));
							break;
						}
					}
					let (texture_index, atlas_alloc_x, atlas_alloc_y) =
						found.unwrap_or_else(|| {
							// If no atlas could fit the texture, make a new atlas
							// TODO error handling
							let mut atlas = Atlas::new(TEXTURE_SIZE, TEXTURE_SIZE);
							let image_id = canvas
								.create_image(
									Img::new(
										vec![RGBA8::new(0, 0, 0, 0); TEXTURE_SIZE * TEXTURE_SIZE],
										TEXTURE_SIZE,
										TEXTURE_SIZE,
									)
									.as_ref(),
									ImageFlags::NEAREST,
								)
								.unwrap();
							let texture_index = self.glyph_textures.len();
							log_info!("Allocating font atlas {}", texture_index);
							let (x, y) = atlas.add_rect(content_w, content_h).unwrap();
							self.glyph_textures.push(FontTexture { atlas, image_id });
							(texture_index, x, y)
						});

					let mut src_buf = Vec::with_capacity(content_w * content_h);
					match rendered.content {
						Content::Mask => {
							for chunk in rendered.data.chunks_exact(1) {
								// Technically we only need to fill the red channel but this helps debugging the atlas
								src_buf.push(RGBA8::new(chunk[0], chunk[0], chunk[0], 255));
							}
						},
						Content::Color => {
							for chunk in rendered.data.chunks_exact(4) {
								src_buf.push(RGBA8::new(chunk[0], chunk[1], chunk[2], chunk[3]));
							}
						},
						Content::SubpixelMask => unreachable!(),
					}
					canvas
						.update_image::<ImageSource>(
							self.glyph_textures[texture_index].image_id,
							ImgRef::new(&src_buf, content_w, content_h).into(),
							atlas_alloc_x,
							atlas_alloc_y,
						)
						.unwrap();

					Some(RenderedGlyph {
						texture_index,
						width: content_w as u32,
						height: content_h as u32,
						offset_x: content_x,
						offset_y: content_y,
						atlas_x: atlas_alloc_x as u32,
						atlas_y: atlas_alloc_y as u32,
					})
				}) else {
					continue;
				};

				let cmd = cmd_map.entry(rendered.texture_index).or_insert_with(|| DrawCommand {
					image_id: self.glyph_textures[rendered.texture_index].image_id,
					quads: Vec::new(),
				});

				let mut q = Quad::default();
				let it = 1.0 / TEXTURE_SIZE as f32;

				q.x0 = (position_x + physical_glyph.x + rendered.offset_x) as f32;
				q.y0 = (position_y + physical_glyph.y + rendered.offset_y) as f32 + run.line_y;
				q.x1 = q.x0 + rendered.width as f32;
				q.y1 = q.y0 + rendered.height as f32;

				q.s0 = rendered.atlas_x as f32 * it;
				q.t0 = rendered.atlas_y as f32 * it;
				q.s1 = (rendered.atlas_x + rendered.width) as f32 * it;
				q.t1 = (rendered.atlas_y + rendered.height) as f32 * it;

				cmd.quads.push(q);
			}
		}

		GlyphDrawCommands {
			alpha_glyphs: cmd_map.into_values().collect(),
			// We don't care about rendering emoji
			color_glyphs: Vec::new(),
		}
	}
}

struct MyFallback;
impl Fallback for MyFallback {
	fn common_fallback(&self) -> &[&'static str] {
		&["Inter"]
	}

	fn forbidden_fallback(&self) -> &[&'static str] {
		&[]
	}

	fn script_fallback(&self, _script: Script, _locale: &str) -> &[&'static str] {
		&[]
	}
}

#[derive(Debug, Copy, Clone)]
pub enum Font {
	Inter,
	Notes,
}

impl Font {
	fn as_str(&self) -> &'static str {
		match self {
			Font::Inter => "Inter",
			Font::Notes => "Notes",
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub struct Rect(pub f32, pub f32, pub f32, pub f32);

pub struct TextEngine {
	font_system: FontSystem,
	glyph_cache: RenderCache,
	scratch_buffer: Buffer,
}

const SHAPING: Shaping = Shaping::Advanced;

impl TextEngine {
	pub fn new() -> Self {
		// let mut font_system = FontSystem::new();

		let mut db = fontdb::Database::new();

		db.load_font_data(Asset::get("font/inter.ttf").unwrap().data.to_vec());
		db.load_font_data(Asset::get("font/notes.ttf").unwrap().data.to_vec());

		// dbg!(&db.faces().last().unwrap().families);

		let mut font_system =
			FontSystem::new_with_locale_and_db_and_fallback("en-US".into(), db, MyFallback {});

		// let mut font_system = FontSystem::new_with_locale_and_db("en-US".into(), db);

		let mut scratch_buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 20.0));
		scratch_buffer.set_wrap(&mut font_system, Wrap::None);

		Self { font_system, glyph_cache: RenderCache::new(), scratch_buffer }
	}

	pub fn draw_label(
		&mut self,
		canvas: &mut Canvas<Renderer>,
		text: &str,
		rect: Rect,
		align: Option<Align>,
		paint: &Paint,
		font: Font,
		font_size: f32,
	) {
		let Rect(x, y, w, h) = rect;
		let line_height = font_size;

		let metrics = Metrics::new(font_size, line_height);
		self.scratch_buffer.set_metrics(&mut self.font_system, metrics);
		self.scratch_buffer.set_size(&mut self.font_system, Some(w), Some(h));

		let attrs = Attrs::new().family(Family::Name(font.as_str()));

		self.scratch_buffer
			.set_text(&mut self.font_system, text, &attrs, SHAPING, align);

		self.scratch_buffer.shape_until_scroll(&mut self.font_system, false);

		// center within box height
		let y_offset = 0.5 * (h - font_size);

		// Since we don't wrap, there's only a single run.
		let line_w = self.scratch_buffer.layout_runs().next().unwrap().line_w;

		if line_w > w {
			// Measure the width of "..."
			// self.scratch_buffer.set_text(
			// 	&mut self.font_system,
			// 	"...",
			// 	&attrs,
			// 	SHAPING,
			// 	align,
			// );
			// self.scratch_buffer.shape_until_scroll(&mut self.font_system, false);
			// let dots_width = self.scratch_buffer.layout_runs().next().unwrap().line_w;

			const DOTS_WIDTH: f32 = 11.3;

			let w_available = w - DOTS_WIDTH;

			// If "..." doesn't fit, draw nothing
			if w_available <= 0.0 {
				return;
			}

			// self.scratch_buffer.set_text(
			// 	&mut self.font_system,
			// 	text,
			// 	&attrs,
			// 	SHAPING,
			// 	align,
			// );
			// self.scratch_buffer.shape_until_scroll(&mut self.font_system, false);

			let run = self.scratch_buffer.layout_runs().next().unwrap();
			let mut index = 0;
			let mut end_x = 0.;

			// Test if we overflow our box
			for glyph in run.glyphs {
				end_x += glyph.w;
				if end_x > w_available {
					break;
				}
				index = glyph.end;
			}

			// Construct truncated string "Substr..."
			let mut truncated = text[0..index].to_string();
			truncated.push_str("...");

			self.scratch_buffer
				.set_text(&mut self.font_system, &truncated, &attrs, SHAPING, align);
			self.scratch_buffer.shape_until_scroll(&mut self.font_system, false);
		}

		// Draw
		let cmds = self.glyph_cache.fill_to_cmds(
			&mut self.font_system,
			canvas,
			&self.scratch_buffer,
			(x, y + y_offset),
			GlyphStyle::Blur,
		);
		let outline_paint = Paint::color(Color::black());
		canvas.draw_glyph_commands(cmds, &outline_paint);

		let cmds = self.glyph_cache.fill_to_cmds(
			&mut self.font_system,
			canvas,
			&self.scratch_buffer,
			(x, y + y_offset),
			GlyphStyle::Normal,
		);
		canvas.draw_glyph_commands(cmds, paint);
	}

	pub fn draw_text(
		&mut self,
		canvas: &mut Canvas<Renderer>,
		text: &str,
		x: f32,
		y: f32,
		paint: &Paint,
		font: Font,
		font_size: f32,
	) {
		let line_height = font_size;

		let metrics = Metrics::new(font_size, line_height);
		self.scratch_buffer.set_metrics(&mut self.font_system, metrics);

		let attrs = Attrs::new().family(Family::Name(font.as_str()));

		self.scratch_buffer
			.set_text(&mut self.font_system, text, &attrs, SHAPING, None);

		self.scratch_buffer.shape_until_scroll(&mut self.font_system, false);

		// Draw
		let cmds = self.glyph_cache.fill_to_cmds(
			&mut self.font_system,
			canvas,
			&self.scratch_buffer,
			(x, y),
			GlyphStyle::Normal,
		);
		canvas.draw_glyph_commands(cmds, paint);
	}

	pub fn measure_width(&mut self, text: &str, font: Font, font_size: f32) -> f32 {
		let line_height = font_size;

		let metrics = Metrics::new(font_size, line_height);
		self.scratch_buffer.set_metrics(&mut self.font_system, metrics);

		let attrs = Attrs::new().family(Family::Name(font.as_str()));

		self.scratch_buffer
			.set_text(&mut self.font_system, text, &attrs, SHAPING, None);

		self.scratch_buffer.shape_until_scroll(&mut self.font_system, false);

		self.scratch_buffer.layout_runs().next().unwrap().line_w
	}

	pub fn draw_debug_atlas(&self, canvas: &mut Canvas<Renderer>) {
		let mut x = 0.;
		let mut y = 32.;
		for tex in &self.glyph_cache.glyph_textures {
			let image_id = tex.image_id;

			let info = canvas.image_info(image_id).unwrap();
			let w = info.width() as f32;
			let h = info.height() as f32;

			let mut path = femtovg::Path::new();
			path.rect(x, y, w, h);

			let paint = femtovg::Paint::color(Color::black());
			canvas.fill_path(&path, &paint);
			let paint = femtovg::Paint::image(image_id, x, y, w, h, 0.0, 1.0);
			canvas.fill_path(&path, &paint);

			x += w;
			if x > canvas.width() as f32 {
				x = 0.;
				y += h;
			}
		}
	}
}

fn blur(src: &[u8], width: usize, height: usize, size: usize, sigma: f32) -> Vec<u8> {
	let padding = size;
	let new_w = width + padding * 2;
	let new_h = height + padding * 2;
	let mut out = vec![0.0f32; new_w * new_h];

	let mut kernel = Vec::with_capacity(size * size);
	let mut kernel_sum = 0.0;

	// generate gaussian kernel
	for y in -(size as i32)..=(size as i32) {
		for x in -(size as i32)..=(size as i32) {
			let dist_sq = (x * x + y * y) as f32;
			let weight = (-dist_sq / (2.0 * sigma * sigma)).exp();
			kernel.push(weight);
			kernel_sum += weight;
		}
	}

	for y in 0..new_h {
		for x in 0..new_w {
			let src_x = (x as i32) - padding as i32;
			let src_y = (y as i32) - padding as i32;

			let mut acc: f32 = 0.0;

			let mut k_idx = 0;

			for ky in -(size as i32)..=(size as i32) {
				for kx in -(size as i32)..=(size as i32) {
					let sx = src_x + kx;
					let sy = src_y + ky;

					let weight = kernel[k_idx];
					k_idx += 1;

					if sx >= 0 && sx < width as i32 && sy >= 0 && sy < height as i32 {
						let val = src[(sy as usize * width) + sx as usize];
						acc += f32::from(val) * weight;
					}
				}
			}

			out[y * new_w + x] = acc / kernel_sum;
		}
	}

	// darken and convert to u8
	out.iter_mut().map(|x| (*x * 1.7).clamp(0., 255.) as u8).collect()
}
