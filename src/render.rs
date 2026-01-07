use crate::audio::MAX_BUF_SIZE;
use crate::channel::Channel;
use crate::context::{AudioMessage, LuaMessage};
use crate::effect::*;
use crate::instrument;
use crate::log::*;
use crate::meters::MeterHandle;
use crate::metronome::Metronome;
use crate::voice_manager::VoiceManager;
use crate::worker::{Request, Response};
use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd};
use std::sync::mpsc::{Receiver, SyncSender};

pub struct Render {
	audio_rx: HeapCons<AudioMessage>,
	lua_tx: SyncSender<LuaMessage>,
	worker_tx: SyncSender<Request>,
	worker_rx: Receiver<Response>,
	scope_tx: HeapProd<f32>,
	channels: Vec<Channel>,
	buffer: [[f32; MAX_BUF_SIZE]; 2],
	pub sample_rate: f32,

	metronome: Metronome,
}

impl Render {
	pub fn new(
		sample_rate: f32,
		audio_rx: HeapCons<AudioMessage>,
		lua_tx: SyncSender<LuaMessage>,
		worker_tx: SyncSender<Request>,
		worker_rx: Receiver<Response>,
		scope_tx: HeapProd<f32>,
	) -> Render {
		Render {
			audio_rx,
			lua_tx,
			worker_tx,
			worker_rx,
			scope_tx,
			channels: Vec::new(),
			buffer: [[0.0f32; MAX_BUF_SIZE]; 2],
			sample_rate,
			metronome: Metronome::new(sample_rate),
		}
	}

	pub fn send(&mut self, m: LuaMessage) {
		let _ = self.lua_tx.try_send(m).is_err();
	}

	pub fn insert_channel(&mut self, channel_index: usize, meter_handle_channel: MeterHandle) {
		if channel_index > self.channels.len() {
			log_error!("Insert channel index {} out of bounds.", channel_index);
			return;
		}
		let channel = Channel::new(self.sample_rate, None, meter_handle_channel);
		self.channels.insert(channel_index, channel);
	}

	pub fn insert_instrument(
		&mut self,
		channel_index: usize,
		instrument_name: &str,
		meter_handle_instrument: MeterHandle,
	) {
		assert!(channel_index > 0, "Trying to insert instrument on master channel");
		let instrument = instrument::new(self.sample_rate, instrument_name);
		let voice_manager =
			VoiceManager::new(self.sample_rate, instrument, meter_handle_instrument);

		let ch = &mut self.channels[channel_index];
		ch.instrument = Some(voice_manager);
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

	pub fn process<'a>(&'a mut self, buffer_out: &mut [&'a mut [f32]; 2]) {
		if self.channels.is_empty() {
			return;
		}

		let len = buffer_out[0].len();

		let (l, r) = self.buffer.split_at_mut(1);
		let buffer_in = &mut [&mut l[0][..len], &mut r[0][..len]];

		// Zero buffer
		for sample in buffer_out.iter_mut().flat_map(|s| s.iter_mut()) {
			*sample = 0.0;
		}

		// Process all channels
		for ch in &mut self.channels[1..] {
			ch.process(buffer_in, buffer_out);
		}

		// swap buffers for master
		std::mem::swap(buffer_in, buffer_out);
		for buf in buffer_out.iter_mut() {
			buf.fill(0.);
		}
		self.channels[0].process(buffer_in, buffer_out);

		// Send everything to scope.
		for s in buffer_out[0].iter() {
			// Don't really care if it's full
			let _ = self.scope_tx.try_push(*s);
		}

		self.metronome.process(buffer_out);

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
				Parameter(channel_index, device_index, index, val) => {
					let ch = &mut self.channels[channel_index];

					let request_data = if device_index == 0
						&& let Some(instrument) = &mut ch.instrument
					{
						instrument.instrument.set_parameter(index, val)
					} else {
						ch.effects[device_index - 1].effect.set_parameter(index, val)
					};

					if let Some(data) = request_data {
						let request = Request::LoadRequest { channel_index, device_index, data };
						// Handle request
						if let Err(e) = self.worker_tx.try_send(request) {
							log_error!("{e}");
						}
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
				Metronome(accent) => {
					self.metronome.trigger(accent);
				},
				AudioMessage::Panic => panic!("oof"),
			}
		}

		while let Ok(response) = self.worker_rx.try_recv() {
			let device_index = response.device_index;
			let channel = &mut self.channels[response.channel_index];

			let garbage = if device_index == 0
				&& let Some(instrument) = &mut channel.instrument
			{
				instrument.instrument.receive_data(response.data)
			} else {
				channel.effects[device_index - 1].effect.receive_data(response.data)
			};

			if let Some(garbage) = garbage {
				let request = Request::Garbage(garbage);
				if let Err(e) = self.worker_tx.try_send(request) {
					log_error!("{e}");
				}
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
