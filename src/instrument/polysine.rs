use crate::dsp::env::AttackRelease;
use crate::dsp::smooth::*;
use crate::dsp::*;
use crate::instrument::*;

#[derive(Debug)]
pub struct Polysine {
	voices: Vec<Voice>,
	sample_rate: f32,
	dc_killer: DcKiller,

	feedback: f32,
}

const N_VOICES: usize = 16;

#[derive(Debug)]
struct Voice {
	accum: f32,
	freq: Smooth,
	vel: AttackRelease,
	note_on: bool,
	active: bool,
	prev: f32,
}

impl Voice {
	fn new(sample_rate: f32) -> Self {
		let mut vel = AttackRelease::new(3.0, 500., sample_rate);
		vel.set_immediate(0.);
		Self {
			freq: Smooth::new(1., 10.0, sample_rate),
			vel,
			accum: 0.,
			note_on: false,
			active: false,
			prev: 0.,
		}
	}
}

impl Instrument for Polysine {
	fn new(sample_rate: f32) -> Self {
		let mut voices = Vec::with_capacity(N_VOICES);
		for _ in 0..N_VOICES {
			voices.push(Voice::new(sample_rate));
		}

		Polysine { voices, dc_killer: DcKiller::new(sample_rate), sample_rate, feedback: 0. }
	}

	fn voice_count(&self) -> usize {
		N_VOICES
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for voice in self.voices.iter_mut().filter(|v| v.active) {
			for sample in bl.iter_mut() {
				let vel = voice.vel.process();
				let f = voice.freq.process();

				voice.accum += f;
				voice.accum = voice.accum - voice.accum.floor();

				let mut out = (voice.accum * TWO_PI + voice.prev * self.feedback).sin();
				out *= vel;

				voice.prev = lerp(voice.prev, out, 0.5);

				*sample += out * 0.5;
			}
			if !voice.note_on && voice.vel.get() < 1e-4 {
				voice.active = false;
			}
		}

		for s in bl.iter_mut() {
			*s = self.dc_killer.process(*s);
		}

		br.copy_from_slice(bl);
	}

	fn pitch(&mut self, pitch: f32, id: usize) {
		let voice = &mut self.voices[id];
		let p = pitch_to_hz(pitch) / self.sample_rate;
		voice.freq.set(p);
	}

	fn pressure(&mut self, pressure: f32, id: usize) {
		let voice = &mut self.voices[id];
		voice.vel.set(pressure);
	}

	fn note_on(&mut self, pitch: f32, vel: f32, id: usize) {
		let voice = &mut self.voices[id];
		let p = pitch_to_hz(pitch) / self.sample_rate;

		voice.freq.set(p);
		if !voice.note_on {
			voice.freq.immediate();
		}
		voice.note_on = true;
		voice.active = true;
		voice.vel.set(vel);
	}

	fn note_off(&mut self, id: usize) {
		let voice = &mut self.voices[id];
		voice.vel.set(0.0);
		voice.note_on = false;
	}
	fn flush(&mut self) {
		for v in &mut self.voices {
			v.vel.set_immediate(0.);
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.feedback = value,
			1 => self.voices.iter_mut().for_each(|v| v.vel.set_attack(value)),
			2 => self.voices.iter_mut().for_each(|v| v.vel.set_release(value)),
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
