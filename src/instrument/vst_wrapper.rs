use crate::instrument::*;
use crate::log::*;
use crate::vst3;
use crate::vst3::Vst3Processor;
use crate::vst3::Vst3State;
use crate::vst3::parameter::N_CHANNELS;

#[allow(unused)]
pub struct VstWrapper {
	processor: Option<Vst3Processor>,
	voice_pitches: [i16; N_CHANNELS],
	mpe_initialized: bool,
	pb_range: f64,
}

impl VstWrapper {
	pub fn get_state(&self) -> Option<String> {
		if let Some(processor) = &self.processor
			&& let Ok(state) = processor.get_state()
		{
			return Some(state.into_string());
		}
		None
	}

	pub fn set_state(&mut self, state: &Vst3State) {
		if let Some(processor) = &self.processor
			&& let Err(e) = processor.set_state(state)
		{
			log_error!("{e}");
		}
	}

	pub fn set_processor(&mut self, processor: Vst3Processor) {
		assert!(self.processor.is_none());
		self.processor = Some(processor);
	}
}

impl Instrument for VstWrapper {
	fn new(_sample_rate: f32) -> Self {
		VstWrapper {
			processor: None,
			voice_pitches: [0; N_CHANNELS],
			mpe_initialized: false,
			pb_range: 48.0,
		}
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
		if let Some(processor) = &mut self.processor {
			let base_pitch = f32::from(self.voice_pitches[id]);
			let pitch_offset = f64::from(pitch - base_pitch);

			// normalize pitchbend value
			let pitchbend = 0.5 + pitch_offset * (0.5 / self.pb_range);
			processor.automation.push_pitchend(id, pitchbend);
		}
	}

	fn pressure(&mut self, pressure: f32, id: usize) {
		if let Some(processor) = &mut self.processor {
			processor.automation.push_pressure(id, f64::from(pressure));
		}
	}

	fn note_on(&mut self, pitch: f32, vel: f32, id: usize) {
		if let Some(processor) = &mut self.processor {
			let base_pitch = pitch.round();
			let pitch_offset = f64::from(pitch - base_pitch);

			let base_pitch = base_pitch as i16;
			self.voice_pitches[id] = base_pitch;

			processor.events.push(vst3::event::note_on(id, base_pitch, vel));

			// normalize pitchbend value
			let pitchbend = 0.5 + pitch_offset * (0.5 / self.pb_range);
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
		if let Some(processor) = &mut self.processor {
			let _ = processor.flush();
		}
	}

	fn set_parameter(&mut self, index: usize, _value: f32) -> Option<RequestData> {
		#[allow(clippy::single_match_else)]
		match index {
			0 => {
				// This corresponds to the ui button. Ignore.
			},
			_ => log_warn!("Parameter with index {index} not found"),
		}
		None
	}

	fn as_vst(&mut self) -> &mut VstWrapper {
		self
	}
}
