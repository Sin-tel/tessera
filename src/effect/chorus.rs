use crate::dsp::delayline::DelayLine;
use crate::dsp::simper::Filter;
use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use crate::effect::*;
use crate::worker::RequestData;

const TIME: f32 = 0.009;

#[derive(Debug)]
pub struct Chorus {
	tracks: [Track; 2],
	sample_rate: f32,
	vibrato: bool,
}

#[derive(Debug)]
struct Track {
	lfo_accum: f32,
	left: bool,
	delayline: DelayLine,
	highpass: Filter,
	balance: Smooth,
	lfo_freq: Smooth,
	lfo_mod: Smooth,
}

impl Track {
	pub fn new(sample_rate: f32, left: bool) -> Self {
		let mut highpass = Filter::new(sample_rate);
		highpass.set_highpass(80.0, BUTTERWORTH_Q);

		Track {
			delayline: DelayLine::new(sample_rate, 3.0 * TIME),
			lfo_accum: 0.,
			left,
			highpass,
			balance: Smooth::new(0., 25.0, sample_rate),
			lfo_freq: Smooth::new(0., 200.0, sample_rate),
			lfo_mod: Smooth::new(0., 200.0, sample_rate),
		}
	}
}

impl Effect for Chorus {
	fn new(sample_rate: f32) -> Self {
		Chorus {
			tracks: [Track::new(sample_rate, true), Track::new(sample_rate, false)],
			sample_rate,
			vibrato: false,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			for sample in buf.iter_mut() {
				let lfo_freq = track.lfo_freq.process();

				let mut lfo_mod = track.lfo_mod.process() / f32::max(1.0, lfo_freq);
				if !self.vibrato {
					lfo_mod *= 0.5;
				}

				// update modulation lfo
				track.lfo_accum += lfo_freq / self.sample_rate;
				track.lfo_accum -= track.lfo_accum.floor();

				let phase = if track.left || self.vibrato { 0. } else { 0.25 };

				let lfo = lfo_mod * sin_cheap(track.lfo_accum + phase);

				let input = *sample;

				let d = track.delayline.go_back_cubic(TIME + lfo);

				let s = if self.vibrato { d } else { input + 0.71 * track.highpass.process(d) };

				track.delayline.push(input);

				let balance = track.balance.process();
				*sample = lerp(input, s, balance);
			}
		}
	}

	fn flush(&mut self) {
		self.tracks[0].delayline.flush();
		self.tracks[1].delayline.flush();
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.tracks.iter_mut().for_each(|t| t.balance.set(value)),
			1 => self.tracks.iter_mut().for_each(|t| t.lfo_freq.set(value)),
			2 => self.tracks.iter_mut().for_each(|t| t.lfo_mod.set(TIME * value)),
			3 => self.vibrato = value > 0.5,
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
