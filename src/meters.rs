use crate::log_info;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

// A simple bump allocator for shared atomic floats

const PAGE_SIZE: usize = 64;

struct MeterPage {
	data: Vec<(AtomicU32, AtomicU32)>,
}

impl MeterPage {
	fn new() -> Self {
		let mut data = Vec::with_capacity(PAGE_SIZE);
		for _ in 0..(PAGE_SIZE) {
			data.push((AtomicU32::new(0), AtomicU32::new(0)));
		}
		Self { data }
	}
}

// handle held by the DSP object
#[derive(Clone)]
pub struct MeterHandle {
	page: Arc<MeterPage>,
	base_index: usize,
}

impl MeterHandle {
	pub fn set(&self, value: (f32, f32)) {
		let (l, r) = &self.page.data[self.base_index];
		l.store(value.0.to_bits(), Ordering::Relaxed);
		r.store(value.1.to_bits(), Ordering::Relaxed);
	}
}

pub struct Meters {
	pages: Vec<Arc<MeterPage>>,
	page_index: usize,
	slot_index: usize,
	global_slot_index: usize,
}

impl Meters {
	pub fn new() -> Self {
		Self {
			pages: vec![Arc::new(MeterPage::new())],
			page_index: 0,
			slot_index: 0,
			global_slot_index: 0,
		}
	}

	pub fn register(&mut self) -> (MeterHandle, usize) {
		if self.slot_index >= PAGE_SIZE {
			self.pages.push(Arc::new(MeterPage::new()));
			self.page_index += 1;
			self.slot_index = 0;
			log_info!("Allocating meter page {}", self.page_index);
		}
		let page = &self.pages[self.page_index];

		let handle = MeterHandle { page: page.clone(), base_index: self.slot_index };
		let index = self.global_slot_index;

		self.slot_index += 1;
		self.global_slot_index += 1;

		(handle, index)
	}

	// get a flat array for all active slots
	pub fn collect(&self) -> Vec<[f32; 2]> {
		let mut result = Vec::with_capacity(self.pages.len() * PAGE_SIZE);

		let (last_page, pages) = self.pages.split_last().unwrap();

		for page in pages {
			for val in &page.data {
				result.push(load_value(val));
			}
		}
		// only send filled portion of last page
		for val in &last_page.data[..self.slot_index] {
			result.push(load_value(val));
		}
		result
	}
}

fn load_value(val: &(AtomicU32, AtomicU32)) -> [f32; 2] {
	let (l, r) = val;
	let l = f32::from_bits(l.load(Ordering::Relaxed));
	let r = f32::from_bits(r.load(Ordering::Relaxed));
	[l, r]
}
