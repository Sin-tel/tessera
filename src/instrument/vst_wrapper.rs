use crate::instrument::*;
use crate::log::*;
use crate::vst3;
use crate::vst3::parameter::N_CHANNELS;
use crate::vst3::scan::probe_vst3;
use crate::vst3::{Vst3Editor, Vst3Processor};
use std::path::PathBuf;

// Hardcoded path for testing
const PATH: &str = r"C:\Program Files\Common Files\VST3\Pianoteq 7.vst3";

#[allow(unused)]
pub struct VstWrapper {
	editor: Vst3Editor,
	processor: Vst3Processor,
	voice_pitches: [i16; N_CHANNELS],
}

#[allow(unused)]
impl Instrument for VstWrapper {
	fn new(sample_rate: f32) -> Self {
		let mut plugin_info = probe_vst3(&PathBuf::from(PATH)).unwrap();
		assert!(plugin_info.len() == 1);
		let plugin = plugin_info.pop().unwrap();

		let (editor, processor) = vst3::load(&plugin, sample_rate).unwrap();

		log_info!("Plugin '{}' loaded succesfully", plugin.name);

		VstWrapper { editor, processor, voice_pitches: [0; 16] }
	}

	fn voice_count(&self) -> usize {
		N_CHANNELS
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		let [bl, br] = buffer;
		self.processor.process(bl, br);
	}

	fn pitch(&mut self, pitch: f32, id: usize) {
		// TODO
	}

	fn pressure(&mut self, pressure: f32, id: usize) {
		// TODO
	}

	fn note_on(&mut self, pitch: f32, vel: f32, id: usize) {
		let base_pitch = pitch.round();
		let pitch_offset = f64::from(pitch - base_pitch);

		let base_pitch = base_pitch as i16;
		self.voice_pitches[id] = base_pitch;

		self.processor
			.events
			.push(vst3::event::note_on(id as i16, base_pitch, vel));

		// normalize pitchbend value (assuming +/- 200c)
		let pitchbend = 0.5 + pitch_offset * 0.25;

		// println!("{}, {}, {}", pitch, pitch_offset, pitchbend);
		self.processor.automation.push(id, 0, pitchbend);
	}

	fn note_off(&mut self, id: usize) {
		let base_pitch = self.voice_pitches[id];

		self.processor
			.events
			.push(vst3::event::note_off(id as i16, base_pitch));
	}
	fn flush(&mut self) {
		// TODO
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		#[allow(clippy::single_match_else)]
		match index {
			0 => println!("UI button pressed ({value})"),
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
