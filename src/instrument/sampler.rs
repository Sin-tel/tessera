use crate::audio::MAX_BUF_SIZE;
use crate::dsp::env::AttackRelease;
use crate::dsp::smooth::Smooth;
use crate::dsp::*;
use crate::instrument::Instrument;
use crate::log::log_warn;
use crate::worker::{RequestData, ResponseData};
use halfband::iir;
use std::any::Any;
use std::sync::Arc;

// Note: because of interpolation scheme we need one sample of padding at the start

#[rustfmt::skip]
const PATHS: &[&str] = &[
	"samples/bassdrum_c1.wav",
	"samples/bell_c6.wav",
	"samples/flute_e5.wav",
	"samples/glass_c7.wav",
	"samples/glockenspiel_c6.wav",
	"samples/gong_1_a2.wav",
	"samples/gong_2_e4.wav",
	"samples/harp_c5.wav",
	"samples/kalimba_1_g4.wav",
	"samples/kalimba_2_c5.wav",
	"samples/kalimba_3_a3.wav",
	"samples/marimba_g3.wav",
	"samples/perc_1_a2.wav",
	"samples/perc_2_c3.wav",
	"samples/scrape_c5.wav",
	"samples/timpani_e4.wav",
	"samples/trombone_f3.wav",
	"samples/tuba_f2.wav",
	"samples/tubular_bell_f4.wav",
	"samples/vox_f4.wav",
	"samples/xylophone_g4.wav",
];

const VOICE_COUNT: usize = 16;

impl Sampler {
	fn calculate_f(&self, pitch: f32) -> f32 {
		// Assuming samples are stored in 44100 hz, root note translates to C5
		// Factor 0.5 for downsampling
		pitch_to_hz(pitch - self.root_note) * 0.5 * 44100.0 / (C5_HZ * self.sample_rate)
	}
}

fn pitch_from_filename(path: &str) -> Option<f32> {
	let name = std::path::Path::new(path).file_stem()?.to_str()?;

	// Split by common separators and try to parse each part starting from the last
	for part in name.split(|c| c == '_' || c == '-' || c == ' ').rev() {
		if let Some(p) = parse_pitch(part) {
			return Some(p);
		}
	}
	None
}

// Parse string like "C#3" to number of semitones relative to C5
fn parse_pitch(s: &str) -> Option<f32> {
	let s = s.trim();
	let mut chars = s.chars();

	let note = chars.next()?.to_ascii_uppercase();
	let mut semitone = match note {
		'C' => 0,
		'D' => 2,
		'E' => 4,
		'F' => 5,
		'G' => 7,
		'A' => 9,
		'B' => 11,
		_ => return None,
	};

	// optional accidental
	let mut next = chars.next()?;
	if next == '#' {
		semitone += 1;
		next = chars.next()?;
	} else if next == 'b' {
		semitone -= 1;
		next = chars.next()?;
	}

	// octave number
	let octave: i32 = next.to_digit(10)?.try_into().unwrap();

	Some(((octave - 5) * 12 + semitone) as f32)
}

struct Voice {
	active: bool,
	note_on: bool,
	position: f32,
	f: f32,

	amp_env: AttackRelease,
	gain: Smooth,
	vel: f32,
}

impl Voice {
	fn new(sample_rate: f32) -> Self {
		Self {
			active: false,
			note_on: false,
			position: 0.0,
			f: 1.0,
			amp_env: AttackRelease::new(2.0, 200.0, sample_rate),
			gain: Smooth::new(0., 25.0, sample_rate),
			vel: 0.0,
		}
	}

	fn note_on(&mut self, f: f32, velocity: f32) {
		self.active = true;
		self.note_on = true;
		self.position = 0.0;
		self.vel = velocity;
		self.f = f;
		self.amp_env.set(1.0);
	}

	fn note_off(&mut self) {
		self.note_on = false;
		self.amp_env.set(0.0);
	}
}

