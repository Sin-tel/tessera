use crate::vst3::error::ToResultExt;
use std::cell::UnsafeCell;
use std::sync::Arc;
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
struct ParameterBuffer {
	id: u32,
	value: UnsafeCell<Option<f64>>,
}
unsafe impl Send for ParameterBuffer {}
unsafe impl Sync for ParameterBuffer {}

struct ParamValueQueue {
	buffer: Arc<ParameterBuffer>,
}

impl Class for ParamValueQueue {
	type Interfaces = (IParamValueQueue,);
}

impl IParamValueQueueTrait for ParamValueQueue {
	unsafe fn getParameterId(&self) -> u32 {
		self.buffer.id
	}
	unsafe fn getPointCount(&self) -> i32 {
		unsafe { if (*self.buffer.value.get()).is_some() { 1 } else { 0 } }
	}
	unsafe fn getPoint(&self, index: i32, sample_offset: *mut i32, value: *mut f64) -> tresult {
		unsafe {
			if index == 0 {
				if let Some(v) = { *self.buffer.value.get() } {
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

struct ParameterChanges {
	buffers: Vec<Arc<ParameterBuffer>>,
	ptrs: Vec<ComPtr<IParamValueQueue>>,
}

unsafe impl Send for ParameterChanges {}
unsafe impl Sync for ParameterChanges {}

impl Class for ParameterChanges {
	type Interfaces = (IParameterChanges,);
}

impl IParameterChangesTrait for ParameterChanges {
	unsafe fn getParameterCount(&self) -> i32 {
		self.ptrs.len() as i32
	}
	unsafe fn getParameterData(&self, index: i32) -> *mut IParamValueQueue {
		let index = index as usize;

		if index < self.ptrs.len() {
			let value = unsafe { *self.buffers[index].value.get() };
			if value.is_some() {
				return self.ptrs[index].as_ptr();
			}
		}
		std::ptr::null_mut()
	}
	unsafe fn addParameterData(&self, _id: *const u32, _index: *mut i32) -> *mut IParamValueQueue {
		std::ptr::null_mut()
	}
}

const RPN_LSB: usize = 32;
const RPN_MSB: usize = 33;
const DATA_MSB: usize = 34;

// Convenience wrapper for 16 channel midi event data
pub struct AutomationQueue {
	changes_obj: ComWrapper<ParameterChanges>,
	changes_ptr: ComPtr<IParameterChanges>,
}

impl AutomationQueue {
	pub fn new(midi_mapping: ComRef<IMidiMapping>) -> Result<Self, String> {
		let mut buffers = Vec::with_capacity(16);
		let mut ptrs = Vec::with_capacity(16);

		let mut add_channel = |id: u32| {
			let buffer = Arc::new(ParameterBuffer { id, value: None.into() });

			let value_queue = ParamValueQueue { buffer: Arc::clone(&buffer) };
			let value_queue_ptr =
				ComWrapper::new(value_queue).to_com_ptr::<IParamValueQueue>().unwrap();

			let index = buffers.len();

			buffers.push(buffer);
			ptrs.push(value_queue_ptr);
			index
		};

		// Query pitch bend for all 16 channels
		for i in 0..16 {
			let mut id = 0;
			unsafe {
				midi_mapping.getMidiControllerAssignment(
					0,
					i as i16,
					ControllerNumbers_::kPitchBend as i16,
					&mut id,
				)
			}
			.as_result()?;
			add_channel(id);
		}

		// Query channel pressure for all 16 channels
		for i in 0..16 {
			let mut id = 0;
			unsafe {
				midi_mapping.getMidiControllerAssignment(
					0,
					i as i16,
					ControllerNumbers_::kAfterTouch as i16,
					&mut id,
				)
			}
			.as_result()?;
			add_channel(id);
		}

		// Query other parameters
		let params = [
			(ControllerNumbers_::kCtrlRPNSelectLSB, RPN_LSB),
			(ControllerNumbers_::kCtrlRPNSelectMSB, RPN_MSB),
			(ControllerNumbers_::kCtrlDataEntryMSB, DATA_MSB),
		];

		for (ctrl_id, index) in &params {
			let mut param_id = 0;
			unsafe {
				midi_mapping.getMidiControllerAssignment(0, 0, *ctrl_id as i16, &mut param_id)
			}
			.as_result()?;
			assert_eq!(add_channel(param_id), *index);
		}

		let changes = ParameterChanges { buffers, ptrs };
		let changes_obj = ComWrapper::new(changes);
		let changes_ptr = changes_obj.to_com_ptr::<IParameterChanges>().unwrap();

		Ok(Self { changes_obj, changes_ptr })
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

	pub fn push_pitchend(&self, id: usize, normalized_value: f64) {
		assert!(id < N_CHANNELS);
		self.push(id + 1, normalized_value);
	}

	pub fn push_pressure(&self, id: usize, normalized_value: f64) {
		assert!(id < N_CHANNELS);
		self.push(id + 17, normalized_value);
	}

	pub fn push(&self, id: usize, normalized_value: f64) {
		unsafe {
			*self.changes_obj.buffers[id].value.get() = Some(normalized_value);
		}
	}

	pub fn clear(&self) {
		// clear all channels
		unsafe {
			for buffer in &self.changes_obj.buffers {
				*buffer.value.get() = None;
			}
		}
	}

	pub fn as_com_ptr(&self) -> *mut IParameterChanges {
		self.changes_ptr.as_ptr()
	}
}
