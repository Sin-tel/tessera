// Rotary speaker model.
// Horn and drum are modeled as rotating sources read from a delay line,
// with two mics and two reflection paths.

use crate::dsp::delayline::DelayLine;
use crate::dsp::onepole::{OnePoleCoefs, OnePoleState};
use crate::dsp::simper::Filter;
use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use crate::effect::*;
use crate::worker::RequestData;

// m/s
const SPEED_OF_SOUND: f32 = 343.;
// m
const MIC_DISTANCE: f32 = 0.4;
const REFLECTION_PATH: f32 = 2.6;

const REFLECTION_GAIN: f32 = 0.4;

const CROSSOVER: f32 = 800.;
const HORN_LP: f32 = 6500.;

const HORN_LEVEL: f32 = 1.6;
const HORN_RADIUS: f32 = 0.16;
const HORN_AMP_MOD: f32 = 0.6;

const DRUM_LEVEL: f32 = 1.0;
const DRUM_RADIUS: f32 = 0.24;
const DRUM_AMP_MOD: f32 = 0.3;

// cutoff of the horn as it faces away / towards the mic
const HORN_CUTOFF_MIN: f32 = 3000.;
const HORN_CUTOFF_MAX: f32 = 12000.;

// mic L direct, L reflection, R direct, R reflection
// (offset in turns, extra path in m)
// reflections come from the wall behind the rotor
const TAPS: [(f32, f32); 4] = [(0., 0.), (0.5, REFLECTION_PATH), (0.5, 0.), (0., REFLECTION_PATH)];

#[derive(Debug, Clone, Copy, Default)]
struct TapParams {
	delay: f32,
	gain: f32,
	coefs: OnePoleCoefs,
}

impl TapParams {
	fn lerp(&self, other: &Self, a: f32) -> Self {
		Self {
			delay: lerp(self.delay, other.delay, a),
			gain: lerp(self.gain, other.gain, a),
			coefs: self.coefs.lerp(&other.coefs, a),
		}
	}
}

#[derive(Debug, Default)]
struct Tap {
	prev: TapParams,
	next: TapParams,
	filter: OnePoleState,
}

#[derive(Debug)]
struct Rotor {
	radius: f32,
	mod_amplitude: f32,
	is_horn: bool,
	// turns
	phase: f32,
	// Hz
	speed: f32,
	delay: DelayLine,
	taps: [Tap; 4],
}

impl Rotor {
	fn new(radius: f32, mod_amplitude: f32, is_horn: bool, sample_rate: f32) -> Self {
		Self {
			radius,
			mod_amplitude,
			is_horn,
			phase: 0.,
			speed: 0.0,
			delay: DelayLine::new(sample_rate, 0.1),
			taps: Default::default(),
		}
	}

	fn advance(&mut self, len: usize) {
		self.phase += self.speed * len as f32;
		self.phase -= fast_floor(self.phase);
	}

	// updated per block
	fn update_taps(&mut self, width: f32, depth: f32, sample_rate: f32) {
		for (tap, (offset, extra)) in self.taps.iter_mut().zip(TAPS) {
			let c = sin_cheap(self.phase + width * offset);
			// first order approximation of the distance
			let delay = (MIC_DISTANCE - depth * self.radius * c + extra) / SPEED_OF_SOUND;
			let gain = 1.0 + self.mod_amplitude * c;

			let coefs = if self.is_horn {
				let facing = 0.5 + 0.5 * c;
				let cutoff = lerp(HORN_CUTOFF_MIN, HORN_CUTOFF_MAX, facing);
				OnePoleCoefs::lowpass(cutoff, sample_rate)
			} else {
				// filter not used here
				OnePoleCoefs::default()
			};

			tap.prev = tap.next;
			tap.next = TapParams { delay, gain, coefs };
		}
	}

	// sum of taps per mic
	fn read(&mut self, a: f32) -> [f32; 2] {
		let mut out = [0.; 2];
		for (k, tap) in self.taps.iter_mut().enumerate() {
			let p = tap.prev.lerp(&tap.next, a);
			let mut s = self.delay.go_back_cubic(p.delay);

			if self.is_horn {
				s = tap.filter.process(&p.coefs, s);
			}
			s *= p.gain;

			let mic = k / 2;
			if k % 2 == 0 {
				out[mic] += s;
			} else {
				out[mic] += REFLECTION_GAIN * s;
			}
		}
		out
	}

