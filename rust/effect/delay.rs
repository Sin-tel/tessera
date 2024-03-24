use crate::dsp::delayline::DelayLine;
use crate::dsp::smooth::{SmoothBuffer, SmoothExp};
use crate::dsp::*;
use crate::effect::*;

// max length in seconds
const MAX_LEN: f32 = 1.0;

#[derive(Debug)]
pub struct Delay {
	tracks: [Track; 2],
	sample_rate: f32,
	balance: f32,
	time: f32,
	offset: f32,
	feedback: f32,
	lfo_freq: f32,
	lfo_mod: f32,
}

#[derive(Debug)]
struct Track {
	delay: SmoothExp,
	delay2: SmoothExp,
	lfo_accum: f32,
	left: bool,
	lfo: SmoothBuffer,
	delayline: DelayLine,
	dc_killer: DcKiller,
}

impl Track {
	pub fn new(sample_rate: f32, left: bool) -> Self {
		Track {
			delay: SmoothExp::new(200.0, sample_rate),
			delay2: SmoothExp::new(200.0, sample_rate),
			delayline: DelayLine::new(sample_rate, MAX_LEN),
			lfo_accum: 0.,
			left,
			lfo: SmoothBuffer::default(),
			dc_killer: DcKiller::new(sample_rate),
		}
	}
}

impl Effect for Delay {
	fn new(sample_rate: f32) -> Self {
		Delay {
			tracks: [
				Track::new(sample_rate, true),
				Track::new(sample_rate, false),
			],
			sample_rate,
			balance: 0.,
			time: 0.,
			offset: 0.,
			feedback: 0.,
			lfo_freq: 0.5,
			lfo_mod: 0.,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			let n = buf.len();

			// update modulation lfo
			track.lfo_accum += (self.lfo_freq / self.sample_rate) * n as f32;
			track.lfo_accum -= track.lfo_accum.floor();

			let lfo_mod = 0.002 * self.lfo_mod / self.lfo_freq;
			let phase = if track.left { 0. } else { 0.25 };
			let lfo = lfo_mod * sin_cheap(track.lfo_accum + phase);
			track.lfo.set(lfo);
			track.lfo.process_buffer(n);

			for (i, sample) in buf.iter_mut().enumerate() {
				let input = *sample;

				// delay is smoothed twice to get a continuous derivative
				track.delay2.set(track.delay.process());
				let delay = track.delay2.process();
				let s = track.delayline.go_back_linear(delay + track.lfo.get(i));

				track.delayline.push(softclip_cubic(
					track.dc_killer.process(input + self.feedback * s),
				));
				*sample = lerp(input, s, self.balance);
			}
		}
	}
	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.balance = value,
			1 => {
				self.time = value;
				self.update_delay();
			}
			2 => {
				self.offset = value;
				self.update_delay();
			}
			3 => self.feedback = value,
			4 => self.lfo_freq = value,
			5 => self.lfo_mod = value,

			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}

impl Delay {
	fn update_delay(&mut self) {
		let multiplier = pow2_cheap(self.offset);
		self.tracks[0].delay.set(self.time * multiplier);
		self.tracks[1].delay.set(self.time / multiplier);
	}
}
