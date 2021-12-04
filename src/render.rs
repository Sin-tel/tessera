use assert_no_alloc::*;
use ringbuf::{Consumer, Producer};

use crate::defs::*;
use crate::instrument::sine::*;
use crate::instrument::*;

pub struct Render {
	audio_rx: Consumer<AudioMessage>,
	lua_tx: Producer<LuaMessage>,
	channels: Vec<Box<dyn Instrument + Send>>,
	buffer2: Vec<StereoSample>,
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
			channels: Vec::new(),
			buffer2: vec![StereoSample { l: 0.0, r: 0.0 }; MAX_BUF_SIZE],
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
			// for &p in new.get_param_list().iter() {
			// 	self.send(LuaMessage::Param(p));
			// }

			self.channels.push(Box::new(new));
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

		// process all channels
		for ch in self.channels.iter_mut() {
			ch.process(buf_slice);
			for (outsample, insample) in buffer.iter_mut().zip(buf_slice.iter()) {
				outsample.l += insample.l;
				outsample.r += insample.r;
			}
		}
	}

	pub fn parse_messages(&mut self) {
		while let Some(m) = self.audio_rx.pop() {
			match m {
				// todo send to correct channel
				AudioMessage::CV(ch_index, cv) => match self.channels.get_mut(ch_index) {
					Some(ch) => ch.cv(cv.freq, cv.vol),
					None => println!("Channel index out of bounds!"),
				},
				AudioMessage::SetParam(ch_index, index, val) => {
					match self.channels.get_mut(ch_index) {
						Some(ch) => ch.set_param(index, val),
						None => println!("Channel index out of bounds!"),
					}
				}

				AudioMessage::Add => self.add(),
			}
		}
	}
}
