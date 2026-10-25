use crate::dsp::simper::Filter;
use crate::dsp::*;
use crate::instrument::*;

// TODO: same note retrigger logic

#[derive(Debug)]
pub struct Epiano {
	voices: Vec<Voice>,
	sample_rate: f32,
	dc_killer: DcKiller,
	eq: Filter,
	// pickup offset (set by asymmetry)
	x0: f32,
	// pickup distance
	y0: f32,
	gain: f32,
	bell: f32,
}

const N_VOICES: usize = 24;
// static output EQ, tuned by ear
const EQ_FREQ: f32 = 500.;
const EQ_GAIN: f32 = -6.;
// output level, -24 LUFS for a held note at velocity 0.7
const OUTPUT_GAIN: f32 = 0.75;
// seconds after note off until the voice is cut
const RELEASE_TIME: f32 = 1.0;
// bell partials fade out between these (Hz), so they don't pile up at nyquist
const FADE_START: f32 = 12000.;
const FADE_END: f32 = 19000.;

// drive: tine displacement in dB
// level: output trim in dB
// decay: relative time constant for decay
// contact: hammer contact time in ms, at the softest velocity. Shorter is brighter, but also excites
//   the fundamental less: drive and level here make up for it at velocity 0.7.
const REGISTER_PARAM: [(f32, f32, f32, f32); 6] = [
	// drive, level, decay, contact   Note   Hz
	(13.87, 2.87, 2.2, 6.0),     //  C2     65
	(7.68, 2.68, 1.8, 4.0),      //  C3    131
	(-0.10, 2.90, 1.3, 2.6),     //  C4    262
	(-9.27, 4.73, 0.8, 1.6),     //  C5    523
	(-18.0, 0.0, 0.5, 0.9556),   //  C6   1047
	(-20.0, -6.0, 0.15, 0.4778), //  C7   2093
];
// C2 = note number 36
const REGISTER_START: f32 = 36.;
// drive and level are relative to physical scaling at this note (C4)
const REGISTER_REF: f32 = 261.63;

#[derive(Debug)]
struct Voice {
	active: bool,
	note_on: bool,
	timer: usize,
	hammer_freq: f32,
	hammer_phase: f32,
	vel: f32,
	freq: [f32; 4],
	filter: [Filter; 4],
	prev: f32,
	gain: f32,
	gain_recip: f32,
	// gain per filter: fundamental, then the bell partials
	mix: [f32; 4],
	x0: f32,
}

impl Voice {
	fn new(sample_rate: f32) -> Self {
		let filter = [
			Filter::new(sample_rate),
			Filter::new(sample_rate),
			Filter::new(sample_rate),
			Filter::new(sample_rate),
		];

		Self {
			active: false,
			note_on: false,
			timer: 0,
			hammer_freq: 0.01,
			hammer_phase: 0.,
			prev: 0.,
			vel: 0.,
			freq: [0.; 4],
			filter,
			gain: 1.,
			gain_recip: 1.,
			mix: [1., 0., 0., 0.],
			x0: 0.5,
		}
	}
}

// Drive gain, level gain, decay factor and contact time (s) at `pitch`, see REGISTER_PARAM.
// Contact time is interpolated in log, so 1/f is exact between octaves.
fn register_parameters(pitch: f32) -> (f32, f32, f32, f32) {
	let last = REGISTER_PARAM.len() - 1;
	let x = ((pitch - REGISTER_START) / 12.).clamp(0., last as f32);
	let i = (x as usize).min(last - 1);
	let (d0, l0, r0, c0) = REGISTER_PARAM[i];
	let (d1, l1, r1, c1) = REGISTER_PARAM[i + 1];
	let a = x - i as f32;
	let contact = 0.001 * c0 * (c1 / c0).powf(a);
	(from_db(lerp(d0, d1, a)), from_db(lerp(l0, l1, a)), lerp(r0, r1, a), contact)
}

// Hammer force pulse over the contact time, x in [0, 1], peak 1.
fn hammer_pulse(x: f32) -> f32 {
	let x = x.clamp(0., 1.);
	(x * (4. - 4. * x)).powi(3)
}

// Magnetic field at the pickup, for the tine at (x, y) relative to it.
fn pickup_field(x: f32, y: f32) -> f32 {
	let x1 = 0.9 * x + 0.1 * softclip(2.5 * x);
	// let x1 = x;
	(x1 * x1 + y * y).sqrt().recip()
}

impl Instrument for Epiano {
	fn new(sample_rate: f32) -> Self {
		let mut voices = Vec::with_capacity(N_VOICES);
		for _ in 0..N_VOICES {
			voices.push(Voice::new(sample_rate));
		}

		let mut eq = Filter::new(sample_rate);
		eq.set_bell(EQ_FREQ, BUTTERWORTH_Q, EQ_GAIN);
		eq.immediate();

		Epiano {
			sample_rate,
			voices,
			dc_killer: DcKiller::new(sample_rate),
			eq,
			x0: 0.5,
			y0: 0.8,
			gain: 1.,
			bell: 1.,
		}
	}

