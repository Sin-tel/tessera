use std::cell::UnsafeCell;
use std::sync::Arc;
use vst3::Steinberg::Vst::{
	IParamValueQueue, IParamValueQueueTrait, IParameterChanges, IParameterChangesTrait,
};
use vst3::Steinberg::{kResultOk, tresult};
use vst3::{Class, ComPtr, ComWrapper};

pub const N_CHANNELS: usize = 16;

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
		N_CHANNELS as i32
	}
	unsafe fn getParameterData(&self, index: i32) -> *mut IParamValueQueue {
		let index = index as usize;
		let value = unsafe { *self.buffers[index].value.get() };

		if index < N_CHANNELS && value.is_some() {
			self.ptrs[index].as_ptr()
		} else {
			std::ptr::null_mut()
		}
	}
	unsafe fn addParameterData(&self, _id: *const u32, _index: *mut i32) -> *mut IParamValueQueue {
		std::ptr::null_mut()
	}
}

// Convenience wrapper for 16 channel midi event data
pub struct AutomationQueue {
	changes_obj: ComWrapper<ParameterChanges>,
	changes_ptr: ComPtr<IParameterChanges>,
}

impl AutomationQueue {
	pub fn new(ids: [u32; N_CHANNELS]) -> Self {
		let mut buffers = Vec::with_capacity(N_CHANNELS);
		let mut ptrs = Vec::with_capacity(N_CHANNELS);

		for i in 0..N_CHANNELS {
			let buffer = Arc::new(ParameterBuffer { id: ids[i], value: None.into() });

			let value_queue = ParamValueQueue { buffer: Arc::clone(&buffer) };
			let value_queue_ptr =
				ComWrapper::new(value_queue).to_com_ptr::<IParamValueQueue>().unwrap();

			buffers.push(Arc::clone(&buffer));
			ptrs.push(value_queue_ptr);
		}

		let changes = ParameterChanges { buffers, ptrs };
		let changes_obj = ComWrapper::new(changes);
		let changes_ptr = changes_obj.to_com_ptr::<IParameterChanges>().unwrap();

		Self { changes_obj, changes_ptr }
	}

	pub fn push(&self, channel: usize, normalized_value: f64) {
		unsafe {
			*self.changes_obj.buffers[channel].value.get() = Some(normalized_value);
		}
	}

	pub fn clear(&self) {
		// clear all channels
		unsafe {
			for channel in &self.changes_obj.buffers {
				*channel.value.get() = None;
			}
		}
	}

	pub fn as_com_ptr(&self) -> *mut IParameterChanges {
		self.changes_ptr.as_ptr()
	}
}
