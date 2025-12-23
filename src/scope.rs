use crate::dsp::C5_HZ;
use crate::opengl::Renderer;
use bit_mask_ring_buf::BitMaskRB;
use femtovg::{Canvas, Color, Paint, Path};
use realfft::{RealFftPlanner, RealToComplex};
use ringbuf::HeapCons;
use ringbuf::traits::*;
use std::sync::Arc;

use crate::audio::SPECTRUM_SIZE;
use crate::dsp::TWO_PI;

pub struct Scope {
	buf: BitMaskRB<f32>,
	pos: isize,
	r2c: Arc<dyn RealToComplex<f32>>,
	rx: HeapCons<f32>,
}

impl Scope {
	pub fn new(rx: HeapCons<f32>) -> Self {
		let mut real_planner = RealFftPlanner::<f32>::new();
		let r2c = real_planner.plan_fft_forward(SPECTRUM_SIZE);

		Scope { buf: BitMaskRB::<f32>::new(SPECTRUM_SIZE, 0.0), pos: 0, r2c, rx }
	}

	pub fn get_spectrum(&self) -> Vec<f32> {
		let mut in_buffer = self.get_buffer();

		let mut spectrum = self.r2c.make_output_vec();

		// Apply Hann window
		for (i, v) in in_buffer.iter_mut().enumerate() {
			*v *= 0.5 * (1.0 - ((TWO_PI * (i as f32)) / (SPECTRUM_SIZE as f32)).cos());
		}

		// Forward fft
		self.r2c.process(&mut in_buffer, &mut spectrum).unwrap();

		// Normalize and calculate norm
		let scale = 1.0 / (SPECTRUM_SIZE as f32).sqrt();
		spectrum.iter().map(|&z| z.norm() * scale).collect()
	}

	pub fn get_buffer(&self) -> Vec<f32> {
		let (a, b) = self.buf.as_slices(self.pos);
		[a, b].concat()
	}

	pub fn update(&mut self) {
		for sample in self.rx.pop_iter() {
			self.buf[self.pos] = sample;
			self.pos = self.buf.constrain(self.pos + 1);
		}
	}

	pub fn draw_scope(&mut self, w: f32, h: f32, color: Color, canvas: &mut Canvas<Renderer>) {
		let buffer = self.get_buffer();

		let scale_x = 3.0 * w / (buffer.len() as f32);
		let scale_y = h * 0.5;

		// calculate trigger position
		let max_val = buffer.iter().fold(0., |max, &val| val.abs().max(max));

		let threshold = 0.3 * max_val + 0.01;
		let mut x_trigger = 0.0;

		let mut schmitt = true;

		for (i, &v) in buffer.iter().enumerate() {
			if schmitt {
				if v < -threshold {
					schmitt = false;
				}
			} else if v > threshold {
				x_trigger = (i as f32) * scale_x;
				break;
			}

			if i > 200 {
				break;
			}
		}

		// draw waveform
		let mut signal_path = Path::new();
		let mut first = true;

		for (i, v) in buffer.iter().enumerate().step_by(4) {
			let x = -x_trigger + (i as f32) * scale_x;
			let y = h * 0.5 - v * scale_y;

			if y > w {
				break;
			}

			if first {
				signal_path.move_to(x, y);
				first = false;
			} else {
				signal_path.line_to(x, y);
			}
		}

		let mut paint = Paint::color(color);
		paint.set_line_join(femtovg::LineJoin::Round);
		paint.set_line_width(2.0);
		canvas.stroke_path(&signal_path, &paint);
	}

	pub fn draw_spectrum(
		&mut self,
		w: f32,
		h: f32,
		sample_rate: f32,
		color: Color,
		canvas: &mut Canvas<Renderer>,
	) {
		let spectrum = self.get_spectrum();

		let tx = w;
		let ty = h * 0.1;
		let scale_x = 210.0 * w / (spectrum.len() as f32);
		let scale_y = -h * 0.07;

		// draw grid
		let mut grid_path = Path::new();

		// grid position to hit C5
		let x_c = (C5_HZ / sample_rate).log2();

		for i in -3..=7 {
			let grid_x = (i as f32) + x_c;
			let px = tx + grid_x * scale_x;

			grid_path.move_to(px, 0.);
			grid_path.line_to(px, h);
		}

		let mut bg_color = color;
		bg_color.a *= 0.3;
		let mut bg_paint = Paint::color(bg_color);
		bg_paint.set_line_width(1.0);
		canvas.stroke_path(&grid_path, &bg_paint);

		// draw spectrum
		let mut signal_path = Path::new();
		let mut first = true;
		for (i, v) in spectrum.iter().enumerate() {
			let p = (i as f32) / ((spectrum.len() - 1) as f32);
			let x = tx + p.log2() * scale_x;
			let y = ty + v.log2() * scale_y;

			if first {
				signal_path.move_to(x, y);
				first = false;
			} else {
				signal_path.line_to(x, y);
			}
		}

		let mut paint = Paint::color(color);
		paint.set_line_width(1.5);
		// paint.set_line_join(femtovg::LineJoin::Round);
		canvas.stroke_path(&signal_path, &paint);
	}
}
