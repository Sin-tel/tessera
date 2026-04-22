use crate::vst3::error::ToResultExt;
use std::cell::UnsafeCell;
use vst3::Steinberg::Vst::ControllerNumbers_;
use vst3::Steinberg::Vst::{
	IMidiMapping, IMidiMappingTrait, IParamValueQueue, IParamValueQueueTrait, IParameterChanges,
	IParameterChangesTrait,
};
use vst3::Steinberg::{kResultOk, tresult};
use vst3::{Class, ComPtr, ComRef, ComWrapper};

// Channel 0 is reserved for master channel
pub const N_CHANNELS: usize = 15;

// We only send a single event per buffer
struct ParamValueQueue {
	id: u32,
	value: UnsafeCell<Option<f64>>,
}
unsafe impl Send for ParamValueQueue {}
unsafe impl Sync for ParamValueQueue {}

impl Class for ParamValueQueue {
	type Interfaces = (IParamValueQueue,);
}

impl IParamValueQueueTrait for ParamValueQueue {
	unsafe fn getParameterId(&self) -> u32 {
		self.id
	}
	unsafe fn getPointCount(&self) -> i32 {
		unsafe { if (*self.value.get()).is_some() { 1 } else { 0 } }
	}
	unsafe fn getPoint(&self, index: i32, sample_offset: *mut i32, value: *mut f64) -> tresult {
		unsafe {
			if index == 0 {
				if let Some(v) = { *self.value.get() } {
					*sample_offset = 0;
					*value = v;
				}
				kResultOk
			} else {
				vst3::Steinberg::kResultFalse
			}
		}
	}
	unsafe fn addPoint(&self, _offset: i32, _value: f64, _idx: *mut i32) -> tresult {
		// Not implemented
		vst3::Steinberg::kResultFalse
	}
}

// Safe wrapper around ParamValueQueue
struct Parameter {
	param: ComWrapper<ParamValueQueue>,
	com_ptr: ComPtr<IParamValueQueue>,
}

impl Parameter {
	fn new(id: u32) -> Self {
		let param = ComWrapper::new(ParamValueQueue { id, value: None.into() });
		let com_ptr = param.to_com_ptr::<IParamValueQueue>().unwrap();
		Self { param, com_ptr }
	}

	fn get(&self) -> Option<f64> {
		unsafe { *self.param.value.get() }
	}

	fn set(&self, value: f64) {
		unsafe { *self.param.value.get() = Some(value) }
	}

	fn clear(&self) {
		unsafe { *self.param.value.get() = None }
	}
}

struct ParameterChanges {
	parameters: Vec<Parameter>,
}

unsafe impl Send for ParameterChanges {}
unsafe impl Sync for ParameterChanges {}

impl Class for ParameterChanges {
	type Interfaces = (IParameterChanges,);
}

impl IParameterChangesTrait for ParameterChanges {
	unsafe fn getParameterCount(&self) -> i32 {
		self.parameters.len() as i32
	}
	unsafe fn getParameterData(&self, index: i32) -> *mut IParamValueQueue {
		let index = index as usize;

		if index < self.parameters.len() {
			let value = self.parameters[index].get();
			if value.is_some() {
				return self.parameters[index].com_ptr.as_ptr();
			}
		}
		std::ptr::null_mut()
	}
	unsafe fn addParameterData(&self, _id: *const u32, _index: *mut i32) -> *mut IParamValueQueue {
		// Not implemented
		std::ptr::null_mut()
	}
}

const RPN_LSB: usize = 32;
const RPN_MSB: usize = 33;
const DATA_MSB: usize = 34;

// Convenience wrapper for 16 channel midi event data
pub struct Parameters {
	changes: ComWrapper<ParameterChanges>,
	com_ptr: ComPtr<IParameterChanges>,
}

impl Parameters {
	pub fn new(midi_mapping: ComRef<IMidiMapping>) -> Result<Self, String> {
		let mut parameters = Vec::with_capacity(16);

		let mut add_channel = |id: u32| {
			let index = parameters.len();
			parameters.push(Parameter::new(id));
			index
		};

		// Query pitch bend for all 16 channels
		const PITCHBEND: i16 = ControllerNumbers_::kPitchBend as i16;
		for i in 0..16 {
			let mut id = 0;
			unsafe { midi_mapping.getMidiControllerAssignment(0, i as i16, PITCHBEND, &mut id) }
				.as_result()?;
			add_channel(id);
		}

		// Query aftertouch (channel pressure) for all 16 channels
		const AFTERTOUCH: i16 = ControllerNumbers_::kAfterTouch as i16;
		for i in 0..16 {
			let mut id = 0;
			unsafe { midi_mapping.getMidiControllerAssignment(0, i as i16, AFTERTOUCH, &mut id) }
				.as_result()?;
			add_channel(id);
		}

		// Query other parameters
		let param_list = [
			(ControllerNumbers_::kCtrlRPNSelectLSB, RPN_LSB),
			(ControllerNumbers_::kCtrlRPNSelectMSB, RPN_MSB),
			(ControllerNumbers_::kCtrlDataEntryMSB, DATA_MSB),
		];

		for (ctrl_id, index) in &param_list {
			let mut param_id = 0;
			unsafe {
				midi_mapping.getMidiControllerAssignment(0, 0, *ctrl_id as i16, &mut param_id)
			}
			.as_result()?;
			assert_eq!(add_channel(param_id), *index);
		}

		let changes = ComWrapper::new(ParameterChanges { parameters });
		let com_ptr = changes.to_com_ptr::<IParameterChanges>().unwrap();

		Ok(Self { changes, com_ptr })
	}

	pub fn mpe_init(&self) {
		// Send MPE message lower zone 15 channels
		// 176  100    6
		// 176  101    0
		// 176    6   15

		self.push(RPN_LSB, 6.0 / 127.0);
		self.push(RPN_MSB, 0.0 / 127.0);
		self.push(DATA_MSB, 15.0 / 127.0);
	}

	pub fn push_pitchend(&self, id: usize, value: f64) {
		assert!(id < N_CHANNELS);
		self.push(id + 1, value);
	}

	pub fn push_pressure(&self, id: usize, value: f64) {
		assert!(id < N_CHANNELS);
		self.push(id + 17, value);
	}

	pub fn push(&self, id: usize, value: f64) {
		self.changes.parameters[id].set(value);
	}

	pub fn clear(&self) {
		// clear all channels
		for parameter in &self.changes.parameters {
			parameter.clear();
		}
	}

	pub fn as_com_ptr(&self) -> *mut IParameterChanges {
		self.com_ptr.as_ptr()
	}
}
