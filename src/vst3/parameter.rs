use std::cell::UnsafeCell;
use std::sync::Arc;
use vst3::Steinberg::Vst::{
	IParamValueQueue, IParamValueQueueTrait, IParameterChanges, IParameterChangesTrait,
};
use vst3::Steinberg::{kResultOk, tresult};
use vst3::{Class, ComPtr, ComWrapper};

pub const N_CHANNELS: usize = 16;

struct ParameterBuffer {
	id: u32,
	points: UnsafeCell<Vec<(i32, f64)>>, // (sample_offset, normalized_value)
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
		unsafe { (*self.buffer.points.get()).len() as i32 }
	}
	unsafe fn getPoint(&self, index: i32, sample_offset: *mut i32, value: *mut f64) -> tresult {
		unsafe {
			let pts = &*self.buffer.points.get();
			if index >= 0 && (index as usize) < pts.len() {
				if !sample_offset.is_null() {
					*sample_offset = pts[index as usize].0;
				}
				if !value.is_null() {
					*value = pts[index as usize].1;
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
		unsafe {
			if index < self.ptrs.len() && !(*self.buffers[index].points.get()).is_empty() {
				self.ptrs[index].as_ptr()
			} else {
				std::ptr::null_mut()
			}
		}
	}
	// Note: Depending on your vst3-rs version, _id might be `&u32`, `u32`, or `*const u32`. Adjust as needed!
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
			let queue = Arc::new(ParameterBuffer {
				id: ids[i],
				points: UnsafeCell::new(Vec::with_capacity(32)),
			});

			let value_queue = ParamValueQueue { buffer: Arc::clone(&queue) };
			let value_queue_obj = ComWrapper::new(value_queue);
			let value_queue_ptr = value_queue_obj.to_com_ptr::<IParamValueQueue>().unwrap();

			buffers.push(Arc::clone(&queue));
			ptrs.push(value_queue_ptr);
		}

		let changes = ParameterChanges { buffers, ptrs };
		let changes_obj = ComWrapper::new(changes);
		let changes_ptr = changes_obj.to_com_ptr::<IParameterChanges>().unwrap();

		Self { changes_obj, changes_ptr }
	}

	pub fn push(&self, channel: usize, sample_offset: i32, normalized_value: f64) {
		unsafe {
			(*self.changes_obj.buffers[channel].points.get())
				.push((sample_offset, normalized_value));
		}
	}

	pub fn clear(&self) {
		// clear all channels
		for channel in &self.changes_obj.buffers {
			unsafe {
				(*channel.points.get()).clear();
			}
		}
	}

	pub fn as_com_ptr(&self) -> *mut IParameterChanges {
		self.changes_ptr.as_ptr()
	}
}
