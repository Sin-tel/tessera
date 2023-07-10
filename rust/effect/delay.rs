use crate::dsp::delayline::DelayLine;
// use crate::dsp::simper::Filter;
use crate::dsp::smooth::SmoothLinear;
use crate::dsp::*;
use crate::effect::Effect;

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
	delay: SmoothLinear,
	lfo_accum: f32,
	lfo_phase: f32,
	lfo_freq: f32,
	lfo_mod: SmoothLinear,
	delayline: DelayLine,
}

impl Track {
	pub fn new(sample_rate: f32, lfo_phase: f32) -> Self {
		Track {
			delay: SmoothLinear::new(200.0, sample_rate),
			delayline: DelayLine::new(sample_rate, MAX_LEN),
			lfo_accum: 0.,
			lfo_phase,
			lfo_freq: 0.,
			lfo_mod: SmoothLinear::new(50.0, sample_rate),
		}
	}
}

impl Delay {
	fn update_delay(&mut self) {
		let multiplier = pow2_cheap(self.offset);
		self.tracks[0].delay.set(self.time * multiplier);
		self.tracks[1].delay.set(self.time / multiplier);
	}

	fn update_lfo(&mut self) {
		self.tracks.iter_mut().for_each(|track| {
			track.lfo_freq = self.lfo_freq / self.sample_rate;
			track.lfo_mod.set(0.005 * self.lfo_mod / self.lfo_freq);
		})
	}
}

impl Effect for Delay {
	fn new(sample_rate: f32) -> Self {
		Delay {
			tracks: [Track::new(sample_rate, 0.), Track::new(sample_rate, 0.25)],
			sample_rate,
			balance: 0.,
			time: 0.,
			offset: 0.,
			feedback: 0.,
			lfo_freq: 0.,
			lfo_mod: 0.,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			for sample in buf.iter_mut() {
				let input = *sample;

				track.lfo_accum += track.lfo_freq;
				let lfo_mod = track.lfo_mod.process();

				let lfo = lfo_mod * sin_cheap(track.lfo_accum + track.lfo_phase);

				let delay = track.delay.process();
				let s = track.delayline.go_back_linear(delay + lfo);

				track.delayline.push(input + self.feedback * s);
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
			4 => {
				self.lfo_freq = value;
				self.update_lfo();
			}
			5 => {
				self.lfo_mod = value;
				self.update_lfo();
			}
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}
