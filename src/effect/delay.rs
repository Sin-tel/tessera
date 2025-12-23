use crate::dsp::delayline::DelayLine;
use crate::dsp::simper::Filter;
use crate::dsp::smooth::{LinearBuffer, SmoothBuffer};
use crate::dsp::*;
use crate::effect::*;
use crate::worker::RequestData;

// max length in seconds
const MAX_LEN: f32 = 1.0;

// TODO: using simper messes up initial state. Make a 2x smoother instead.

#[derive(Debug)]
pub struct Delay {
	tracks: [Track; 2],
	sample_rate: f32,
	balance: SmoothBuffer,
	time: f32,
	offset: f32,
	feedback: SmoothBuffer,
	lfo_freq: f32,
	lfo_mod: f32,
}

#[derive(Debug)]
struct Track {
	delay: f32,
	delay_f: Filter,
	lfo_accum: f32,
	left: bool,
	lfo: LinearBuffer,
	delayline: DelayLine,
	dc_killer: DcKiller,
}

impl Track {
	pub fn new(sample_rate: f32, left: bool) -> Self {
		let mut delay_f = Filter::new(sample_rate);
		delay_f.set_lowpass(2.0, 0.5);
		delay_f.immediate();
		Track {
			delay: 0.4,
			delay_f,
			delayline: DelayLine::new(sample_rate, MAX_LEN),
			lfo_accum: 0.,
			left,
			lfo: LinearBuffer::new(0.),
			dc_killer: DcKiller::new(sample_rate),
		}
	}
}

impl Effect for Delay {
	fn new(sample_rate: f32) -> Self {
		Delay {
			tracks: [Track::new(sample_rate, true), Track::new(sample_rate, false)],
			sample_rate,
			balance: SmoothBuffer::new(0., 25.0, sample_rate),
			time: 0.4,
			offset: 0.,
			feedback: SmoothBuffer::new(0., 25.0, sample_rate),
			lfo_freq: 0.5,
			lfo_mod: 0.,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		// calculate LFOS
		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			let n = buf.len();

			// update modulation lfo
			track.lfo_accum += (self.lfo_freq / self.sample_rate) * n as f32;
			track.lfo_accum -= track.lfo_accum.floor();

			let lfo_mod = 0.002 * self.lfo_mod / self.lfo_freq;
			let phase = if track.left { 0. } else { 0.25 };
			let lfo = lfo_mod * sin_cheap(track.lfo_accum + phase);
			track.lfo.set(lfo);
			track.lfo.process_block(n);
		}

		let len = buffer[0].len();
		assert!(len <= MAX_BUF_SIZE);
		self.balance.process_block(len);
		self.feedback.process_block(len);

		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			for (i, sample) in buf.iter_mut().enumerate() {
				let input = *sample;

				let delay = track.delay_f.process(track.delay);
				let s = track.delayline.go_back_linear(delay + track.lfo.get(i));

				track.delayline.push(softclip_cubic(
					track.dc_killer.process(input + self.feedback.get(i) * s),
				));
				*sample = lerp(input, s, self.balance.get(i));
			}
		}
	}

	fn flush(&mut self) {
		self.tracks[0].delayline.flush();
		self.tracks[1].delayline.flush();
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.balance.set(value),
			1 => {
				self.time = value;
				self.update_delay();
			},
			2 => {
				self.offset = value;
				self.update_delay();
			},
			3 => self.feedback.set(value),
			4 => self.lfo_freq = value,
			5 => self.lfo_mod = value,

			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}

impl Delay {
	fn update_delay(&mut self) {
		let multiplier = pow2_cheap(self.offset);
		self.tracks[0].delay = self.time * multiplier;
		self.tracks[1].delay = self.time / multiplier;
	}
}
