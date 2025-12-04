use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd};

use crate::audio::MAX_BUF_SIZE;
use crate::context::{AudioMessage, LuaMessage};
use crate::dsp::env::AttackRelease;
use crate::effect::*;
use crate::instrument;
use crate::instrument::*;
use crate::log::log_warn;
use crate::voice_manager::VoiceManager;

pub struct Channel {
	pub instrument: VoiceManager,
	pub effects: Vec<Bypass>,
	pub mute: bool,
}

impl Channel {
	pub fn new(intrument: Box<dyn Instrument + Send>) -> Self {
		Self { instrument: VoiceManager::new(intrument), effects: Vec::new(), mute: false }
	}
}

pub struct Render {
	audio_rx: HeapCons<AudioMessage>,
	lua_tx: HeapProd<LuaMessage>,
	scope_tx: HeapProd<f32>,
	channels: Vec<Channel>,
	buffer2: [[f32; MAX_BUF_SIZE]; 2],
	pub sample_rate: f32,

	peak_l: AttackRelease,
	peak_r: AttackRelease,
}

impl Render {
	pub fn new(
		sample_rate: f32,
		audio_rx: HeapCons<AudioMessage>,
		lua_tx: HeapProd<LuaMessage>,
		scope_tx: HeapProd<f32>,
	) -> Render {
		Render {
			audio_rx,
			lua_tx,
			scope_tx,
			channels: Vec::new(),
			buffer2: [[0.0f32; MAX_BUF_SIZE]; 2],
			sample_rate,
			peak_l: AttackRelease::new_direct(0.5, 0.1),
			peak_r: AttackRelease::new_direct(0.5, 0.1),
		}
	}

	pub fn send(&mut self, m: LuaMessage) {
		self.lua_tx.try_push(m).ok();
	}

	pub fn insert_channel(&mut self, channel_index: usize, instrument_name: &str) {
		let instrument = instrument::new(self.sample_rate, instrument_name);
		let channel = Channel::new(instrument);
		self.channels.insert(channel_index, channel);
	}

	pub fn remove_channel(&mut self, index: usize) {
		self.channels.remove(index);
	}

	pub fn insert_effect(&mut self, channel_index: usize, effect_index: usize, name: &str) {
		if let Some(ch) = self.channels.get_mut(channel_index) {
			ch.effects.insert(effect_index, Bypass::new(self.sample_rate, name));
		} else {
			log_warn!("Channel index out of bounds");
		}
	}

	pub fn remove_effect(&mut self, channel_index: usize, effect_index: usize) {
		let ch = &mut self.channels[channel_index];
		ch.effects.remove(effect_index);
	}

	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [mut l, mut r] = self.buffer2;
		let len = buffer[0].len();
		let buf_slice = &mut [&mut l[..len], &mut r[..len]];

		// Zero buffer
		for sample in buffer.iter_mut().flat_map(|s| s.iter_mut()) {
			*sample = 0.0;
		}

		// Process all channels
		for ch in &mut self.channels {
			if !ch.mute {
				// Zero buffer
				for sample in buf_slice.iter_mut().flat_map(|s| s.iter_mut()) {
					*sample = 0.0;
				}

				ch.instrument.instrument.process(buf_slice);
				#[cfg(debug_assertions)]
				check_fp(buf_slice);

				for fx in &mut ch.effects {
					fx.process(buf_slice);

					#[cfg(debug_assertions)]
					check_fp(buf_slice);
				}

				for (outsample, insample) in buffer
					.iter_mut()
					.flat_map(|s| s.iter_mut())
					.zip(buf_slice.iter().flat_map(|s| s.iter()))
				{
					*outsample += insample;
				}
			}
		}

		// Calculate peak
		let mut sum = [0.0; 2];
		for (i, track) in buffer.iter().enumerate() {
			sum[i] = track.iter().map(|x| x.abs()).fold(f32::MIN, f32::max);
		}

		self.peak_l.set(sum[0]);
		self.peak_r.set(sum[1]);
		let peak_l = self.peak_l.process();
		let peak_r = self.peak_r.process();
		self.send(LuaMessage::Meter { l: peak_l, r: peak_r });

		// Send everything to scope.
		for s in buffer[0].iter() {
			self.scope_tx.try_push(*s).ok(); // Don't really care if its full
		}

		// hardclip
		for s in buffer.iter_mut().flat_map(|s| s.iter_mut()) {
			*s = s.clamp(-1.0, 1.0);
		}
	}

	pub fn parse_messages(&mut self) {
		use AudioMessage::*;
		while let Some(m) = self.audio_rx.try_pop() {
			match m {
				AllNotesOff => {
					for ch in &mut self.channels {
						ch.instrument.all_notes_off();
					}
				},
				NoteOn(ch_index, token, pitch, vel) => {
					let ch = &mut self.channels[ch_index];
					if !ch.mute {
						ch.instrument.note_on(token, pitch, vel);
					}
				},
				NoteOff(ch_index, token) => {
					let ch = &mut self.channels[ch_index];
					if !ch.mute {
						ch.instrument.note_off(token);
					}
				},
				Pitch(ch_index, token, pitch) => {
					let ch = &mut self.channels[ch_index];
					if !ch.mute {
						ch.instrument.pitch(token, pitch);
					}
				},
				Pressure(ch_index, token, pressure) => {
					let ch = &mut self.channels[ch_index];
					if !ch.mute {
						ch.instrument.pressure(token, pressure);
					}
				},
				Sustain(ch_index, sustain) => {
					let ch = &mut self.channels[ch_index];
					if !ch.mute {
						ch.instrument.sustain(sustain);
					}
				},
				Parameter(ch_index, device_index, index, val) => {
					let ch = &mut self.channels[ch_index];
					if device_index == 0 {
						ch.instrument.instrument.set_parameter(index, val);
					} else {
						ch.effects[device_index - 1].effect.set_parameter(index, val);
					}
				},
				Mute(ch_index, mute) => self.channels[ch_index].mute = mute,
				Bypass(ch_index, device_index, bypass) => {
					let ch = &mut self.channels[ch_index];
					if device_index == 0 {
						log_warn!("Bypass instrument is not supported");
					} else {
						ch.effects[device_index - 1].bypassed = bypass;
					}
				},
				ReorderEffect(ch_index, old_index, new_index) => {
					let ch = &mut self.channels[ch_index];
					let e = ch.effects.remove(old_index);
					ch.effects.insert(new_index, e);
				},
				AudioMessage::Panic => panic!("oof"),
			}
		}
	}

	pub fn flush(&mut self) {
		for ch in &mut self.channels {
			ch.instrument.instrument.flush();
			for fx in &mut ch.effects {
				fx.effect.flush();
			}
		}
	}
}

#[cfg(debug_assertions)]
fn check_fp(buffer: &mut [&mut [f32]; 2]) {
	use std::num::FpCategory::*;
	for s in buffer.iter().flat_map(|s| s.iter()) {
		match s.classify() {
			Normal | Zero => (),
			Nan => panic!("number was NaN"),
			Infinite => panic!("number was Inf"),
			Subnormal => unreachable!(),
		}
	}
}