pub struct Sampler {
	voices: [Voice; VOICE_COUNT],
	downsampler: [iir::Downsampler8; 2],
	buffer_l: [f32; 2 * MAX_BUF_SIZE],
	buffer_r: [f32; 2 * MAX_BUF_SIZE],
	sample: Option<Arc<[Vec<f32>; 2]>>,
	loading: bool,
	sample_rate: f32,
	root_note: f32,
}

impl Instrument for Sampler {
	fn new(sample_rate: f32) -> Self {
		let voices = std::array::from_fn(|_| Voice::new(sample_rate));
		Self {
			voices,
			sample: None,
			sample_rate,
			root_note: 0.0,
			loading: true,
			downsampler: [iir::Downsampler8::default(), iir::Downsampler8::default()],
			buffer_l: [0.; 2 * MAX_BUF_SIZE],
			buffer_r: [0.; 2 * MAX_BUF_SIZE],
		}
	}

	fn voice_count(&self) -> usize {
		VOICE_COUNT
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let n = 2 * buffer[0].len();
		let bl = &mut self.buffer_l[..n];
		let br = &mut self.buffer_r[..n];
		bl.fill(0.0);
		br.fill(0.0);

		if let Some(sample_data) = &self.sample {
			for voice in self.voices.iter_mut().filter(|v| v.active) {
				let len = sample_data[0].len();

				for (l, r) in bl.iter_mut().zip(br.iter_mut()) {
					if voice.position as usize >= len - 3 {
						// sample finished
						voice.active = false;
						break;
					}
					let env = voice.amp_env.process();
					let gain = voice.gain.process();

					// interpolation
					let (i, frac) = make_usize_frac(voice.position);

					let mut out = [0., 0.];
					for ch in 0..2 {
						let y0 = sample_data[ch][i];
						let y1 = sample_data[ch][i + 1];
						let y2 = sample_data[ch][i + 2];
						let y3 = sample_data[ch][i + 3];
						out[ch] = hermite4(y0, y1, y2, y3, frac);
					}

					let out_gain = env * voice.vel * gain;
					*l += out[0] * out_gain;
					*r += out[1] * out_gain;

					voice.position += voice.f;
				}

				// if envelope finished, kill voice
				if !voice.note_on && voice.amp_env.get() < 0.0001 {
					voice.active = false;
					continue;
				}
			}
		}

		self.downsampler[0].process_block(bl, buffer[0]);
		self.downsampler[1].process_block(br, buffer[1]);
	}

	fn pitch(&mut self, pitch: f32, id: usize) {
		let f = self.calculate_f(pitch);
		self.voices[id].f = f;
	}

	fn pressure(&mut self, _pressure: f32, _id: usize) {
		// todo
	}

	fn note_on(&mut self, pitch: f32, vel: f32, id: usize) {
		if self.loading {
			return;
		}
		let f = self.calculate_f(pitch);
		self.voices[id].note_on(f, vel);
	}

	fn note_off(&mut self, id: usize) {
		self.voices[id].note_off();
	}

	fn flush(&mut self) {
		for v in &mut self.voices {
			v.active = false;
			v.amp_env.set_immediate(0.0);
		}
	}
	fn receive_data(&mut self, data: ResponseData) -> Option<Box<dyn Any + Send>> {
		if let ResponseData::Sample(sample) = data {
			assert_eq!(sample[0].len(), sample[1].len());
			self.sample = Some(sample);
			self.loading = false;
		}
		None
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		match index {
			0 => {
				let idx = (value as usize).max(1) - 1;
				if let Some(&path) = PATHS.get(idx) {
					self.loading = true;
					self.root_note = pitch_from_filename(path).unwrap_or(0.);
					self.flush();
					return Some(RequestData::Sample(path));
				}
				log_warn!("Sample index out of bounds: {}", idx);
			},
			1 => {
				let gain = from_db(value);
				self.voices.iter_mut().for_each(|v| v.gain.set(gain))
			},
			2 => self.voices.iter_mut().for_each(|v| v.amp_env.set_attack(value)),
			3 => self.voices.iter_mut().for_each(|v| v.amp_env.set_release(value)),
			_ => log_warn!("Parameter {} not found", index),
		}
		None
	}
}
