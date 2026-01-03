use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use crate::effect::Effect;
use crate::log::log_warn;
use crate::worker::RequestData;

// Analog-style limiter
// TODO: Add a "modern" mode with lookahead

#[derive(Debug)]
pub struct Limiter {
	sample_rate: f32,

	attack: f32,
	release: f32,
	input_gain: Smooth,
	ceiling: Smooth,
	stereo_link: bool,

	gain: [f32; 2],
}

fn clip(x: f32) -> f32 {
	let x = x.clamp(-5. / 4., 5. / 4.);
	x - (256. / 3125.) * x.powi(5)
}

impl Effect for Limiter {
	fn new(sample_rate: f32) -> Self {
		Self {
			sample_rate,
			// Fixed attack
			attack: time_constant(5.0, sample_rate),
			release: time_constant(400.0, sample_rate),
			input_gain: Smooth::new(1.0, 50.0, sample_rate),
			ceiling: Smooth::new(1.0, 50.0, sample_rate),
			stereo_link: true,
			gain: [1.0, 1.0],
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;

		for (l, r) in bl.iter_mut().zip(br.iter_mut()) {
			let mut in_gain = self.input_gain.process();
			let ceiling = self.ceiling.process();
			in_gain /= ceiling;

			let x = [*l * in_gain, *r * in_gain];
			let mut peak = x.map(f32::abs);

			if self.stereo_link {
				let max_peak = f32::max(peak[0], peak[1]);
				peak = [max_peak, max_peak];
			}

			let mut out = [0., 0.];

			for ch in 0..2 {
				let target = if peak[ch] > 1.0 { 1.0 / peak[ch] } else { 1.0 };
				if target < self.gain[ch] {
					self.gain[ch] += (target - self.gain[ch]) * self.attack;
				} else {
					self.gain[ch] += (target - self.gain[ch]) * self.release;
				}

				out[ch] = x[ch] * self.gain[ch];
				out[ch] = clip(out[ch] * 1.1);
			}

			*l = out[0] * ceiling;
			*r = out[1] * ceiling;
		}
	}

	fn flush(&mut self) {
		self.gain = [1.0, 1.0];
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.release = time_constant(value * 2.0, self.sample_rate),
			1 => self.input_gain.set(from_db(value)),
			2 => self.ceiling.set(from_db(value)),
			3 => self.stereo_link = value > 0.5,
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
