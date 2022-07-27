// can autogen this
use crate::effect::gain::Gain;
use crate::effect::Effect;
use crate::instrument::sine::Sine;
use crate::instrument::Instrument;

// list of instruments
pub fn new_instrument(sample_rate: f32, instrument_number: usize) -> Box<dyn Instrument + Send> {
	Box::new(match instrument_number {
		0 => Sine::new(sample_rate),
		_ => {
			eprintln!(
				"Instrument with number {} not found. Returning default.",
				instrument_number
			);
			Sine::new(sample_rate)
		}
	})
}

// list of effects
pub fn new_effect(sample_rate: f32, effect_number: usize) -> Box<dyn Effect + Send> {
	Box::new(match effect_number {
		0 => Gain::new(sample_rate),
		_ => {
			eprintln!(
				"Instrument with number {} not found. Returning default.",
				effect_number
			);
			Gain::new(sample_rate)
		}
	})
}

pub trait Param {
	fn set_param(&mut self, index: usize, val: f32);
}
