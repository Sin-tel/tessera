use crate::dsp::simper::Filter;
use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use crate::effect::Effect;
use crate::log::log_warn;
use crate::worker::RequestData;

const STAGES: usize = 4;
const Q_FACTOR: f32 = 0.5;

#[derive(Debug)]
struct Track {
	filters: [Filter; STAGES],
	balance: Smooth,
	feedback: Smooth,
	lfo_phase_offset: f32,
}

impl Track {
	fn new(sample_rate: f32, left: bool) -> Self {
		let filters = std::array::from_fn(|_| Filter::new(sample_rate));
		let lfo_phase_offset = if left { 0.25 } else { 0.0 };

		Self {
			filters,
			balance: Smooth::new(0.5, 25.0, sample_rate),
			feedback: Smooth::new(0.0, 25.0, sample_rate),
			lfo_phase_offset,
		}
	}
}

pub struct Phaser {
	tracks: [Track; 2],
	lfo_accum: f32,
	sample_rate: f32,
	f_base: f32,
	lfo_rate: f32,
	lfo_depth: f32,
}

impl Effect for Phaser {
	fn new(sample_rate: f32) -> Self {
		Self {
			tracks: [Track::new(sample_rate, true), Track::new(sample_rate, false)],
			lfo_accum: 0.0,
			sample_rate,
			f_base: 1000.0,
			lfo_rate: 0.5,
			lfo_depth: 0.0,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let n = buffer[0].len();
		// update LFO per block
		self.lfo_accum += (n as f32) * self.lfo_rate / self.sample_rate;
		if self.lfo_accum >= 1.0 {
			self.lfo_accum -= 1.0;
		}
		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			let lfo = 0.9 * self.lfo_depth * sin_cheap(self.lfo_accum + track.lfo_phase_offset);
			for ap in &mut track.filters {
				let f = self.f_base * (1.0 + lfo);
				ap.set_allpass(f, Q_FACTOR);
			}

			for sample in buf.iter_mut() {
				let input = *sample;

				let balance = track.balance.process();
				let feedback = track.feedback.process();

				// compute instantaneous response y = g*x + s
				let mut g_total = 1.0;
				let mut s_total = 0.0;
				for ap in &mut track.filters {
					let (g, s) = ap.predict();
					g_total *= g;
					s_total = s_total * g + s;
				}

				// solve for feedback loop
				let chain_in = (input - feedback * s_total) / (1.0 + feedback * g_total);

				// update state
				let mut x = softclip(chain_in);
				for ap in &mut track.filters {
					x = ap.process(x);
				}

				let out = -x;
				*sample = lerp(input, out, balance);
			}
		}
	}

	fn flush(&mut self) {
		self.lfo_accum = 0.0;
		for t in &mut self.tracks {
			for f in &mut t.filters {
				f.reset_state();
			}
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.tracks.iter_mut().for_each(|t| t.balance.set(0.5 * value)),
			1 => self.f_base = value,
			2 => self.tracks.iter_mut().for_each(|t| t.feedback.set(value)),
			3 => self.lfo_rate = value,
			4 => self.lfo_depth = value,
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
