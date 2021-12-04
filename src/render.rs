use assert_no_alloc::*;
use ringbuf::Consumer;

use crate::defs::*;
use crate::instrument::sine::*;
use crate::instrument::*;

pub struct Render {
	cons: Consumer<Message>,
	pub channels: Vec<Box<dyn Instrument + Send>>,
	buffer2: Vec<StereoSample>,
	sample_rate: f32,
}

impl Render {
	pub fn new(sample_rate: f32, cons: Consumer<Message>) -> Render {
		Render {
			cons,
			channels: Vec::new(),
			buffer2: vec![StereoSample { l: 0.0, r: 0.0 }; MAX_BUF_SIZE],
			sample_rate,
		}
	}

	pub fn add(&mut self) {
		permit_alloc(|| {
			self.channels.push(Box::new(Sine::new(self.sample_rate)));
		});
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
		while let Some(m) = self.cons.pop() {
			match m {
				// todo send to correct channel
				Message::CV(index, cv) => match self.channels.get_mut(index) {
					Some(ch) => ch.cv(cv.freq, cv.vol),
					None => println!("Channel index out of bounds!"),
				},
				Message::Add => self.add(),
			}
		}
	}
}
