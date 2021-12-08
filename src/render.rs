use assert_no_alloc::*;
use ringbuf::{Consumer, Producer};

use crate::defs::*;
use crate::instrument::sine::*;
use crate::instrument::*;

pub struct Render {
	audio_rx: Consumer<AudioMessage>,
	lua_tx: Producer<LuaMessage>,
	instruments: Vec<Box<dyn Instrument + Send>>,
	effects: Vec<Vec<Box<dyn Effect + Send>>>,
	// buffer2: Vec<StereoSample>,
	buffer2: [StereoSample; MAX_BUF_SIZE],
	sample_rate: f32,
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
			instruments: Vec::new(),
			effects: Vec::new(),
			// buffer2: vec![StereoSample { l: 0.0, r: 0.0 }; MAX_BUF_SIZE],
			buffer2: [StereoSample { l: 0.0, r: 0.0 }; MAX_BUF_SIZE],
			sample_rate,
		}
	}

	fn send(&mut self, m: LuaMessage) {
		if !self.lua_tx.is_full() {
			self.lua_tx.push(m).unwrap();
		} else {
			println!("Lua queue full. Dropped message!")
		}
	}

	pub fn add(&mut self) {
		permit_alloc(|| {
			let new = Sine::new(self.sample_rate);
			self.instruments.push(Box::new(new));
			self.effects.push(Vec::new());
		});

		// arbitrary test value
		self.send(LuaMessage::Test(42.0));
	}

	pub fn process(&mut self, buffer: &mut [StereoSample]) {
		let buf_slice = &mut self.buffer2[..buffer.len()];

		// zero buffer
		for sample in buffer.iter_mut() {
			sample.l = 0.0;
			sample.r = 0.0;
		}

		// process all instruments
		for ch in self.instruments.iter_mut() {
			ch.process(buf_slice);
			for (outsample, insample) in buffer.iter_mut().zip(buf_slice.iter()) {
				outsample.l += insample.l;
				outsample.r += insample.r;
			}
		}

		// default 12dB headroom + tanh
		for sample in buffer.iter_mut() {
			sample.l = (sample.l * 0.25).tanh();
			sample.r = (sample.r * 0.25).tanh();
		}
	}

	pub fn parse_messages(&mut self) {
		while let Some(m) = self.audio_rx.pop() {
			match m {
				// todo send to correct channel
				AudioMessage::CV(ch_index, cv) => match self.instruments.get_mut(ch_index) {
					Some(ch) => ch.cv(cv.pitch, cv.vel),
					None => println!("Channel index out of bounds!"),
				},
				AudioMessage::NoteOn(ch_index, cv) => match self.instruments.get_mut(ch_index) {
					Some(ch) => ch.note_on(cv.pitch, cv.vel),
					None => println!("Channel index out of bounds!"),
				},
				AudioMessage::SetParam(ch_index, device_index, index, val) => {
					if device_index == 0 {
						match self.instruments.get_mut(ch_index) {
							Some(ch) => ch.set_param(index, val),
							None => println!("Channel index out of bounds!"),
						}
					} else {
						match self.effects.get_mut(ch_index) {
							Some(ch) => match ch.get_mut(device_index - 1) {
								Some(ch) => ch.set_param(index, val),
								None => println!("Device index out of bounds!"),
							},
							None => println!("Channel index out of bounds!"),
						}
					}
				}

				AudioMessage::Add => self.add(),
			}
		}
	}
}
