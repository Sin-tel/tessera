use crate::opengl::Renderer;
use crate::text::imgref::Img;
use crate::text::imgref::ImgRef;
use crate::text::rgb::RGBA8;
use cosmic_text::CacheKey;
use cosmic_text::Fallback;
use cosmic_text::Family;
use cosmic_text::SubpixelBin;
use cosmic_text::fontdb;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};
use femtovg::Atlas;
use femtovg::DrawCommand;
use femtovg::GlyphDrawCommands;
use femtovg::ImageFlags;
use femtovg::ImageId;
use femtovg::ImageSource;
use femtovg::Quad;
use femtovg::imgref;
use femtovg::rgb;
use femtovg::{Canvas, Paint};
use std::collections::HashMap;
use swash::scale::image::Content;
use unicode_script::Script;

const TEXTURE_SIZE: usize = 512;

pub struct FontTexture {
	atlas: Atlas,
	image_id: ImageId,
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
	color_glyph: bool,
}

pub struct RenderCache {
	swash_cache: SwashCache,
	rendered_glyphs: HashMap<CacheKey, Option<RenderedGlyph>>,
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
	) -> GlyphDrawCommands {
		let mut alpha_cmd_map = HashMap::new();
		let mut color_cmd_map = HashMap::new();

		//let total_height = buffer.layout_runs().len() as i32 * buffer.metrics().line_height;
		for run in buffer.layout_runs() {
			for glyph in run.glyphs {
				let physical_glyph = glyph.physical((0.0, 0.0), 1.0);

				let mut cache_key = physical_glyph.cache_key;

				let position_x = position.0 + cache_key.x_bin.as_float();
				let position_y = position.1 + cache_key.y_bin.as_float();
				//let position_x = position_x - run.line_w * justify.0;
				//let position_y = position_y - total_height as f32 * justify.1;
				let (position_x, subpixel_x) = SubpixelBin::new(position_x);
				let (position_y, subpixel_y) = SubpixelBin::new(position_y);
				cache_key.x_bin = subpixel_x;
				cache_key.y_bin = subpixel_y;
				// perform cache lookup for rendered glyph
				let Some(rendered) = self.rendered_glyphs.entry(cache_key).or_insert_with(|| {
					// resterize glyph
					let rendered = self.swash_cache.get_image_uncached(system, cache_key)?;

					// upload it to the GPU
					// pick an atlas texture for our glyph
					let content_w = rendered.placement.width as usize;
					let content_h = rendered.placement.height as usize;

					if content_w == 0 && content_h == 0 {
						return None;
					}

					let mut found = None;
					for (texture_index, glyph_atlas) in self.glyph_textures.iter_mut().enumerate() {
						if let Some((x, y)) = glyph_atlas.atlas.add_rect(content_w, content_h) {
							found = Some((texture_index, x, y));
							break;
						}
					}
					let (texture_index, atlas_alloc_x, atlas_alloc_y) =
						found.unwrap_or_else(|| {
							// if no atlas could fit the texture, make a new atlas tyvm
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
							let (x, y) = atlas.add_rect(content_w, content_h).unwrap();
							self.glyph_textures.push(FontTexture { atlas, image_id });
							(texture_index, x, y)
						});

					let mut src_buf = Vec::with_capacity(content_w * content_h);
					match rendered.content {
						Content::Mask => {
							for chunk in rendered.data.chunks_exact(1) {
								src_buf.push(RGBA8::new(chunk[0], 0, 0, 0));
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
						width: rendered.placement.width,
						height: rendered.placement.height,
						offset_x: rendered.placement.left,
						offset_y: rendered.placement.top,
						atlas_x: atlas_alloc_x as u32,
						atlas_y: atlas_alloc_y as u32,
						color_glyph: matches!(rendered.content, Content::Color),
					})
				}) else {
					continue;
				};

				let cmd_map =
					if rendered.color_glyph { &mut color_cmd_map } else { &mut alpha_cmd_map };

				let cmd = cmd_map.entry(rendered.texture_index).or_insert_with(|| DrawCommand {
					image_id: self.glyph_textures[rendered.texture_index].image_id,
					quads: Vec::new(),
				});

				let mut q = Quad::default();
				let it = 1.0 / TEXTURE_SIZE as f32;

				q.x0 = (position_x + physical_glyph.x + rendered.offset_x) as f32;
				q.y0 = (position_y + physical_glyph.y - rendered.offset_y) as f32 + run.line_y;
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
			alpha_glyphs: alpha_cmd_map.into_values().collect(),
			color_glyphs: color_cmd_map.into_values().collect(),
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

pub struct TextEngine {
	font_system: FontSystem,
	glyph_cache: RenderCache,
	scratch_buffer: Buffer,
}

impl TextEngine {
	pub fn new() -> Self {
		// let mut font_system = FontSystem::new();

		let mut db = fontdb::Database::new();
		db.load_font_data(include_bytes!("../assets/font/inter.ttf").to_vec());
		db.load_font_data(include_bytes!("../assets/font/notes.ttf").to_vec());

		let mut font_system =
			FontSystem::new_with_locale_and_db_and_fallback("en-US".into(), db, MyFallback {});

		// let mut font_system = FontSystem::new_with_locale_and_db("en-US".into(), db);

		let mut scratch_buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 20.0));
		scratch_buffer.set_wrap(&mut font_system, cosmic_text::Wrap::None);

		Self { font_system, glyph_cache: RenderCache::new(), scratch_buffer }
	}

	pub fn draw_text(
		&mut self,
		canvas: &mut Canvas<Renderer>,
		text: &str,
		x: f32,
		y: f32,
		paint: &Paint,
		font_name: &str,
	) {
		let font_size = paint.font_size();
		let line_height = font_size * 1.2;

		let metrics = Metrics::new(font_size, line_height);

		self.scratch_buffer.set_metrics(&mut self.font_system, metrics);

		let attrs = Attrs::new().family(Family::Name(font_name));

		self.scratch_buffer.set_text(
			&mut self.font_system,
			text,
			&attrs,
			Shaping::Basic,
			// Some(Align::Left),
			None,
		);

		self.scratch_buffer.shape_until_scroll(&mut self.font_system, false);

		let cmds = self.glyph_cache.fill_to_cmds(
			&mut self.font_system,
			canvas,
			&self.scratch_buffer,
			(x, y),
		);

		canvas.draw_glyph_commands(cmds, paint);
	}

	pub fn measure_width(&mut self, text: &str, font_size: f32) -> f32 {
		let metrics = Metrics::new(font_size, font_size * 1.2);
		self.scratch_buffer.set_metrics(&mut self.font_system, metrics);

		self.scratch_buffer.set_text(
			&mut self.font_system,
			text,
			&Attrs::new(),
			Shaping::Advanced,
			// Some(Align::Left),
			None,
		);
		self.scratch_buffer.shape_until_scroll(&mut self.font_system, false);

		self.scratch_buffer
			.layout_runs()
			.map(|run| run.line_w)
			.fold(0.0, f32::max)
	}
}
