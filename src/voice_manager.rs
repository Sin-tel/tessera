use crate::audio::MAX_BUF_SIZE;
use crate::dsp::{MuteState, PeakMeter, time_constant};
use crate::instrument::Instrument;
use crate::meters::MeterHandle;
use std::collections::VecDeque;

pub type Token = u32;

#[derive(Clone, Copy, Debug)]
struct Voice {
	token: Token,
	pitch: f32,
	vel: f32,
	key_down: bool,
	active: bool,
	age: u64,
}

impl Default for Voice {
	fn default() -> Self {
		Self { token: 0, pitch: 0.0, vel: 0.0, key_down: false, age: 0, active: false }
	}
}

impl Voice {
	fn new(token: Token, pitch: f32, vel: f32) -> Self {
		Self { token, pitch, vel, key_down: true, active: true, ..Default::default() }
	}
}

pub struct VoiceManager {
	pub instrument: Box<dyn Instrument + Send>,
	voices: Vec<Voice>,
	queue: VecDeque<Voice>,
	sustain: bool,

	peak: PeakMeter,
	meter_handle: MeterHandle,

	mute: bool,
	state: MuteState,
	gain: f32,
	smoothing_f: f32,
}

impl VoiceManager {
	pub fn new(
		sample_rate: f32,
		instrument: Box<dyn Instrument + Send>,
		meter_handle: MeterHandle,
	) -> Self {
		let voice_count = instrument.voice_count();
		Self {
			instrument,
			voices: vec![Voice::default(); voice_count],
			queue: VecDeque::with_capacity(8),
			sustain: false,

			peak: PeakMeter::new(sample_rate),
			meter_handle,

			mute: false,
			state: MuteState::Active,
			gain: 1.0,
			smoothing_f: time_constant(15.0, sample_rate),
		}
	}

	fn get_index(&self, token: Token) -> Option<usize> {
		self.voices.iter().position(|v| v.token == token)
	}

	fn get_queue_index(&self, token: Token) -> Option<usize> {
		self.queue.iter().position(|v| v.token == token)
	}

	pub fn note_on(&mut self, token: Token, pitch: f32, offset: f32, vel: f32) {
		if self.mute {
			return;
		}
		let mut playing_best_i = None;
		let mut playing_min_dist = f32::MAX;

		let mut released_best_i = None;
		let mut released_max_age = 0;

		for (i, v) in self.voices.iter_mut().enumerate() {
			v.age += 1;

			if v.key_down {
				// Minimize pitch distance
				let dist = (v.pitch - pitch).abs();
				if dist < playing_min_dist {
					playing_min_dist = dist;
					playing_best_i = Some(i);
				}
			} else {
				//  Oldest released voice
				if v.age >= released_max_age {
					released_max_age = v.age;
					released_best_i = Some(i);
				}
			}
		}

		let target_i = released_best_i.or(playing_best_i).expect("Should find a voice");

		if self.voices[target_i].key_down {
			// Drop the oldest stolen voice if capacity is full
			if self.queue.len() == self.queue.capacity() {
				self.queue.pop_front();
			}
			self.queue.push_back(self.voices[target_i]);
		}

		let voice = Voice::new(token, pitch, vel);
		self.voices[target_i] = voice;

		self.instrument.note_on(pitch + offset, vel, target_i);
	}

	pub fn note_off(&mut self, token: Token) {
		if self.mute {
			return;
		}
		let Some(i) = self.get_index(token) else {
			// Voice was already dead
			if let Some(pos) = self.get_queue_index(token) {
				self.queue.remove(pos);
			}
			return;
		};

		if self.queue.is_empty() {
			self.voices[i].key_down = false;
			if !self.sustain {
				self.voices[i].active = false;
				self.instrument.note_off(i);
			}
		} else {
			// Revive latest dead voice from queue
			let recovered = self.queue.pop_back().unwrap();
			self.voices[i] = recovered;

			self.instrument.note_on(self.voices[i].pitch, self.voices[i].vel, i);
		}
	}

	pub fn pitch(&mut self, token: Token, offset: f32) {
		if self.mute {
			return;
		}
		if let Some(i) = self.get_index(token) {
			let pitch = self.voices[i].pitch + offset;
			self.instrument.pitch(pitch, i);
		}
	}

	pub fn pressure(&mut self, token: Token, pressure: f32) {
		if self.mute {
			return;
		}
		if let Some(i) = self.get_index(token) {
			self.instrument.pressure(pressure, i);
		}
	}

	pub fn sustain(&mut self, sustain: bool) {
		if self.mute {
			return;
		}
		self.sustain = sustain;
		if !sustain {
			for (i, v) in self.voices.iter_mut().enumerate() {
				if !v.key_down && v.active {
					v.active = false;
					self.instrument.note_off(i);
				}
			}
		}
	}

	pub fn all_notes_off(&mut self) {
		for (i, v) in self.voices.iter_mut().enumerate() {
			if v.active {
				self.instrument.note_off(i);
			}
			*v = Voice::default();
		}
		self.sustain = false;
	}

	pub fn set_mute(&mut self, mute: bool) {
		self.mute = mute;
		self.state = MuteState::Transition;
	}

	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		match self.state {
			MuteState::Off => {},
			MuteState::Active | MuteState::Transition => {
				self.instrument.process(buffer);
			},
		}

		match self.state {
			MuteState::Off => {
				self.meter_handle.set([0., 0.]);
			},
			MuteState::Active => {
				let peak = self.peak.process_block(buffer);
				self.meter_handle.set(peak);
			},
			MuteState::Transition => {
				let target = if self.mute { 0.0 } else { 1.0 };
				let samples = buffer[0].len();
				assert!(samples <= MAX_BUF_SIZE);

				for i in 0..samples {
					self.gain += self.smoothing_f * (target - self.gain);

					buffer[0][i] *= self.gain;
					buffer[1][i] *= self.gain;
				}

				let peak = self.peak.process_block(buffer);
				self.meter_handle.set(peak);

				// Check state transition
				if (self.gain - target).abs() < 1e-4 {
					self.gain = target;
					if self.mute {
						self.flush();
						self.state = MuteState::Off;
					} else {
						self.state = MuteState::Active;
					}
				}
			},
		}
	}

	pub fn flush(&mut self) {
		self.all_notes_off();
		self.instrument.flush();
		self.meter_handle.set([0., 0.]);
	}
}
