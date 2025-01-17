use std::iter::zip;

use crate::dsp::delayline::DelayLine;
use crate::dsp::env::AttackRelease;
use crate::dsp::onepole::OnePole;
use crate::dsp::simper::Filter;
use crate::dsp::smooth::*;
use crate::dsp::*;
use crate::instrument::*;
use fastrand::Rng;

const MAX_LEN: f32 = 0.2;

#[derive(Debug)]
pub struct Pluck {
	voices: Vec<Voice>,
	sample_rate: f32,
	// dc_killer: DcKiller,
	rng: Rng,

	position: f32,
	noise: f32,
	damp: f32,
	decay: f32,
	release: f32,
}

const N_VOICES: usize = 8;

#[derive(Debug)]
struct Voice {
	freq: SmoothExp,
	note_on: bool,
	active: bool,

	hammer_x: f32,
	hammer_v: f32,
	position: f32,
	decay: f32,
	release: f32,
	off_time: f32,
	mute_state: f32,

	delay_l: DelayLine,
	delay_r: DelayLine,

	lp: OnePole,
	ap: Filter,
	noise_filter: Filter,
	mute_filter: Filter,
}

impl Voice {
	fn new(sample_rate: f32) -> Self {
		let mut vel = AttackRelease::new(3., 500., sample_rate);
		vel.set_immediate(0.);

		let mut mute_filter = Filter::new(sample_rate);
		mute_filter.set_highshelf(1000., BUTTERWORTH_Q, -4.);

		let mut noise_filter = Filter::new(sample_rate);
		noise_filter.set_lowpass(5000., 0.5);

		Self {
			freq: SmoothExp::new(10., sample_rate),
			note_on: false,
			active: false,

			hammer_x: 0.,
			hammer_v: 0.,
			position: 0.,
			decay: 0.,
			release: 0.,
			off_time: 0.,
			mute_state: 0.,

			delay_l: DelayLine::new(sample_rate, MAX_LEN),
			delay_r: DelayLine::new(sample_rate, MAX_LEN),

			// lp: Filter::new(sample_rate),
			lp: OnePole::new(sample_rate),
			ap: Filter::new(sample_rate),
			noise_filter,
			mute_filter,
		}
	}
}

impl Instrument for Pluck {
	fn new(sample_rate: f32) -> Self {
		let mut voices = Vec::with_capacity(N_VOICES);
		for _ in 0..N_VOICES {
			voices.push(Voice::new(sample_rate));
		}

		Pluck {
			voices,
			// dc_killer: DcKiller::new(sample_rate),
			rng: Rng::new(),
			sample_rate,

			decay: 0.0,
			release: 0.0,
			damp: 0.0,
			position: 0.2,
			noise: 0.0,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for voice in self.voices.iter_mut().filter(|v| v.active) {
			for sample in bl.iter_mut() {
				let f = voice.freq.process();

				// TODO: proper tuning table
				let tun_o = 0.00001;
				let tun_f = 1.07186;
				let len = tun_o + tun_f / f;

				let pos = voice.position;

				let mut right = -voice.delay_r.go_back_cubic((1.0 - pos) * len);
				let mut left = -voice.delay_l.go_back_cubic(pos * len);

				right = voice.ap.process(right);
				left = voice.lp.process(left);

				let s = right + left;

				let mut hf = 0.0;
				let mut hf_n = 0.0;
				if s > voice.hammer_x {
					let nse = voice.noise_filter.process(self.rng.f32() - 0.5);
					let h = s - voice.hammer_x;

					hf = 0.5 * h.powi(2);
					hf_n = hf * (1.0 + self.noise * nse);
					// hf = hf.min(1.0);
				}

				voice.hammer_v += 0.00001 * hf * f;
				voice.hammer_x += voice.hammer_v;

				if voice.note_on {
					right *= voice.decay;
				} else {
					voice.mute_state = lerp(voice.mute_state, 1.0, 0.03);
					let r2 = voice.mute_filter.process(right * voice.release);
					// let r2 = nr * voice.release;

					right = lerp(right, r2, voice.mute_state);
				}

				// let rattle = -0.4;
				// if right < rattle {
				// 	right = (right - rattle) * 0.5 + rattle
				// 	// right = (right + rattle) * 0.98 - rattle
				// }

				voice.delay_l.push(right - hf_n);
				voice.delay_r.push(left * voice.decay - hf_n);

				let out = left + right;

				*sample += out * 0.2;
			}

			if !voice.note_on {
				voice.off_time += bl.len() as f32 / self.sample_rate;
				if voice.off_time > 1. {
					voice.active = false;
				}
			}
		}

		for (l, r) in zip(bl.iter_mut(), br.iter_mut()) {
			*r = *l;
		}
	}

	fn pitch(&mut self, pitch: f32, id: usize) {
		let voice = &mut self.voices[id];
		let p = pitch_to_hz(pitch);
		voice.freq.set(p);
	}

	fn pressure(&mut self, _pressure: f32, _id: usize) {}

	fn note_on(&mut self, pitch: f32, vel: f32, id: usize) {
		let voice = &mut self.voices[id];
		let f = pitch_to_hz(pitch);
		voice.freq.set(f);
		if !voice.note_on {
			voice.freq.immediate();
		}

		voice.note_on = true;
		voice.active = true;
		voice.off_time = 0.;
		voice.mute_state = 0.0;

		voice.hammer_x = 1.0;
		voice.hammer_v = -0.08 * vel;

		voice.decay = 0.99_f32.powf((1.0 - self.decay) * 120. / f);

		let mut r = 100. + 300. * self.release;
		if self.release < 0. {
			r = 100. - 110. * self.release;
		}
		voice.release = 0.9_f32.powf(r / f);

		let apf = (f * 7.0).min(18000.0);
		voice.ap.set_allpass(apf, BUTTERWORTH_Q);

		voice.position = self.position + 0.05 * (self.rng.f32() - 0.5);
		voice.position = voice.position.clamp(0.05, 0.95);

		let high_gain = -self.damp * 400. / f;
		voice.lp.set_highshelf(6000.0 - 4000.0 * self.damp, high_gain);

		voice.noise_filter.set_lowpass(1000. + 6000. * vel, 0.5);

		// let mute_gain = -6. * (1.0 - self.release);
		let mut mute_gain = 0.;
		if self.release < 0. {
			mute_gain = 8. * self.release;
		}
		voice.mute_filter.set_highshelf(1000., BUTTERWORTH_Q, mute_gain);
	}

	fn note_off(&mut self, id: usize) {
		let voice = &mut self.voices[id];
		voice.note_on = false;
	}

	fn flush(&mut self) {}

	fn set_parameter(&mut self, index: usize, value: f32) {
		match index {
			0 => self.decay = value,
			1 => self.release = value,
			2 => self.damp = value,
			3 => self.position = value,
			4 => self.noise = 0.5 * value * value,
			// 1 => self.voices.iter_mut().for_each(|v| v.vel.set_attack(value)),
			// 2 => self.voices.iter_mut().for_each(|v| v.vel.set_release(value)),
			_ => log_warn!("Parameter with index {index} not found"),
		}
	}
}
