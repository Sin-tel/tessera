// use crate::defs::*;
use crate::dsp::delayline::DelayLine;
use crate::dsp::simper::Filter;
use crate::dsp::Smoothed;

// interaural time difference, 660 μs
const ITD: f32 = 0.00066;
// head filter at 4 kHz
const HEAD_CUTOFF: f32 = 4000.0;
const HEAD_Q: f32 = 0.4;

#[derive(Debug)]
struct Bus {
	gain: Smoothed,
	delay: Smoothed,
	filter: Filter,
	delayline: DelayLine,
}

impl Bus {
	pub fn new(sample_rate: f32) -> Self {
		let mut filter = Filter::new(sample_rate);
		filter.set_highshelf(HEAD_CUTOFF, HEAD_Q, 0.0);
		Bus {
			gain: Smoothed::new(1.0, 100.0, sample_rate),
			delay: Smoothed::new(0.0, 100.0, sample_rate),
			filter,
			delayline: DelayLine::new(sample_rate, ITD),
		}
	}
}

#[derive(Debug)]
pub struct Pan {
	buses: [Bus; 2],
}

impl Pan {
	pub fn new(sample_rate: f32) -> Self {
		Pan {
			buses: [Bus::new(sample_rate), Bus::new(sample_rate)],
		}
	}

	pub fn set(&mut self, gain: f32, pan: f32) {
		self.buses[0].delay.set((ITD * pan).max(0.0));
		self.buses[1].delay.set((-ITD * pan).max(0.0));

		let lshelf = -1.5 * pan * (pan + 3.0);
		let rshelf = -1.5 * pan * (pan - 3.0);
		self.buses[0]
			.filter
			.set_highshelf(HEAD_CUTOFF, HEAD_Q, lshelf);
		self.buses[1]
			.filter
			.set_highshelf(HEAD_CUTOFF, HEAD_Q, rshelf);

		let lgain = -0.084 * pan * (pan + 2.53) + 1.0;
		let rgain = -0.084 * pan * (pan - 2.53) + 1.0;
		self.buses[0].gain.set(lgain * gain);
		self.buses[1].gain.set(rgain * gain);
	}

	pub fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		for (s, bus) in buffer.iter_mut().zip(self.buses.iter_mut()) {
			for sample in s.iter_mut() {
				bus.gain.update();
				bus.delay.update();

				let input = sample.clone();

				// delay
				let mut s = bus.delayline.go_back_cubic(bus.delay.value);

				// head shadow filter
				s = bus.filter.process(s);

				// volume difference
				s *= bus.gain.value;

				*sample = s;

				bus.delayline.push(input);
			}
		}
	}
}
