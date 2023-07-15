use ringbuf::{HeapConsumer, HeapProducer};

use crate::audio::MAX_BUF_SIZE;
use crate::dsp::env::AttackRelease;
use crate::effect::*;
use crate::instrument;
use crate::instrument::*;
use crate::lua::{AudioMessage, LuaMessage};

pub struct Channel {
	pub instrument: Box<dyn Instrument + Send>,
	pub effects: Vec<Bypass>,
	pub mute: bool,
}

pub struct Render {
	audio_rx: HeapConsumer<AudioMessage>,
	lua_tx: HeapProducer<LuaMessage>,
	scope_tx: HeapProducer<f32>,
	channels: Vec<Channel>,
	buffer2: [[f32; MAX_BUF_SIZE]; 2],
	pub sample_rate: f32,

	peak_l: AttackRelease,
	peak_r: AttackRelease,
}

impl Render {
	pub fn new(
		sample_rate: f32,
		audio_rx: HeapConsumer<AudioMessage>,
		lua_tx: HeapProducer<LuaMessage>,
		scope_tx: HeapProducer<f32>,
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
		self.lua_tx.push(m).ok();
	}

	pub fn add_channel(&mut self, instrument_index: usize) {
		let new_instr = instrument::new(self.sample_rate, instrument_index);
		let newch = Channel {
			instrument: new_instr,
			effects: Vec::new(),
			mute: false,
		};
		self.channels.push(newch);
	}

	pub fn remove_channel(&mut self, index: usize) {
		self.channels.remove(index);
	}

	pub fn add_effect(&mut self, channel_index: usize, effect_number: usize) {
		match self.channels.get_mut(channel_index) {
			Some(ch) => {
				ch.effects
					.insert(0, Bypass::new(self.sample_rate, effect_number));
			}
			None => println!("Channel index out of bounds"),
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
				ch.instrument.process(buf_slice);
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
			sum[i] = track.iter().map(|x| x.abs()).fold(std::f32::MIN, f32::max);
		}

		self.peak_l.set(sum[0]);
		self.peak_r.set(sum[1]);
		let peak_l = self.peak_l.process();
		let peak_r = self.peak_r.process();
		self.send(LuaMessage::Meter(peak_l, peak_r));

		// Send everything to scope.
		for s in buffer[0].iter() {
			self.scope_tx.push(*s).ok(); // Don't really care if its full
		}

		// hardclip
		for s in buffer.iter_mut().flat_map(|s| s.iter_mut()) {
			*s = s.clamp(-1.0, 1.0);
		}
	}

	pub fn parse_messages(&mut self) {
		use AudioMessage::*;
		while let Some(m) = self.audio_rx.pop() {
			match m {
				CV(ch_index, pitch, pres, id) => {
					let ch = &mut self.channels[ch_index];
					if !ch.mute {
						ch.instrument.cv(pitch, pres, id);
					}
				}
				Note(ch_index, pitch, vel, id) => {
					let ch = &mut self.channels[ch_index];
					if !ch.mute {
						ch.instrument.note(pitch, vel, id);
					}
				}
				Parameter(ch_index, device_index, index, val) => {
					let ch = &mut self.channels[ch_index];
					if device_index == 0 {
						ch.instrument.set_parameter(index, val);
					} else {
						ch.effects[device_index - 1]
							.effect
							.set_parameter(index, val);
					}
				}
				Mute(ch_index, mute) => self.channels[ch_index].mute = mute,
				Bypass(ch_index, device_index, bypass) => {
					let ch = &mut self.channels[ch_index];
					if device_index == 0 {
						eprintln!("Bypass instrument is not supported");
					} else {
						ch.effects[device_index - 1].bypassed = bypass;
					}
				}
				ReorderEffect(ch_index, old_index, new_index) => {
					let ch = &mut self.channels[ch_index];
					let e = ch.effects.remove(old_index);
					ch.effects.insert(new_index, e);
				}
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
