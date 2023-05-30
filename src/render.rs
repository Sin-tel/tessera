use no_denormals::no_denormals;
use ringbuf::{Consumer, Producer};

use crate::defs::*;
use crate::device::*;
use crate::dsp::env::SmoothedEnv;
use crate::dsp::softclip;
use crate::effect::*;
use crate::instrument::*;
use crate::pan::*;

pub struct Channel {
	pub instrument: Box<dyn Instrument + Send>,
	pub pan: Pan,
	pub effects: Vec<BypassEffect>,
	pub mute: bool,
}

pub struct Render {
	audio_rx: Consumer<AudioMessage>,
	lua_tx: Producer<LuaMessage>,
	scope_tx: Producer<f32>,
	channels: Vec<Channel>,
	buffer2: [[f32; MAX_BUF_SIZE]; 2],
	pub sample_rate: f32,

	peakl: SmoothedEnv,
	peakr: SmoothedEnv,
}

impl Render {
	pub fn new(
		sample_rate: f32,
		audio_rx: Consumer<AudioMessage>,
		lua_tx: Producer<LuaMessage>,
		scope_tx: Producer<f32>,
	) -> Render {
		Render {
			audio_rx,
			lua_tx,
			scope_tx,
			channels: Vec::new(),
			buffer2: [[0.0f32; MAX_BUF_SIZE]; 2],
			sample_rate,
			peakl: SmoothedEnv::new_direct(0.5, 0.1),
			peakr: SmoothedEnv::new_direct(0.5, 0.1),
		}
	}

	pub fn send(&mut self, m: LuaMessage) {
		// if self.lua_tx.push(m).is_err() {
		// 	println!("Lua queue full. Dropped message!");
		// }
		self.lua_tx.push(m).ok();
	}

	pub fn add_channel(&mut self, instrument_index: usize) {
		let new_instr = new_instrument(self.sample_rate, instrument_index);
		let newch = Channel {
			instrument: new_instr,
			pan: Pan::new(self.sample_rate),
			effects: Vec::new(),
			mute: false,
		};
		self.channels.push(newch);
	}

	pub fn add_effect(&mut self, ch_index: usize, effect_number: usize) {
		match self.channels.get_mut(ch_index) {
			Some(ch) => ch
				.effects
				.push(BypassEffect::new(self.sample_rate, effect_number)),
			None => println!("Channel index out of bounds!"),
		}
	}

	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		no_denormals(|| {
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

					for fx in ch.effects.iter_mut() {
						fx.process(buf_slice);
					}

					ch.pan.process(buf_slice);

					for (outsample, insample) in buffer
						.iter_mut()
						.flat_map(|s| s.iter_mut())
						.zip(buf_slice.iter().flat_map(|s| s.iter()))
					{
						*outsample += insample;
					}
				}
			}

			// Send everything to scope before clipping.
			// For now, we only send the left channel.
			for s in buffer[0].iter() {
				self.scope_tx.push(*s).ok(); // Don't really care if its full
			}

			// Calculate peak
			let mut sum = [0.0; 2];
			for (i, track) in buffer.iter().enumerate() {
				sum[i] = track
					.iter()
					.map(|x| x.abs())
					.fold(std::f32::MIN, |a, b| a.max(b));
			}

			self.peakl.set(sum[0]);
			self.peakr.set(sum[1]);
			self.peakl.update();
			self.peakr.update();
			self.send(LuaMessage::Meter(self.peakl.get(), self.peakr.get()));

			// Add 6dB headroom and some tanh-like softclip
			for s in buffer.iter_mut().flat_map(|s| s.iter_mut()) {
				*s = softclip(*s * 0.50);
				// *s *= 0.50;
			}

			// clipping isn't strictly necessary but we'll do it anyway
			for s in buffer.iter_mut().flat_map(|s| s.iter_mut()) {
				*s = s.clamp(-1.0, 1.0);
			}
		})
	}

	pub fn parse_messages(&mut self) {
		while let Some(m) = self.audio_rx.pop() {
			match m {
				AudioMessage::CV(ch_index, pitch, vel) => match self.channels.get_mut(ch_index) {
					Some(ch) => {
						if !ch.mute {
							ch.instrument.cv(pitch, vel);
						}
					}
					None => println!("Channel index out of bounds!"),
				},
				AudioMessage::Note(ch_index, pitch, vel, id) => {
					match self.channels.get_mut(ch_index) {
						Some(ch) => {
							if !ch.mute {
								ch.instrument.note(pitch, vel, id);
							}
						}
						None => println!("Channel index out of bounds!"),
					}
				}
				AudioMessage::SetParam(ch_index, device_index, index, val) => {
					match self.channels.get_mut(ch_index) {
						Some(ch) => {
							if device_index == 0 {
								ch.instrument.set_param(index, val);
							} else {
								match ch.effects.get_mut(device_index - 1) {
									Some(e) => e.effect.set_param(index, val),
									None => println!("Device index out of bounds!"),
								}
							}
						}
						None => println!("Channel index out of bounds!"),
					}
				}
				AudioMessage::Pan(ch_index, gain, pan) => match self.channels.get_mut(ch_index) {
					Some(ch) => ch.pan.set(gain, pan),
					None => println!("Channel index out of bounds!"),
				},
				AudioMessage::Mute(ch_index, mute) => match self.channels.get_mut(ch_index) {
					Some(ch) => ch.mute = mute,
					None => println!("Channel index out of bounds!"),
				},
			}
		}
	}
}
