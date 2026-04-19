use crate::vst3::error::ToResultExt;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
use vst3::Steinberg::IBStream_::IStreamSeekMode_;
use vst3::Steinberg::{
	IBStream, IBStreamTrait, kInvalidArgument, kResultFalse, kResultOk, tresult,
};
use vst3::{Class, ComPtr, ComWrapper};

struct Stream {
	buffer: Arc<Mutex<Cursor<Vec<u8>>>>,
}

impl Class for Stream {
	type Interfaces = (IBStream,);
}

impl IBStreamTrait for Stream {
	unsafe fn read(
		&self,
		buffer: *mut std::ffi::c_void,
		num_bytes: i32,
		num_bytes_read: *mut i32,
	) -> tresult {
		if buffer.is_null() {
			return kInvalidArgument;
		}

		let mut cursor = self.buffer.lock().unwrap();
		let slice =
			unsafe { std::slice::from_raw_parts_mut(buffer as *mut u8, num_bytes as usize) };

		match cursor.read(slice) {
			Ok(bytes_read) => {
				if !num_bytes_read.is_null() {
					unsafe { *num_bytes_read = bytes_read as i32 };
				}
				kResultOk
			},
			Err(_) => kResultFalse,
		}
	}

	unsafe fn write(
		&self,
		buffer: *mut std::ffi::c_void,
		num_bytes: i32,
		num_bytes_written: *mut i32,
	) -> tresult {
		if buffer.is_null() {
			return kInvalidArgument;
		}

		let mut cursor = self.buffer.lock().unwrap();
		let slice = unsafe { std::slice::from_raw_parts(buffer as *const u8, num_bytes as usize) };

		match cursor.write_all(slice) {
			Ok(()) => {
				if !num_bytes_written.is_null() {
					unsafe { *num_bytes_written = num_bytes };
				}
				kResultOk
			},
			Err(_) => kResultFalse,
		}
	}

	unsafe fn seek(&self, pos: i64, mode: i32, result: *mut i64) -> tresult {
		let mut cursor = self.buffer.lock().unwrap();

		let seek_from = match mode {
			IStreamSeekMode_::kIBSeekSet => SeekFrom::Start(pos as u64),
			IStreamSeekMode_::kIBSeekCur => SeekFrom::Current(pos),
			IStreamSeekMode_::kIBSeekEnd => SeekFrom::End(pos),
			_ => return kInvalidArgument,
		};

		match cursor.seek(seek_from) {
			Ok(new_pos) => {
				if !result.is_null() {
					unsafe { *result = new_pos as i64 };
				}
				kResultOk
			},
			Err(_) => kResultFalse,
		}
	}

	unsafe fn tell(&self, pos: *mut i64) -> tresult {
		if pos.is_null() {
			return kInvalidArgument;
		}
		let mut cursor = self.buffer.lock().unwrap();

		match cursor.stream_position() {
			Ok(current_pos) => {
				unsafe { *pos = current_pos as i64 };
				kResultOk
			},
			Err(_) => kResultFalse,
		}
	}
}

pub struct Vst3State {
	buffer: Arc<Mutex<Cursor<Vec<u8>>>>,
	stream: ComPtr<IBStream>,
}

impl Vst3State {
	fn from_bytes(bytes: Vec<u8>) -> Self {
		let buffer = Arc::new(Mutex::new(Cursor::new(bytes)));
		let mem_stream = Stream { buffer: Arc::clone(&buffer) };
		let stream_obj = ComWrapper::new(mem_stream);
		let stream = stream_obj.to_com_ptr::<IBStream>().unwrap();

		Self { buffer, stream }
	}

	pub fn new() -> Self {
		Self::from_bytes(Vec::new())
	}

	pub fn from_string(state_base64: String) -> Result<Self, String> {
		let bytes = STANDARD.decode(state_base64).map_err(|e| e.to_string())?;
		Ok(Self::from_bytes(bytes))
	}

	pub fn as_ptr(&self) -> *mut IBStream {
		self.stream.as_ptr()
	}

	pub fn rewind(&self) -> Result<(), String> {
		unsafe { self.stream.seek(0, 0, std::ptr::null_mut()) }.as_result()
	}

	pub fn into_string(self) -> String {
		let bytes = self.buffer.lock().unwrap();
		STANDARD.encode(bytes.get_ref())
	}
}
