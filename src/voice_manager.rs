use crate::audio::MAX_BUF_SIZE;
use crate::dsp::{MuteState, PeakMeter, time_constant};
use crate::instrument::Instrument;
use crate::meters::MeterHandle;
use std::collections::VecDeque;

pub type Token = u32;

// Silent voices kept free for new notes, for instruments that support Instrument::kill. Other voices
// are killed (faded out) early to keep them, so a new note never has to replace a sounding voice.
const RESERVE: usize = 2;

#[derive(Clone, Copy, Debug)]
struct Voice {
	token: Token,
	pitch: f32,
	vel: f32,
	pressure: f32,
	key_down: bool,
	active: bool,
	// fading out after Instrument::kill
	killed: bool,
	age: u64,
}

impl Default for Voice {
	fn default() -> Self {
		Self {
			token: 0,
			pitch: 0.0,
			vel: 0.0,
			pressure: 0.0,
			key_down: false,
			active: false,
			killed: false,
			age: 0,
		}
	}
}

impl Voice {
	fn new(token: Token, pitch: f32, vel: f32, pressure: f32) -> Self {
		Self { token, pitch, vel, pressure, key_down: true, active: true, ..Default::default() }
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

	pub fn note_on(&mut self, token: Token, pitch: f32, offset: f32, vel: f32, pressure: f32) {
		if self.mute {
			return;
		}
		for v in &mut self.voices {
			v.age += 1;
		}

		let voice = Voice::new(token, pitch, vel, pressure);
		if let Some(i) = self.find_retrigger(pitch) {
			self.start(i, voice, pitch + offset);
		} else if let Some(i) = self.find_free() {
			self.start(i, voice, pitch + offset);
			self.keep_reserve(pitch, i);
		} else {
			// no silent voice: replace one in place, the oldest if all are fading out already
			let i = self
				.find_victim(pitch, None)
				.or_else(|| (0..self.voices.len()).max_by_key(|&i| self.voices[i].age))
				.expect("Should find a voice");
			self.displace(i);
			self.start(i, voice, pitch + offset);
		}
	}

	// Released voice at `pitch` that still sounds (sustained or ringing out).
	fn find_retrigger(&self, pitch: f32) -> Option<usize> {
		(0..self.voices.len()).find(|&i| {
			let v = &self.voices[i];
			!v.killed
				&& (v.pitch - pitch).abs() < 1e-3
				&& (v.active || self.instrument.voice_active(i))
		})
	}

	// Silent voice that isn't held, oldest first.
	fn find_free(&self) -> Option<usize> {
		(0..self.voices.len())
			.filter(|&i| !self.voices[i].key_down && !self.instrument.voice_active(i))
			.max_by_key(|&i| self.voices[i].age)
	}

	// Voice to give up for a new note at `pitch`: the oldest released one that still sounds, else the
	// held one closest in pitch. Skips `exclude` and voices already fading out.
	fn find_victim(&self, pitch: f32, exclude: Option<usize>) -> Option<usize> {
		let candidates = || {
			(0..self.voices.len()).filter(move |&i| {
				let v = &self.voices[i];
				Some(i) != exclude && !v.killed && (v.key_down || self.instrument.voice_active(i))
			})
		};
		let released = candidates()
			.filter(|&i| !self.voices[i].key_down)
			.max_by_key(|&i| self.voices[i].age);
		released.or_else(|| {
			candidates().filter(|&i| self.voices[i].key_down).min_by(|&a, &b| {
				let da = (self.voices[a].pitch - pitch).abs();
				let db = (self.voices[b].pitch - pitch).abs();
				da.total_cmp(&db)
			})
		})
	}

	// A held voice that gets replaced goes to the queue, to come back when a key is released.
	fn displace(&mut self, i: usize) {
		if self.voices[i].key_down {
			// Drop the oldest stolen voice if capacity is full
			if self.queue.len() == self.queue.capacity() {
				self.queue.pop_front();
			}
			self.queue.push_back(self.voices[i]);
		}
	}

	fn start(&mut self, i: usize, voice: Voice, pitch: f32) {
		self.voices[i] = voice;
		self.instrument.note_on(pitch, voice.vel, i);
		self.instrument.pressure(voice.pressure, i);
	}

	// Kill voices until RESERVE silent ones are free.
	// They become free once faded out.
	fn keep_reserve(&mut self, pitch: f32, new: usize) {
		let n = self.voices.len();
		let free = (0..n)
			.filter(|&i| !self.voices[i].key_down && !self.instrument.voice_active(i))
			.count();
		let mut fading = (0..n)
			.filter(|&i| self.voices[i].killed && self.instrument.voice_active(i))
			.count();
		while free + fading < RESERVE {
			let Some(i) = self.find_victim(pitch, Some(new)) else {
				return;
			};
			if !self.instrument.kill(i) {
				// not supported
				return;
			}
			self.displace(i);
			self.voices[i] = Voice { killed: true, ..Default::default() };
			fading += 1;
		}
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
			// Revive latest stolen voice from the queue
			let recovered = self.queue.pop_back().unwrap();
			match self.find_free() {
				Some(j) if self.instrument.kill(i) => {
					self.voices[i] = Voice { killed: true, ..Default::default() };
					self.start(j, recovered, recovered.pitch);
					self.keep_reserve(recovered.pitch, j);
				},
				_ => self.start(i, recovered, recovered.pitch),
			}
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
			self.voices[i].pressure = pressure;
			self.instrument.pressure(pressure, i);
		} else if let Some(q) = self.get_queue_index(token) {
			self.queue[q].pressure = pressure;
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
		self.queue.clear();
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::meters::Meters;
	use crate::worker::RequestData;
	use std::sync::{Arc, Mutex};

	type Log = Arc<Mutex<Vec<(&'static str, usize, f32)>>>;

	// Records the calls it gets, never sounds.
	struct Mock {
		log: Log,
		voices: usize,
	}

	impl Instrument for Mock {
		fn new(_sample_rate: f32) -> Self {
			unimplemented!()
		}
		fn process(&mut self, _buffer: &mut [&mut [f32]; 2]) {}
		fn note_on(&mut self, pitch: f32, _vel: f32, id: usize) {
			self.log.lock().unwrap().push(("note_on", id, pitch));
		}
		fn note_off(&mut self, id: usize) {
			self.log.lock().unwrap().push(("note_off", id, 0.0));
		}
		fn pitch(&mut self, _pitch: f32, _id: usize) {}
		fn pressure(&mut self, pressure: f32, id: usize) {
			self.log.lock().unwrap().push(("pressure", id, pressure));
		}
		fn set_parameter(&mut self, _index: usize, _value: f32) -> Option<RequestData> {
			None
		}
		fn flush(&mut self) {}
		fn voice_count(&self) -> usize {
			self.voices
		}
	}

	fn manager(voices: usize) -> (VoiceManager, Log) {
		let log = Log::default();
		let (meter, _) = Meters::new().register();
		let vm = VoiceManager::new(44100.0, Box::new(Mock { log: log.clone(), voices }), meter);
		(vm, log)
	}

	#[test]
	fn note_on_comes_with_its_pressure() {
		let (mut vm, log) = manager(4);
		vm.note_on(1, 60.0, 0.0, 0.5, 0.3);
		let log = log.lock().unwrap();
		let i = log[0].1;
		assert_eq!(*log, [("note_on", i, 60.0), ("pressure", i, 0.3)]);
	}

	#[test]
	fn revived_voice_keeps_its_pressure() {
		let (mut vm, log) = manager(1);
		vm.note_on(1, 60.0, 0.0, 0.5, 0.2);
		// steals the only voice, note 1 waits in the queue
		vm.note_on(2, 62.0, 0.0, 0.5, 0.4);
		vm.pressure(1, 0.9);
		log.lock().unwrap().clear();
		vm.note_off(2);
		assert_eq!(*log.lock().unwrap(), [("note_on", 0, 60.0), ("pressure", 0, 0.9)]);
	}
}
