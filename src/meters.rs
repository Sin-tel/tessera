use crate::dsp::atomic_float::AtomicFloat;
use crate::log_info;
use std::sync::Arc;

// A simple bump allocator for shared atomic floats

const PAGE_SIZE: usize = 64;

struct Page {
	data: [[AtomicFloat; 2]; PAGE_SIZE],
}

impl Page {
	fn new() -> Self {
		Self { data: std::array::from_fn(|_| [AtomicFloat::new(), AtomicFloat::new()]) }
	}
}

// handle held by the DSP object
#[derive(Clone)]
pub struct MeterHandle {
	page: Arc<Page>,
	slot_index: usize,
}

impl MeterHandle {
	pub fn set(&self, value: [f32; 2]) {
		let [l, r] = &self.page.data[self.slot_index];
		l.store(value[0]);
		r.store(value[1]);
	}
}

pub struct Meters {
	pages: Vec<Arc<Page>>,
	page_index: usize,
	slot_index: usize,
	global_index: usize,
}

impl Meters {
	pub fn new() -> Self {
		Self { pages: vec![Arc::new(Page::new())], page_index: 0, slot_index: 0, global_index: 0 }
	}

	pub fn register(&mut self) -> (MeterHandle, usize) {
		if self.slot_index >= PAGE_SIZE {
			self.pages.push(Arc::new(Page::new()));
			self.page_index += 1;
			self.slot_index = 0;
			log_info!("Allocating meter page {}", self.page_index);
		}
		let page = &self.pages[self.page_index];

		let handle = MeterHandle { page: page.clone(), slot_index: self.slot_index };
		let index = self.global_index;

		self.slot_index += 1;
		self.global_index += 1;

		(handle, index)
	}

	// get a flat array for all active slots
	// TODO: probably more efficient to send (Vec<f32>, Vec<f32>)
	pub fn collect(&self) -> Vec<[f32; 2]> {
		let mut result = Vec::with_capacity(self.global_index);

		let (last_page, pages) = self.pages.split_last().expect("There is at least one page.");

		for page in pages {
			for [l, r] in &page.data {
				result.push([l.load(), r.load()]);
			}
		}
		// only send filled portion of last page
		for [l, r] in &last_page.data[..self.slot_index] {
			result.push([l.load(), r.load()]);
		}
		result
	}
}
