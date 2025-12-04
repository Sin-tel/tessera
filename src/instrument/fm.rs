use crate::dsp::env::*;
use crate::dsp::smooth::*;
use crate::dsp::*;
use crate::instrument::*;

// TODO: ADE env

#[derive(Debug)]
pub struct Fm {
	voices: Vec<Voice>,
	sample_rate: f32,
	dc_killer: DcKiller,

	feedback: f32,
	depth: f32,
	ratio: f32,
	ratio_fine: f32,
	offset: f32,
	pitch_mod: f32,
	pitch_decay: f32,
	keytrack: f32,
}

const N_VOICES: usize = 16;

#[derive(Debug)]
struct Voice {
	active: bool,
	accum: f32,
	accum2: f32,
	prev: f32,
	freq: SmoothExp,
	freq2: SmoothExp,
	env: Adsr,
	vel: f32,
	pres: AttackRelease,
	pitch_env: f32,
	bright: f32,
}

impl Voice {
	fn new(sample_rate: f32) -> Self {
		Self {
			freq: SmoothExp::new(10.0, sample_rate),
			freq2: SmoothExp::new(10.0, sample_rate),
			env: Adsr::new(sample_rate),
			pres: AttackRelease::new(80.0, 300.0, sample_rate),
			active: false,
			accum: 0.,
			accum2: 0.,
			prev: 0.,
			vel: 0.,
			pitch_env: 0.,
			bright: 0.,
		}
	}

	fn set_modulator(&mut self, ratio: f32, ratio_fine: f32, offset: f32) {
		self.freq2.set((ratio + ratio_fine) * self.freq.target() + offset);
	}
}

impl Instrument for Fm {
	fn new(sample_rate: f32) -> Self {
		let mut voices = Vec::with_capacity(N_VOICES);
		for _ in 0..N_VOICES {
			voices.push(Voice::new(sample_rate));
		}

		Fm {
			voices,
			dc_killer: DcKiller::new(sample_rate),
			sample_rate,

			feedback: 0.,
			depth: 0.,
			ratio: 0.,
			ratio_fine: 0.,
			offset: 0.,
			pitch_mod: 0.,
			pitch_decay: 0.,
			keytrack: 0.,
		}
	}

	fn voice_count(&self) -> usize {
		N_VOICES
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for voice in self.voices.iter_mut().filter(|v| v.active) {
			for sample in bl.iter_mut() {
				let env = voice.env.process();
				let pres = voice.pres.process();
				let f = voice.freq.process();
				let f2 = voice.freq2.process();

				voice.pitch_env *= self.pitch_decay;
				let pitch_mod = 1.0 + voice.pitch_env;

				voice.accum += f * pitch_mod;
				voice.accum -= voice.accum.floor();

				voice.accum2 += f2 * pitch_mod;
				voice.accum2 -= voice.accum2.floor();

				let mut prev = voice.prev;
				if self.feedback < 0.0 {
					prev *= prev;
				}
				let feedback = self.feedback.abs();
				let op2 = sin_cheap(voice.accum2 + feedback * prev);

				// filter the modulator to prevent nyquist artifacts
				// also limits aliasing in the carrier
				voice.prev = lerp(voice.prev, op2, 0.3);

				let mut out =
					sin_cheap(voice.accum + voice.bright * (self.depth + pres) * voice.prev);
				out *= env;

				*sample += 0.5 * out;
			}
			if voice.env.get() < 1e-4 {
				voice.active = false;
			}
		}

		self.dc_killer.process_block(bl);
		br.copy_from_slice(bl);
	}

	fn pitch(&mut self, pitch: f32, id: usize) {
		let voice = &mut self.voices[id];
		let p = pitch_to_hz(pitch) / self.sample_rate;
		voice.freq.set(p);
		voice.set_modulator(self.ratio, self.ratio_fine, self.offset);
	}

	fn pressure(&mut self, pressure: f32, id: usize) {
		let voice = &mut self.voices[id];
		voice.pres.set(pressure);
	}

	fn note_on(&mut self, pitch: f32, vel: f32, id: usize) {
		let voice = &mut self.voices[id];
		let f = pitch_to_hz(pitch) / self.sample_rate;
		voice.freq.set_immediate(f);
		voice.set_modulator(self.ratio, self.ratio_fine, self.offset);
		voice.freq2.immediate();
		voice.env.note_on(vel);
		voice.vel = vel;
		voice.bright = 1.0 - ((pitch - 60.) * 0.03 * self.keytrack).clamp(-1., 0.95);

		// phase reset
		if !voice.active {
			voice.accum = 0.0;
			voice.accum2 = 0.0;
		}
		voice.active = true;

		voice.pitch_env = self.pitch_mod;
	}

	fn note_off(&mut self, id: usize) {
		let voice = &mut self.voices[id];
		voice.env.note_off();
	}
	fn flush(&mut self) {
		for v in &mut self.voices {
			v.env.reset();
			v.active = false;
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.feedback = value * 0.5,
			1 => self.depth = value,
			2 => {
				self.ratio = value;
				self.voices
					.iter_mut()
					.for_each(|v| v.set_modulator(self.ratio, self.ratio_fine, self.offset));
			},
			3 => {
				self.ratio_fine = value;
				self.voices
					.iter_mut()
					.for_each(|v| v.set_modulator(self.ratio, self.ratio_fine, self.offset));
			},
			4 => {
				self.offset = value / self.sample_rate;
				self.voices
					.iter_mut()
					.for_each(|v| v.set_modulator(self.ratio, self.ratio_fine, self.offset));
			},
			5 => self.voices.iter_mut().for_each(|v| v.env.set_attack(value)),
			6 => self.voices.iter_mut().for_each(|v| v.env.set_decay(value)),
			7 => self.voices.iter_mut().for_each(|v| v.env.set_sustain(value)),
			8 => self.voices.iter_mut().for_each(|v| v.env.set_release(value)),
			9 => self.pitch_mod = 0.5 * value + 10. * value.max(0.).powi(3),
			10 => self.pitch_decay = 1.0 - time_constant(value, self.sample_rate),
			11 => self.keytrack = value,

			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}
