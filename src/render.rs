use ringbuf::{Consumer, Producer};

use crate::defs::*;
use crate::instrument::*;
use crate::param::*;
use crate::pan::*;
use crate::dsp::*;

pub struct Channel {
	pub instrument: Box<dyn Instrument + Send>,
	pub pan: Pan,
}

pub struct Render {
	audio_rx: Consumer<AudioMessage>,
	lua_tx: Producer<LuaMessage>,
	channels: Vec<Channel>,
	effects: Vec<Vec<Box<dyn Effect + Send>>>,
	buffer2: [StereoSample; MAX_BUF_SIZE],
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
			effects: Vec::new(),
			// buffer2: vec![Default::default(); MAX_BUF_SIZE],
			buffer2: [Default::default(); MAX_BUF_SIZE],
			sample_rate,
			peakl: SmoothedEnv::new(0.0, 0.5, 0.1),
			peakr: SmoothedEnv::new(0.0, 0.5, 0.1),
		}
	}

	pub fn send(&mut self, m: LuaMessage) {
		if !self.lua_tx.is_full() {
			self.lua_tx.push(m).unwrap();
		} else {
			println!("Lua queue full. Dropped message!")
		}
	}

	pub fn add_channel(&mut self, instrument_index: usize) {

		let new_instr = new_instrument(self.sample_rate, instrument_index);
		let newch = Channel {
			instrument: new_instr,
			pan: Pan::new(self.sample_rate),
		};
		self.channels.push(newch);
		self.effects.push(Vec::new());
	}

	pub fn process(&mut self, buffer: &mut [StereoSample]) {
		let buf_slice = &mut self.buffer2[..buffer.len()];

		// zero buffer
		for sample in buffer.iter_mut() {
			sample.l = 0.0;
			sample.r = 0.0;
		}

		// process all channels
		for ch in self.channels.iter_mut() {
			ch.instrument.process(buf_slice);
			ch.pan.process(buf_slice);
			for (outsample, insample) in buffer.iter_mut().zip(buf_slice.iter()) {
				outsample.l += insample.l;
				outsample.r += insample.r;
			}
		}

		// default 12dB headroom + tanh
		// for sample in buffer.iter_mut() {
		// 	sample.l = (sample.l * 0.25).tanh();
		// 	sample.r = (sample.r * 0.25).tanh();
		// }


		let mut suml: f32 = 0.0;
		let mut sumr: f32 = 0.0;
		// peak meter
		for sample in buffer.iter_mut() {
			suml = suml.max(sample.l.abs());
			sumr = sumr.max(sample.r.abs());
		}

		self.peakl.set(suml);
		self.peakr.set(sumr);

		self.peakl.update();
		self.peakr.update();

		self.send(LuaMessage::Meter(self.peakl.value, self.peakr.value));

		 for sample in buffer.iter_mut() {
			sample.l = sample.l.clamp(-1.0,1.0);
			sample.r = sample.r.clamp(-1.0,1.0);
		}
	}

	pub fn parse_messages(&mut self) {
		while let Some(m) = self.audio_rx.pop() {
			match m {
				// todo send to correct channel
				AudioMessage::CV(ch_index, cv) => match self.channels.get_mut(ch_index) {
					Some(ch) => ch.instrument.cv(cv.pitch, cv.vel),
					None => println!("Channel index out of bounds!"),
				},
				AudioMessage::Note(ch_index, cv) => match self.channels.get_mut(ch_index) {
					Some(ch) => ch.instrument.note(cv.pitch, cv.vel),
					None => println!("Channel index out of bounds!"),
				},
				AudioMessage::SetParam(ch_index, device_index, index, val) => {
					if device_index == 0 {
						match self.channels.get_mut(ch_index) {
							Some(ch) => ch.instrument.set_param(index, val),
							None => println!("Channel index out of bounds!"),
						}
					} else {
						match self.effects.get_mut(ch_index) {
							Some(ch) => match ch.get_mut(device_index - 1) {
								Some(e) => e.set_param(index, val),
								None => println!("Device index out of bounds!"),
							},
							None => println!("Channel index out of bounds!"),
						}
					}
				}
				AudioMessage::Pan(ch_index, gain, pan) => match self.channels.get_mut(ch_index) {
					Some(ch) => ch.pan.set(gain, pan),
					None => println!("Channel index out of bounds!"),
				},
				// _ => eprintln!("Didnt handle message!"),
				// AudioMessage::Add => self.add(),
			}
		}
	}
}
