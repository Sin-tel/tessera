use std::iter::zip;

use crate::dsp::env::AttackRelease;
use crate::dsp::smooth::*;
use crate::dsp::*;
use crate::instrument::*;

#[derive(Debug)]
pub struct Polysine {
	voices: Vec<Voice>,
	sample_rate: f32,
}

const N_VOICES: usize = 4;

#[derive(Debug, Default)]
struct Voice {
	accum: f32,
	freq: SmoothExp,
	vel: AttackRelease,
	note_on: bool,
}

impl Voice {
	fn new(sample_rate: f32) -> Self {
		let mut vel = AttackRelease::new(3.0, 1200., sample_rate);
		vel.set_immediate(0.);
		Self {
			freq: SmoothExp::new(2.0, sample_rate),
			vel,
			..Default::default()
		}
	}
	fn process(&mut self, buffer: &mut [f32]) {
		for sample in buffer {
			let vel = self.vel.process();
			let f = self.freq.process();

			self.accum += f;
			self.accum = self.accum - self.accum.floor();

			let mut out = (self.accum * TWO_PI).sin();
			out *= vel;

			*sample += out;
		}
	}
}

impl Instrument for Polysine {
	fn new(sample_rate: f32) -> Self {
		let mut voices = Vec::with_capacity(N_VOICES);
		for _ in 0..N_VOICES {
			voices.push(Voice::new(sample_rate));
		}

		Polysine {
			voices,
			sample_rate,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for voice in &mut self.voices {
			voice.process(bl);
		}

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			*r = *l;
		}
	}

	fn cv(&mut self, _pitch: f32, _: f32, _id: usize) {
		// let p = pitch_to_hz(pitch) / self.sample_rate;
		// self.freq.set(p);
	}

	fn note(&mut self, pitch: f32, vel: f32, id: usize) {
		let opt = self.voices.get_mut(id);
		if let Some(voice) = opt {
			if vel == 0.0 {
				voice.vel.set(0.0);
				voice.note_on = false;
			} else {
				let p = pitch_to_hz(pitch) / self.sample_rate;

				voice.freq.set(p);
				if !voice.note_on {
					voice.freq.immediate();
				}
				voice.note_on = true;
				voice.vel.set(vel * 0.5);
			}
		} else {
			eprintln!("Tried to play voice {id}");
		}
	}
	#[allow(clippy::match_single_binding)]
	fn set_parameter(&mut self, index: usize, _value: f32) {
		match index {
			_ => eprintln!("Parameter with index {index} not found"),
		}
	}
}
