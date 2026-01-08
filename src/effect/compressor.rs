use crate::dsp::simper::Filter;
use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use crate::effect::*;
use crate::worker::RequestData;

// TODO: switch out low-cut / bandpass on sidechain

const KNEE: f32 = 3.0;
const ATTACK_MIX: f32 = 0.15;

#[derive(Debug)]
pub struct Compressor {
	tracks: [Track; 2],
	sample_rate: f32,
	threshold: f32,
	ratio: f32,
	attack: f32,
	attack_slow: f32,
	release: f32,
	balance: Smooth,
	make_up: Smooth,

	gain_a: f32,
	gain_b: f32,
	gain_c: f32,
}

#[derive(Debug)]
struct Track {
	highpass: Filter,
	shelf: Filter,
}

impl Track {
	pub fn new(sample_rate: f32) -> Self {
		let mut highpass = Filter::new(sample_rate);
		highpass.set_highpass(100.0, BUTTERWORTH_Q);

		let mut shelf = Filter::new(sample_rate);
		shelf.set_highshelf(1500.0, 0.5, 4.0);

		Track { highpass, shelf }
	}
}

impl Effect for Compressor {
	fn new(sample_rate: f32) -> Self {
		Compressor {
			tracks: [Track::new(sample_rate), Track::new(sample_rate)],
			sample_rate,
			threshold: 0.,
			ratio: 1.,
			attack: 0.1,
			attack_slow: 0.1,
			release: 0.1,
			make_up: Smooth::new(0., 25.0, sample_rate),
			balance: Smooth::new(1., 25.0, sample_rate),

			gain_a: 0.,
			gain_b: 0.,
			gain_c: 0.,
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;
		for (l, r) in bl.iter_mut().zip(br.iter_mut()) {
			let s_in = [*l, *r];

			let mut side = [0., 0.];
			for ch in 0..2 {
				// sidechain loudness weight
				side[ch] = self.tracks[ch].highpass.process(s_in[ch]);
				side[ch] = self.tracks[ch].shelf.process(side[ch]);
			}

			// stereo-link peak
			let mut peak = f32::max(side[0].abs(), side[1].abs());

			// to log
			peak = to_db(peak + 0.0001);

			// gain computer
			peak -= self.threshold;
			if peak < -KNEE {
				peak = 0.0;
			} else if peak < KNEE {
				peak = (peak + KNEE).powi(2) / (4. * KNEE);
			}
			peak = (1. - self.ratio) * peak / self.ratio;

			// release envelope
			if peak < self.gain_a {
				self.gain_a = peak;
			} else {
				self.gain_a = lerp(self.gain_a, peak, self.release);
			}

			// attack smoothing
			self.gain_b = lerp(self.gain_b, self.gain_a, self.attack);
			self.gain_c = lerp(self.gain_c, self.gain_a, self.attack_slow);

			let mut g = ATTACK_MIX * self.gain_b + (1. - ATTACK_MIX) * self.gain_c;

			// back to linear
			let make_up = self.make_up.process();
			g = from_db(g + make_up);

			let balance = self.balance.process();
			g = g * balance + (1. - balance);

			*l = s_in[0] * g;
			*r = s_in[1] * g;
		}
	}
	fn flush(&mut self) {}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.balance.set(value),
			1 => self.threshold = value,
			2 => self.ratio = value,
			3 => {
				self.attack = time_constant(value * 2_000., self.sample_rate);
				self.attack_slow = time_constant(value * 64_000., self.sample_rate);
			},
			4 => self.release = time_constant(value * 5_000., self.sample_rate),
			5 => self.make_up.set(value),
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
