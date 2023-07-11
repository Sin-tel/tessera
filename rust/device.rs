use crate::effect::Effect;
use crate::effect::{delay::Delay, drive::Drive, gain::Gain, pan::Pan, reverb::Reverb};
use crate::instrument::Instrument;
use crate::instrument::{
	analog::Analog, fm::Fm, polysine::Polysine, sine::Sine, wavetable::Wavetable,
};

// list of instruments
pub fn new_instrument(sample_rate: f32, instrument_number: usize) -> Box<dyn Instrument + Send> {
	match instrument_number {
		0 => Box::new(Sine::new(sample_rate)),
		1 => Box::new(Wavetable::new(sample_rate)),
		2 => Box::new(Analog::new(sample_rate)),
		3 => Box::new(Fm::new(sample_rate)),
		4 => Box::new(Polysine::new(sample_rate)),
		_ => {
			eprintln!("Instrument with number {instrument_number} not found. Returning default.");
			Box::new(Sine::new(sample_rate))
		}
	}
}

// list of effects
pub fn new_effect(sample_rate: f32, effect_number: usize) -> Box<dyn Effect + Send> {
	match effect_number {
		0 => Box::new(Pan::new(sample_rate)),
		1 => Box::new(Gain::new(sample_rate)),
		2 => Box::new(Drive::new(sample_rate)),
		3 => Box::new(Delay::new(sample_rate)),
		4 => Box::new(Reverb::new(sample_rate)),
		_ => {
			eprintln!("Effect with number {effect_number} not found. Returning default.");
			Box::new(Gain::new(sample_rate))
		}
	}
}
