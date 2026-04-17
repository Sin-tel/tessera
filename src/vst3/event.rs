use std::cell::UnsafeCell;
use std::sync::Arc;
use vst3::Steinberg::Vst::Event_::EventTypes_;
use vst3::Steinberg::Vst::NoteOffEvent;
use vst3::Steinberg::Vst::{Event, Event__type0, IEventList, IEventListTrait, NoteOnEvent};
use vst3::Steinberg::{kResultOk, tresult};
use vst3::{Class, ComPtr, ComWrapper};

// capacity doesn't need to be very big since buffers are tiny
const CAPACITY: usize = 32;

struct EventBuffer(UnsafeCell<Vec<Event>>);

unsafe impl Send for EventBuffer {}
unsafe impl Sync for EventBuffer {}

struct EventList {
	buffer: Arc<EventBuffer>,
}

impl Class for EventList {
	type Interfaces = (IEventList,);
}

impl IEventListTrait for EventList {
	unsafe fn getEventCount(&self) -> i32 {
		unsafe { (*self.buffer.0.get()).len() as i32 }
	}

	unsafe fn getEvent(&self, index: i32, event: *mut Event) -> tresult {
		unsafe {
			let vec = &*self.buffer.0.get();
			if index >= 0 && (index as usize) < vec.len() {
				*event = vec[index as usize];
				kResultOk
			} else {
				vst3::Steinberg::kResultFalse
			}
		}
	}

	unsafe fn addEvent(&self, _event: *mut Event) -> tresult {
		// Not implemented because plugins don't push input events to the host
		kResultOk
	}
}

pub struct EventQueue {
	buffer: Arc<EventBuffer>,
	com_ptr: ComPtr<IEventList>,
}

impl EventQueue {
	pub fn new() -> Self {
		let buffer = Arc::new(EventBuffer(UnsafeCell::new(Vec::with_capacity(CAPACITY))));

		let com_obj = ComWrapper::new(EventList { buffer: Arc::clone(&buffer) });
		let com_ptr = com_obj.to_com_ptr::<IEventList>().unwrap();

		Self { buffer, com_ptr }
	}

	pub fn clear(&mut self) {
		unsafe { (*self.buffer.0.get()).clear() };
	}

	pub fn push(&mut self, event: Event) {
		// Note: may still allocate if we exceed capacity.
		unsafe { (*self.buffer.0.get()).push(event) };
	}

	pub fn as_com_ptr(&self) -> *mut IEventList {
		self.com_ptr.as_ptr()
	}
}

pub fn note_on(id: usize, pitch: i16, velocity: f32) -> Event {
	let channel = (id + 1) as i16;
	Event {
		busIndex: 0,
		sampleOffset: 0,
		ppqPosition: 0.0,
		flags: 0,
		r#type: EventTypes_::kNoteOnEvent as u16,
		__field0: Event__type0 {
			noteOn: NoteOnEvent { channel, pitch, tuning: 0., velocity, length: 0, noteId: -1 },
		},
	}
}

pub fn note_off(id: usize, pitch: i16) -> Event {
	let channel = (id + 1) as i16;
	Event {
		busIndex: 0,
		sampleOffset: 0,
		ppqPosition: 0.0,
		flags: 0,
		r#type: EventTypes_::kNoteOffEvent as u16,
		__field0: Event__type0 {
			noteOff: NoteOffEvent { channel, pitch, velocity: 0.0, tuning: 0.0, noteId: -1 },
		},
	}
}
