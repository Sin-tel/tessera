use crate::instrument::*;
use crate::log::*;
use crate::vst3;
use crate::vst3::Vst3Processor;
use crate::vst3::parameter::N_CHANNELS;
use std::any::Any;

#[allow(unused)]
pub struct VstWrapper {
	processor: Option<Vst3Processor>,
	voice_pitches: [i16; N_CHANNELS],
	mpe_initialized: bool,
}

#[allow(unused)]
impl Instrument for VstWrapper {
	fn new(sample_rate: f32) -> Self {
		VstWrapper { processor: None, voice_pitches: [0; N_CHANNELS], mpe_initialized: false }
	}

	fn voice_count(&self) -> usize {
		N_CHANNELS
	}

	fn process(&mut self, buffer: &mut [&mut [f32]; 2]) {
		if let Some(processor) = &mut self.processor {
			if !self.mpe_initialized {
				processor.automation.mpe_init();
				self.mpe_initialized = true;
			}

			let [bl, br] = buffer;
			processor.process(bl, br);
		}
	}

	fn pitch(&mut self, pitch: f32, id: usize) {
		// TODO
	}

	fn pressure(&mut self, pressure: f32, id: usize) {
		// TODO
	}

	fn note_on(&mut self, pitch: f32, vel: f32, id: usize) {
		if let Some(processor) = &mut self.processor {
			let base_pitch = pitch.round();
			let pitch_offset = f64::from(pitch - base_pitch);

			let base_pitch = base_pitch as i16;
			self.voice_pitches[id] = base_pitch;

			processor.events.push(vst3::event::note_on(id, base_pitch, vel));

			// normalize pitchbend value
			const PB_RANGE: f64 = 48.0;
			let pitchbend = 0.5 + pitch_offset * (0.5 / PB_RANGE);

			processor.automation.push_pitchend(id, pitchbend);
		}
	}

	fn note_off(&mut self, id: usize) {
		if let Some(processor) = &mut self.processor {
			let base_pitch = self.voice_pitches[id];
			processor.events.push(vst3::event::note_off(id, base_pitch));
		}
	}
	fn flush(&mut self) {
		// TODO
	}

	fn receive_data(&mut self, data: ResponseData) -> Option<Box<dyn Any + Send>> {
		assert!(self.processor.is_none());
		if let ResponseData::Vst3Processor(processor) = data {
			self.processor = Some(*processor);
		}
		None
	}

	fn set_parameter(&mut self, index: usize, value: f32) -> Option<RequestData> {
		#[allow(clippy::single_match_else)]
		match index {
			0 => {
				// This corresponds to the ui button. Ignore.
			},
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}
}
