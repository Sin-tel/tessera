use crate::dsp::simper::Filter;
use crate::dsp::*;
use crate::instrument::*;
use fastrand::Rng;
use std::iter::zip;

// TODO: replace differentiation with more gentle filter to reduce register difference
// TODO: same note retrigger logic

#[derive(Debug)]
pub struct Epiano {
	voices: Vec<Voice>,
	sample_rate: f32,
	dc_killer: DcKiller,
	rng: Rng,
	x0: f32,
	y0: f32,
	gain: f32,
	wobble: f32,
	bell: f32,
}

const N_VOICES: usize = 16;

#[derive(Debug)]
struct Voice {
	active: bool,
	note_on: bool,
	timer: usize,
	hammer_freq: f32,
	hammer_phase: f32,
	vel: f32,
	freq: [f32; 4],
	filter: [Filter; 4],
	prev: f32,
	gain: f32,
	gain_recip: f32,
	wobble: f32,
	bell: f32,
}

impl Voice {
	fn new(sample_rate: f32) -> Self {
		let filter = [
			Filter::new(sample_rate),
			Filter::new(sample_rate),
			Filter::new(sample_rate),
			Filter::new(sample_rate),
		];

		Self {
			active: false,
			note_on: false,
			timer: 0,
			hammer_freq: 0.01,
			hammer_phase: 0.,
			prev: 0.,
			vel: 0.,
			freq: [0.; 4],
			filter,
			gain: 1.,
			gain_recip: 1.,
			wobble: 1.,
			bell: 1.,
		}
	}
}

impl Instrument for Epiano {
	fn new(sample_rate: f32) -> Self {
		let mut voices = Vec::with_capacity(N_VOICES);
		for _ in 0..N_VOICES {
			voices.push(Voice::new(sample_rate));
		}

		Epiano {
			sample_rate,
			voices,
			dc_killer: DcKiller::new(sample_rate),
			rng: Rng::new(),
			x0: 0.5,
			y0: 1.0,
			gain: 1.,
			wobble: 1.,
			bell: 1.,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for voice in self.voices.iter_mut().filter(|v| v.active) {
			for sample in bl.iter_mut() {
				let mut hammer = 0.;
				if voice.hammer_phase < 1.0 {
					voice.hammer_phase += voice.hammer_freq;

					let x = voice.hammer_phase;
					hammer = (x * (4. - 4. * x)).powi(3) * voice.vel;
				}

				let mut s = [0.; 4];
				for (i, v) in s.iter_mut().enumerate() {
					*v = voice.filter[i].process(hammer);
				}

				let mut y = 0.20 * voice.wobble * s[1];
				let mut x = s[0] + 0.20 * voice.wobble * s[1] + voice.bell * (s[2] + s[3]);

				x = voice.gain * x - self.x0;
				y = voice.gain * y - self.y0;

				// pickup magnetic field
				let field = (x * x + y * y).sqrt().recip();

				// output voltage is derivative of magnetic field
				let diff = field - voice.prev;

				voice.prev = field;

				let out = diff * 20.0 * voice.gain_recip + 0.05 * hammer;

				*sample += out;
			}

			if !voice.note_on {
				voice.timer += 1;
				if voice.timer > 1000 {
					voice.active = false;
				}
			}
		}

		for s in bl.iter_mut() {
			*s = self.dc_killer.process(*s);
		}

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			*r = *l;
		}
	}

	fn cv(&mut self, _pitch: f32, _pres: f32, _id: usize) {
		// epiano doesn't respond to pitch & pressure
	}

	fn note_on(&mut self, pitch: f32, vel: f32, id: usize) {
		let voice = &mut self.voices[id];
		let f = pitch_to_hz(pitch);

		voice.vel = 0.004 * vel * (self.sample_rate / f);

		voice.hammer_freq = f / self.sample_rate;
		voice.hammer_freq = voice.hammer_freq.min(0.015);
		voice.hammer_freq *= 1. + vel;

		voice.freq[0] = f;
		voice.freq[1] = f + 0.4 + 0.4 * self.rng.f32();
		voice.freq[2] = f * 4. + 1200. * self.rng.f32();
		voice.freq[3] = f * 6. + 1600. * self.rng.f32();

		voice.filter[0].set_bandpass(voice.freq[0], voice.freq[0] * 6.0);
		voice.filter[1].set_bandpass(voice.freq[1], voice.freq[1] * 4.0);
		voice.filter[2].set_bandpass(voice.freq[2], voice.freq[2] * 0.4);
		voice.filter[3].set_bandpass(voice.freq[3], voice.freq[3] * 0.3);

		voice.filter.iter_mut().for_each(Filter::reset_state);
		voice.filter.iter_mut().for_each(Filter::immediate);

		voice.hammer_phase = 0.;

		voice.prev = (self.x0 * self.x0 + self.y0 * self.y0).sqrt().recip();

		voice.gain = self.gain;
		voice.gain_recip = self.gain.recip();
		voice.wobble = self.wobble;
		voice.bell = self.bell;

		voice.active = true;
		voice.note_on = true;
		voice.timer = 0;
	}

	fn note_off(&mut self, id: usize) {
		let voice = &mut self.voices[id];
		voice.note_on = false;
		for (i, v) in voice.filter.iter_mut().enumerate() {
			v.set_bandpass(voice.freq[i], voice.freq[i] * 0.12);
		}
	}

	#[allow(clippy::match_single_binding)]
	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.gain = from_db(value),
			1 => self.wobble = value,
			2 => self.bell = value * value,
			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}
