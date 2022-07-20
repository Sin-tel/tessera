use ringbuf::{Consumer, Producer};

use crate::defs::*;
use crate::dsp::*;
use crate::instrument::*;
use crate::pan::*;
use crate::param::*;

pub struct BypassEffect {
	pub effect: Box<dyn Effect + Send>,
	pub bypassed: bool,
}

pub struct Channel {
	pub instrument: Box<dyn Instrument + Send>,
	pub pan: Pan,
	pub effects: Vec<BypassEffect>,
	pub mute: bool,
}

pub struct Render {
	audio_rx: Consumer<AudioMessage>,
	lua_tx: Producer<LuaMessage>,
	channels: Vec<Channel>,
	buffer2: [Stereo; MAX_BUF_SIZE],
	pub sample_rate: f32,

	peakl: SmoothedEnv,
	peakr: SmoothedEnv,
}

impl Render {
	pub fn new(
		sample_rate: f32,
		audio_rx: Consumer<AudioMessage>,
		lua_tx: Producer<LuaMessage>,
	) -> Render {
		Render {
			audio_rx,
			lua_tx,
			channels: Vec::new(),
			buffer2: [Stereo::default(); MAX_BUF_SIZE],
			sample_rate,
			peakl: SmoothedEnv::new(0.0, 0.5, 0.1, 1.0),
			peakr: SmoothedEnv::new(0.0, 0.5, 0.1, 1.0),
		}
	}

	pub fn send(&mut self, m: LuaMessage) {
		if self.lua_tx.push(m).is_err() {
			println!("Lua queue full. Dropped message!");
		}
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

	pub fn process(&mut self, buffer: &mut [Stereo]) {
		let buf_slice = &mut self.buffer2[..buffer.len()];

		// zero buffer
		for sample in buffer.iter_mut() {
			*sample = [0.0; 2]
		}

		// process all channels
		for ch in &mut self.channels {
			if !ch.mute {
				ch.instrument.process(buf_slice);
				ch.pan.process(buf_slice);
				for (outsample, insample) in buffer.iter_mut().zip(buf_slice.iter()) {
					outsample[0] += insample[0];
					outsample[1] += insample[1];
				}
			}
		}

		// default 6dB headroom + tanh
		for s in buffer.iter_mut().flat_map(|s| s.iter_mut()) {
			*s = softclip(*s * 0.50);
		}

		let mut suml: f32 = 0.0;
		let mut sumr: f32 = 0.0;

		// peak meter
		for sample in buffer.iter() {
			suml = suml.max(sample[0].abs());
			sumr = sumr.max(sample[1].abs());
		}

		self.peakl.set(suml);
		self.peakr.set(sumr);

		self.peakl.update();
		self.peakr.update();

		self.send(LuaMessage::Meter(self.peakl.value, self.peakr.value));

		for s in buffer.iter_mut().flat_map(|s| s.iter_mut()) {
			*s = s.clamp(-1.0, 1.0);
		}
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
