use crate::defs::*;
use crate::dsp::*;
use crate::instrument::*;

#[derive(Debug, Default)]
pub struct Sine {
	accum: f32,
	freq: Smoothed<Mono>,
	vel: SmoothedEnv<Mono>,
	sample_rate: f32,
	prev: f32,
	pub feedback: f32,
}

impl Instrument for Sine {
	fn new(sample_rate: f32) -> Sine {
		Sine {
			freq: Smoothed::new(Mono(0.0), 50.0, sample_rate),
			vel: SmoothedEnv::new(Mono(0.0), 50.0, 20.0, sample_rate),
			sample_rate,
			..Default::default()
		}
	}

	fn cv(&mut self, pitch: f32, vel: f32) {
		let p = pitch_to_f(pitch, self.sample_rate);
		self.freq.set(Mono(p));
		self.vel.set(Mono(vel));
	}

	fn process(&mut self, buffer: &mut [Stereo]) {
		for sample in buffer.iter_mut() {
			self.vel.update();
			self.freq.update();
			self.accum += self.freq.value.0;
			self.accum = self.accum.fract();
			let mut out = (self.accum * TWO_PI + self.feedback * self.prev).sin();
			out *= self.vel.value;

			self.prev = out;

			*sample = Stereo([out, out])
		}
	}

	fn note(&mut self, pitch: f32, vel: f32, _id: usize) {
		let p = pitch_to_f(pitch, self.sample_rate);
		self.freq.set_hard(Mono(p));

		// self.vel.set_hard(vel);
		// self.accum = 0.0;

		if self.vel.value < Mono(0.01) {
			self.vel.set_hard(Mono(vel));
			self.accum = 0.0;
		} else {
			self.vel.set(Mono(vel));
		}
	}
}