	fn voice_count(&self) -> usize {
		N_VOICES
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;
		let release_samples = (RELEASE_TIME * self.sample_rate) as usize;

		for voice in self.voices.iter_mut().filter(|v| v.active) {
			for sample in bl.iter_mut() {
				let mut hammer = 0.;
				if voice.hammer_phase < 1.0 {
					voice.hammer_phase += voice.hammer_freq;

					hammer = hammer_pulse(voice.hammer_phase) * voice.vel;
				}

				let mut x = 0.;
				for (filter, &mix) in voice.filter.iter_mut().zip(&voice.mix) {
					if mix > 0. {
						x += mix * filter.process(hammer);
					}
				}

				// pickup magnetic field
				let field = pickup_field(voice.gain * x - voice.x0, self.y0) + hammer * 0.2;

				// output voltage is derivative of magnetic field
				let diff = field - voice.prev;

				voice.prev = field;

				let out = (diff * 20.0) * voice.gain_recip;

				*sample += out * OUTPUT_GAIN;
			}

			if !voice.note_on {
				voice.timer += bl.len();
				if voice.timer > release_samples {
					voice.active = false;
				}
			}
		}

		self.dc_killer.process_block(bl);
		self.eq.process_block(bl);
		br.copy_from_slice(bl);
	}

	fn pitch(&mut self, _pitch: f32, _id: usize) {}
	fn pressure(&mut self, _pressure: f32, _id: usize) {}

	fn note_on(&mut self, pitch: f32, vel: f32, id: usize) {
		let vel = velocity_curve(vel, 36.0);
		let (x0, y0) = (self.x0, self.y0);
		let voice = &mut self.voices[id];
		let f = pitch_to_hz(pitch);

		let (drive, level, decay_factor, contact) = register_parameters(pitch);

		// harder hits are shorter
		let contact = contact / (1. + vel);
		voice.hammer_freq = 1. / (contact * self.sample_rate);

		voice.vel = 0.004 * vel * (self.sample_rate / REGISTER_REF) * drive;

		voice.freq[0] = f;
		// pitch rounded to 10 cents, so the detuning is the same every time a key is played
		let key = 4 * (10. * pitch).round() as i64;
		voice.freq[1] = f * 4.2 + 800. * hash_uniform(key + 1);
		voice.freq[2] = f * 7.3 + 1200. * hash_uniform(key + 2);
		voice.freq[3] = f * 11.3 + 1600. * hash_uniform(key + 3);

		voice.filter[0].set_bandpass(voice.freq[0], voice.freq[0] * 8.0 * decay_factor);
		voice.filter[1].set_bandpass(voice.freq[1], voice.freq[1] * 0.1 * decay_factor);
		voice.filter[2].set_bandpass(voice.freq[2], voice.freq[2] * 0.8 * decay_factor);
		voice.filter[3].set_bandpass(voice.freq[3], voice.freq[3] * 0.6 * decay_factor);

		voice.filter.iter_mut().for_each(Filter::immediate);
		if !voice.note_on {
			// don't reset state in case of voice stealing to minimize clicking
			// a stolen voice keeps its pickup position
			voice.filter.iter_mut().for_each(Filter::reset_state);
			voice.x0 = x0;
			voice.prev = pickup_field(-x0, y0);
		}

		voice.hammer_phase = 0.;
		voice.gain = self.gain;
		// undo the level change from drive (relative to physical), then trim
		voice.gain_recip = self.gain.recip() * (REGISTER_REF / f) / drive * level;
		// fade near nyquist, lower sample rates fade earlier
		for i in 1..4 {
			let fade = ((FADE_END - voice.freq[i]) / (FADE_END - FADE_START)).clamp(0., 1.);
			voice.mix[i] = self.bell * fade;
		}

		voice.active = true;
		voice.note_on = true;
		voice.timer = 0;
	}

	fn note_off(&mut self, id: usize) {
		let voice = &mut self.voices[id];
		voice.note_on = false;
		for (i, v) in voice.filter.iter_mut().enumerate() {
			v.set_bandpass(voice.freq[i], voice.freq[i] * 0.12);
		}
	}

	fn flush(&mut self) {
		self.dc_killer.reset();
		self.eq.reset_state();
		let rest_field = pickup_field(-self.x0, self.y0);
		for voice in &mut self.voices {
			voice.filter.iter_mut().for_each(Filter::reset_state);
			voice.filter.iter_mut().for_each(Filter::immediate);
			voice.hammer_phase = 2.;
			voice.prev = rest_field;
			voice.x0 = self.x0;

			voice.active = false;
			voice.note_on = false;
			voice.timer = 0;
			voice.vel = 0.;
		}
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => self.gain = 2.0 * from_db(value),
			1 => self.x0 = 0.9 - 0.8 * value,
			2 => self.bell = value * value,
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