	fn flush(&mut self, phase: f32, width: f32, depth: f32, sample_rate: f32) {
		self.delay.flush();
		self.phase = phase;
		for tap in &mut self.taps {
			tap.filter.reset();
		}
		self.update_taps(width, depth, sample_rate);
		self.update_taps(width, depth, sample_rate);
	}
}

#[derive(Debug)]
pub struct Rotary {
	sample_rate: f32,
	horn: Rotor,
	drum: Rotor,
	horn_hp: Filter,
	horn_lp: Filter,
	drum_lp: Filter,
	width: f32,
	depth: f32,
	mix: Smooth,
	// aligns dry with the direct path
	dry_delay: [DelayLine; 2],
}

impl Effect for Rotary {
	fn new(sample_rate: f32) -> Self {
		let mut horn_hp = Filter::new(sample_rate);
		horn_hp.set_highpass(CROSSOVER, BUTTERWORTH_Q);
		let mut horn_lp = Filter::new(sample_rate);
		horn_lp.set_lowpass(HORN_LP, BUTTERWORTH_Q);
		let mut drum_lp = Filter::new(sample_rate);
		drum_lp.set_lowpass(CROSSOVER, BUTTERWORTH_Q);

		Self {
			sample_rate,
			horn: Rotor::new(HORN_RADIUS, HORN_AMP_MOD, true, sample_rate),
			drum: Rotor::new(DRUM_RADIUS, DRUM_AMP_MOD, false, sample_rate),
			horn_hp,
			horn_lp,
			drum_lp,
			width: 0.5,
			depth: 1.0,
			mix: Smooth::new(1., 25., sample_rate),
			dry_delay: std::array::from_fn(|_| DelayLine::new(sample_rate, 0.01)),
		}
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;
		let len = bl.len();

		self.horn.advance(len);
		self.drum.advance(len);
		self.horn.update_taps(self.width, self.depth, self.sample_rate);
		self.drum.update_taps(self.width, self.depth, self.sample_rate);

		for (i, (l, r)) in bl.iter_mut().zip(br.iter_mut()).enumerate() {
			let a = (i + 1) as f32 / len as f32;
			let x = 0.5 * (*l + *r);

			let h = self.horn_lp.process(self.horn_hp.process(x));
			let d = self.drum_lp.process(x);
			self.horn.delay.push(h);
			self.drum.delay.push(d);

			let horn = self.horn.read(a);
			let drum = self.drum.read(a);

			let wet_l = HORN_LEVEL * horn[0] + DRUM_LEVEL * drum[0];
			let wet_r = HORN_LEVEL * horn[1] + DRUM_LEVEL * drum[1];

			let [dl, dr] = &mut self.dry_delay;
			dl.push(*l);
			dr.push(*r);
			let t = MIC_DISTANCE / SPEED_OF_SOUND;
			let dry_l = dl.go_back_int(t);
			let dry_r = dr.go_back_int(t);

			let mix = self.mix.process();
			*l = lerp(dry_l, wet_l, mix);
			*r = lerp(dry_r, wet_r, mix);
		}
	}

	fn flush(&mut self) {
		self.horn.flush(0., self.width, self.depth, self.sample_rate);
		// drum starts a quarter turn off so the rotors don't line up
		self.drum.flush(0.25, self.width, self.depth, self.sample_rate);
		self.horn_hp.reset_state();
		self.horn_lp.reset_state();
		self.drum_lp.reset_state();
		self.horn_hp.immediate();
		self.horn_lp.immediate();
		self.drum_lp.immediate();
		self.mix.immediate();
		for d in &mut self.dry_delay {
			d.flush();
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.mix.set(value),
			1 => {
				self.horn.speed = value / self.sample_rate;
				self.drum.speed = 0.86 * value / self.sample_rate;
			},
			2 => self.depth = value,
			3 => self.width = value,
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
