use crate::dsp::simper::Filter;
use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use crate::effect::*;
use crate::worker::RequestData;

// TODO: switch out low-cut / bandpass on sidechain
// TODO: stereo/mono split gain calculation

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
}

#[derive(Debug)]
struct Track {
	gain_a: f32,
	gain_b: f32,
	gain_c: f32,

	balance: Smooth,
	make_up: Smooth,
	highpass: Filter,
	shelf: Filter,
}

impl Track {
	pub fn new(sample_rate: f32) -> Self {
		let mut highpass = Filter::new(sample_rate);
		highpass.set_highpass(100.0, BUTTERWORTH_Q);

		let mut shelf = Filter::new(sample_rate);
		shelf.set_highshelf(1500.0, 0.5, 0.0);

		Track {
			gain_a: 0.,
			gain_b: 0.,
			gain_c: 0.,
			make_up: Smooth::new(0., 25.0, sample_rate),
			balance: Smooth::new(1., 25.0, sample_rate),
			highpass,
			shelf,
		}
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
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		for (buf, track) in buffer.iter_mut().zip(self.tracks.iter_mut()) {
			for sample in buf.iter_mut() {
				let s_in = *sample;

				// sidechain loudness weight
				let mut s_w = track.highpass.process(s_in);
				s_w = track.shelf.process(s_w);

				// to log
				let mut peak = to_db(s_w.abs() + 0.0001);

				// gain computer
				peak -= self.threshold;
				if peak < -KNEE {
					peak = 0.0;
				} else if peak < KNEE {
					peak = (peak - KNEE).powi(2) / (4. * KNEE);
				}
				peak = (1. - self.ratio) * peak / self.ratio;

				// release envelope
				if peak < track.gain_a {
					track.gain_a = peak;
				} else {
					track.gain_a = lerp(track.gain_a, peak, self.release);
				}

				// attack smoothing
				track.gain_b = lerp(track.gain_b, track.gain_a, self.attack);
				track.gain_c = lerp(track.gain_c, track.gain_a, self.attack_slow);

				let mut g = ATTACK_MIX * track.gain_b + (1. - ATTACK_MIX) * track.gain_c;

				// back to linear
				let make_up = track.make_up.process();
				g = from_db(g + make_up);

				let balance = track.balance.process();
				g = g * balance + (1. - balance);

				*sample = s_in * g;
			}
		}
	}
	fn flush(&mut self) {}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => {
				self.tracks[0].balance.set(value);
				self.tracks[1].balance.set(value);
			},
			1 => self.threshold = value,
			2 => self.ratio = value,
			3 => {
				self.attack = time_constant(value * 2_000., self.sample_rate);
				self.attack_slow = time_constant(value * 64_000., self.sample_rate);
			},
			4 => self.release = time_constant(value * 5_000., self.sample_rate),
			5 => {
				self.tracks[0].make_up.set(value);
				self.tracks[1].make_up.set(value);
			},
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
