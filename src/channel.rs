use crate::audio::MAX_BUF_SIZE;
use crate::dsp;
use crate::dsp::PeakMeter;
use crate::dsp::smooth::Smooth;
use crate::dsp::{MuteState, time_constant};
use crate::effect::*;
use crate::meters::MeterHandle;
use crate::voice_manager::VoiceManager;

pub struct Channel {
	pub instrument: Option<VoiceManager>,
	pub effects: Vec<Bypass>,
	peak: PeakMeter,
	meter_handle: MeterHandle,
	gain: Smooth,

	mute: bool,
	state: MuteState,
	value: f32,
	smoothing_f: f32,
}

impl Channel {
	pub fn new(sample_rate: f32, instrument: VoiceManager, meter_handle: MeterHandle) -> Self {
		Self {
			instrument: Some(instrument),
			effects: Vec::new(),
			peak: PeakMeter::new(sample_rate),
			meter_handle,
			gain: Smooth::new(1., 25., sample_rate),
			mute: false,
			state: MuteState::Active,
			value: 1.0,
			smoothing_f: time_constant(15.0, sample_rate),
		}
	}

	pub fn process(&mut self, buffer_in: &mut [&mut [f32]; 2], buffer_out: &mut [&mut [f32]; 2]) {
		if let Some(instrument) = &mut self.instrument {
			buffer_in[0].fill(0.0);
			buffer_in[1].fill(0.0);
			instrument.process(buffer_in);
		}
		for fx in &mut self.effects {
			fx.process(buffer_in);
		}

		match self.state {
			MuteState::Off => {
				self.meter_handle.set([0., 0.]);
			},
			MuteState::Active => {
				let samples = buffer_in[0].len();
				assert!(samples <= MAX_BUF_SIZE);
				for i in 0..samples {
					let gain = self.gain.process();
					buffer_in[0][i] *= gain;
					buffer_in[1][i] *= gain;
					buffer_out[0][i] += buffer_in[0][i];
					buffer_out[1][i] += buffer_in[1][i];
				}

				let peak = self.peak.process_block(buffer_in);
				self.meter_handle.set(peak);
			},
			MuteState::Transition => {
				let target = if self.mute { 0.0 } else { 1.0 };

				let samples = buffer_in[0].len();
				assert!(samples <= MAX_BUF_SIZE);

				for i in 0..samples {
					self.value += self.smoothing_f * (target - self.value);
					let gain = self.value * self.gain.process();
					buffer_in[0][i] *= gain;
					buffer_in[1][i] *= gain;
					buffer_out[0][i] += buffer_in[0][i];
					buffer_out[1][i] += buffer_in[1][i];
				}

				let peak = dsp::peak(buffer_in);
				self.meter_handle.set(peak);

				// Check state transition
				if (self.value - target).abs() < 1e-4 {
					self.value = target;
					if self.mute {
						self.state = MuteState::Off;
					} else {
						self.state = MuteState::Active;
					}
				}
			},
		}
	}

	pub fn set_mute(&mut self, mute: bool) {
		self.mute = mute;
		self.state = MuteState::Transition;
	}

	pub fn set_gain(&mut self, gain: f32) {
		self.gain.set(gain);
	}
}
