use crate::instrument::*;
use crate::dsp::*;
use crate::dsp::simper::*;

#[derive(Debug, Default)]
pub struct Sine {
	accum: f32,
	freq: Smoothed,
	vel: SmoothedEnv,
	sample_rate: f32,
	prev: f32,
	pub feedback: f32,
	filter: Filter,
}

impl Instrument for Sine {
	fn new(sample_rate: f32) -> Sine {
		let mut filter = Filter::new(sample_rate);
		filter.set(FilterSettings::HighShelf(300.0, 5.0, -12.0));
		Sine {
			freq: Smoothed::new(0.0, 50.0 / sample_rate),
			vel: SmoothedEnv::new(0.0, 200.0 / sample_rate, 20.0 / sample_rate),
			sample_rate,
			filter,
			..Default::default()
		}
	}

	fn cv(&mut self, pitch: f32, vel: f32) {
		let p = pitch_to_f(pitch, self.sample_rate);
		// dbg!(p);
		self.freq.set(p);
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

			out = self.filter.process(out);

			sample.l = out;
			sample.r = out;
		}
	}

	fn note(&mut self, pitch: f32, vel: f32) {
		let p = pitch_to_f(pitch, self.sample_rate);
		self.freq.set_hard(p);
		if self.vel.value < 0.0001 {
			self.vel.set_hard(vel);
			self.accum = 0.0;
		} else {
			self.vel.set(vel);
		}
	}
}
