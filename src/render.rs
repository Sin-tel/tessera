use crate::audio::MAX_BUF_SIZE;
use crate::channel::Channel;
use crate::context::{AudioMessage, LuaMessage};
use crate::dsp;
use crate::dsp::env::AttackRelease;
use crate::effect::*;
use crate::instrument;
use crate::meters::MeterHandle;
use crate::voice_manager::VoiceManager;
use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd};
use std::sync::mpsc::SyncSender;

pub struct Render {
	audio_rx: HeapCons<AudioMessage>,
	lua_tx: SyncSender<LuaMessage>,
	scope_tx: HeapProd<f32>,
	channels: Vec<Channel>,
	buffer: [[f32; MAX_BUF_SIZE]; 2],
	pub sample_rate: f32,

	peak_l: AttackRelease,
	peak_r: AttackRelease,
}

impl Render {
	pub fn new(
		sample_rate: f32,
		audio_rx: HeapCons<AudioMessage>,
		lua_tx: SyncSender<LuaMessage>,
		scope_tx: HeapProd<f32>,
	) -> Render {
		Render {
			audio_rx,
			lua_tx,
			scope_tx,
			channels: Vec::new(),
			buffer: [[0.0f32; MAX_BUF_SIZE]; 2],
			sample_rate,
			peak_l: AttackRelease::new_direct(0.5, 0.05),
			peak_r: AttackRelease::new_direct(0.5, 0.05),
		}
	}

	pub fn send(&mut self, m: LuaMessage) {
		let _ = self.lua_tx.try_send(m).is_err();
	}

	pub fn insert_channel(
		&mut self,
		channel_index: usize,
		instrument_name: &str,
		meter_handle_channel: MeterHandle,
		meter_handle_instrument: MeterHandle,
	) {
		let instrument = instrument::new(self.sample_rate, instrument_name);
		let voice_manager =
			VoiceManager::new(self.sample_rate, instrument, meter_handle_instrument);
		let channel = Channel::new(self.sample_rate, voice_manager, meter_handle_channel);
		self.channels.insert(channel_index, channel);
	}

	pub fn remove_channel(&mut self, index: usize) {
		self.channels.remove(index);
	}

	pub fn insert_effect(
		&mut self,
		channel_index: usize,
		effect_index: usize,
		name: &str,
		meter_handle: MeterHandle,
	) {
		let ch = &mut self.channels[channel_index];
		ch.effects
			.insert(effect_index, Bypass::new(self.sample_rate, name, meter_handle));
	}

	pub fn remove_effect(&mut self, channel_index: usize, effect_index: usize) {
		let ch = &mut self.channels[channel_index];
		ch.effects.remove(effect_index);
	}

	pub fn process(&mut self, buffer_out: &mut [&mut [f32]; 2]) {
		let len = buffer_out[0].len();

		let (l, r) = self.buffer.split_at_mut(1);
		let buffer_in = &mut [&mut l[0][..len], &mut r[0][..len]];

		// Zero buffer
		for sample in buffer_out.iter_mut().flat_map(|s| s.iter_mut()) {
			*sample = 0.0;
		}

		// Process all channels
		for ch in &mut self.channels {
			ch.process(buffer_in, buffer_out);
		}

		// Calculate master peak
		let [peak_l, peak_r] = dsp::peak(buffer_out);
		self.peak_l.set(peak_l);
		self.peak_r.set(peak_r);

		let peak_l = self.peak_l.process();
		let peak_r = self.peak_r.process();
		self.send(LuaMessage::Meter { l: peak_l, r: peak_r });

		// Send everything to scope.
		for s in buffer_out[0].iter() {
			// Don't really care if it's full
			let _ = self.scope_tx.try_push(*s);
		}

		// hardclip
		for s in buffer_out.iter_mut().flat_map(|s| s.iter_mut()) {
			*s = s.clamp(-1.0, 1.0);
		}
	}

	pub fn parse_messages(&mut self) {
		use AudioMessage::*;
		while let Some(m) = self.audio_rx.try_pop() {
			match m {
				AllNotesOff => {
					for ch in &mut self.channels {
						if let Some(instrument) = &mut ch.instrument {
							instrument.all_notes_off();
						}
					}
				},
				NoteOn(ch_index, token, pitch, offset, vel) => {
					let ch = &mut self.channels[ch_index];
					if let Some(instrument) = &mut ch.instrument {
						instrument.note_on(token, pitch, offset, vel);
					}
				},
				NoteOff(ch_index, token) => {
					let ch = &mut self.channels[ch_index];
					if let Some(instrument) = &mut ch.instrument {
						instrument.note_off(token);
					}
				},
				Pitch(ch_index, token, pitch) => {
					let ch = &mut self.channels[ch_index];
					if let Some(instrument) = &mut ch.instrument {
						instrument.pitch(token, pitch);
					}
				},
				Pressure(ch_index, token, pressure) => {
					let ch = &mut self.channels[ch_index];
					if let Some(instrument) = &mut ch.instrument {
						instrument.pressure(token, pressure);
					}
				},
				Sustain(ch_index, sustain) => {
					let ch = &mut self.channels[ch_index];
					if let Some(instrument) = &mut ch.instrument {
						instrument.sustain(sustain);
					}
				},
				Parameter(ch_index, device_index, index, val) => {
					let ch = &mut self.channels[ch_index];
					if device_index == 0
						&& let Some(instrument) = &mut ch.instrument
					{
						instrument.set_parameter(index, val);
					} else {
						ch.effects[device_index - 1].effect.set_parameter(index, val);
					}
				},
				ChannelMute(ch_index, mute) => self.channels[ch_index].set_mute(mute),
				ChannelGain(ch_index, gain) => self.channels[ch_index].set_gain(gain),
				DeviceMute(ch_index, device_index, mute) => {
					let ch = &mut self.channels[ch_index];
					if device_index == 0
						&& let Some(instrument) = &mut ch.instrument
					{
						instrument.set_mute(mute);
					} else {
						ch.effects[device_index - 1].set_mute(mute);
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
			if let Some(instrument) = &mut ch.instrument {
				instrument.flush();
			}
			for fx in &mut ch.effects {
				fx.effect.flush();
			}
		}
	}
}
