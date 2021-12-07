use crate::instrument::*;
use crate::math::*;

#[derive(Debug, Default)]
pub struct Sine {
	accum: f32,
	freq: Smoothed,
	vel: SmoothedEnv,
	sample_rate: f32,
	prev: f32,
	pub feedback: f32,
}

impl Instrument for Sine {
	fn new(sample_rate: f32) -> Sine {
		Sine {
			freq: Smoothed::new(0.0, 50.0 / sample_rate),
			vel: SmoothedEnv::new(0.0, 200.0 / sample_rate, 20.0 / sample_rate),
			sample_rate,
			..Default::default()
		}
	}

	fn cv(&mut self, pitch: f32, vel: f32) {
		// if vel != 0.0 { // note off
		self.freq.set(pitch_to_f(pitch, self.sample_rate));
		// }
		self.vel.set(vel);
	}

	fn process(&mut self, buffer: &mut [StereoSample]) {
		for sample in buffer.iter_mut() {
			self.vel.update();
			self.freq.update();
			self.accum += self.freq.value;
			self.accum = self.accum.fract();
			let mut out = (self.accum * TWO_PI + self.feedback * self.prev).sin();
			out *= self.vel.value;

			self.prev = out;

			// self.accum += 440.0 / self.sample_rate;
			// let mut out = (self.accum * TWO_PI + 0.5 * self.prev).sin() * 0.1;
			// out *= 0.2;
			sample.l = out;
			sample.r = out;
		}
	}

	fn note_on(&mut self, pitch: f32, vel: f32) {
		self.freq.set_hard(pitch_to_f(pitch, self.sample_rate));
		if self.vel.value < 0.0001 {
			self.vel.set_hard(vel);
			self.accum = 0.0;
		} else {
			self.vel.set(vel);
		}
	}
}
