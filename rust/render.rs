use ringbuf::{HeapConsumer, HeapProducer};

use crate::audio::MAX_BUF_SIZE;
use crate::device::*;
use crate::dsp::env::AttackRelease;
use crate::dsp::softclip;
use crate::effect::*;
use crate::instrument::*;
use crate::lua::{AudioMessage, LuaMessage};

pub struct Channel {
	pub instrument: Box<dyn Instrument + Send>,
	pub effects: Vec<BypassEffect>,
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
		// if self.lua_tx.push(m).is_err() {
		// 	println!("Lua queue full. Dropped message!");
		// }
		self.lua_tx.push(m).ok();
	}

	pub fn add_channel(&mut self, instrument_index: usize) {
		let new_instr = new_instrument(self.sample_rate, instrument_index);
		let newch = Channel {
			instrument: new_instr,
			effects: Vec::new(),
			mute: false,
		};
		self.channels.push(newch);
	}

	// TODO: temp fix right inserts the fx in second to last place so the pan is always last
	//       also update this in channel_handler
	pub fn add_effect(&mut self, ch_index: usize, effect_number: usize) {
		match self.channels.get_mut(ch_index) {
			Some(ch) => {
				let l = ch.effects.len().saturating_sub(1);
				ch.effects
					.insert(l, BypassEffect::new(self.sample_rate, effect_number));
			}
			None => println!("Channel index out of bounds!"),
		}
	}

	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) -> Result<(), &'static str> {
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
				check_fp(buf_slice)?;

				for fx in &mut ch.effects {
					fx.process(buf_slice);

					#[cfg(debug_assertions)]
					check_fp(buf_slice)?;
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

		// Send everything to scope before clipping.
		// For now, we only send the left channel.
		for s in buffer[0].iter() {
			self.scope_tx.push(*s).ok(); // Don't really care if its full
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

		// Add 6dB headroom and some tanh-like softclip
		for s in buffer.iter_mut().flat_map(|s| s.iter_mut()) {
			*s = softclip(*s * 0.50);
			// *s *= 0.50;
		}

		// clipping isn't strictly necessary but we'll do it anyway
		for s in buffer.iter_mut().flat_map(|s| s.iter_mut()) {
			*s = s.clamp(-1.0, 1.0);
		}
		Ok(())
	}

	pub fn parse_messages(&mut self) {
		while let Some(m) = self.audio_rx.pop() {
			match m {
				AudioMessage::CV(ch_index, pitch, pres) => match self.channels.get_mut(ch_index) {
					Some(ch) => {
						if !ch.mute {
							ch.instrument.cv(pitch, pres);
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
				AudioMessage::Parameter(ch_index, device_index, index, val) => {
					match self.channels.get_mut(ch_index) {
						Some(ch) => {
							if device_index == 0 {
								ch.instrument.set_parameter(index, val);
							} else {
								match ch.effects.get_mut(device_index - 1) {
									Some(e) => e.effect.set_parameter(index, val),
									None => println!("Device index out of bounds!"),
								}
							}
						}
						None => println!("Channel index out of bounds!"),
					}
				}

				AudioMessage::Mute(ch_index, mute) => match self.channels.get_mut(ch_index) {
					Some(ch) => ch.mute = mute,
					None => println!("Channel index out of bounds!"),
				},
			}
		}
	}
}

#[cfg(debug_assertions)]
fn check_fp(buffer: &mut [&mut [f32]; 2]) -> Result<(), &'static str> {
	use std::num::FpCategory::*;
	buffer
		.iter()
		.flat_map(|s| s.iter())
		.try_for_each(|s| match s.classify() {
			Normal | Zero => Ok(()),
			Nan => Err("number was NaN"),
			Infinite => Err("number was Inf"),
			Subnormal => unreachable!(),
		})
}
