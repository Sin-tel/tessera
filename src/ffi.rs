use std::ffi::{c_void, CStr};
use std::os::raw::c_char;

use std::sync::{Arc, Mutex};

use ringbuf::{Consumer, Producer};

use crate::audio::*;
use crate::defs::*;
use crate::render::*;

// big struct that lua side holds ptr to
pub struct Userdata {
	pub stream: cpal::Stream,
	pub audio_tx: Producer<AudioMessage>,
	pub stream_tx: Producer<bool>,
	pub lua_rx: Consumer<LuaMessage>,
	pub m_render: Arc<Mutex<Render>>,
}

/// # Safety
///
/// Make sure the arguments point to valid null-terminated c strings.
#[no_mangle]
pub unsafe extern "C" fn stream_new(
	host_ptr: *const c_char,
	device_ptr: *const c_char,
) -> *mut c_void {
	let host_name = CStr::from_ptr(host_ptr).to_str().unwrap();
	let device_name = CStr::from_ptr(device_ptr).to_str().unwrap();

	match audio_run(host_name, device_name) {
		Ok(ud) => Box::into_raw(Box::new(ud)) as *mut c_void,
		Err(_) => std::ptr::null_mut() as *mut c_void,
	}
}

#[no_mangle]
pub extern "C" fn stream_free(stream_ptr: *mut c_void) {
	unsafe {
		let d = Box::from_raw(stream_ptr as *mut Userdata);
		drop(d);
	}
	println!("Cleaned up stream!");
}

#[no_mangle]
pub extern "C" fn send_CV(stream_ptr: *mut c_void, ch: usize, pitch: f32, vel: f32) {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	send_message(d, AudioMessage::CV(ch, pitch, vel));
}

#[no_mangle]
pub extern "C" fn send_noteOn(stream_ptr: *mut c_void, ch: usize, pitch: f32, vel: f32, id: usize) {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	send_message(d, AudioMessage::Note(ch, pitch, vel, id));
}

#[no_mangle]
pub extern "C" fn send_pan(stream_ptr: *mut c_void, ch: usize, gain: f32, pan: f32) {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	send_message(d, AudioMessage::Pan(ch, gain, pan));
}

#[no_mangle]
pub extern "C" fn send_mute(stream_ptr: *mut c_void, ch: usize, mute: bool) {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	send_message(d, AudioMessage::Mute(ch, mute));
}

#[no_mangle]
pub extern "C" fn send_param(
	stream_ptr: *mut c_void,
	ch_index: usize,
	device_index: usize,
	index: usize,
	value: f32,
) {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	send_message(
		d,
		AudioMessage::SetParam(ch_index, device_index, index, value),
	);
}

#[no_mangle]
pub extern "C" fn play(stream_ptr: *mut c_void) {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	send_paused(d, false);
}

#[no_mangle]
pub extern "C" fn pause(stream_ptr: *mut c_void) {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	send_paused(d, true);
}

#[no_mangle]
pub extern "C" fn add_channel(stream_ptr: *mut c_void, instrument_index: usize) {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	// Should never fail
	let mut render = d.m_render.lock().expect("Failed to get lock.");

	render.add_channel(instrument_index);
}

#[inline]
fn dither() -> f64 {
	// Don't know if this is the correct scaling for dithering, but it sounds good
	(fastrand::f64() - fastrand::f64()) / (2.0 * (i16::MAX as f64))
}

#[inline]
fn convert_sample_wav(x: f32) -> f64 {
	let z = ((x as f64) + dither()).clamp(-1.0, 1.0);

	if z >= 0.0 {
		z * (i16::MAX as f64)
	} else {
		-z * (i16::MIN as f64)
	}
}

#[no_mangle]
pub extern "C" fn render_block(stream_ptr: *mut c_void) -> C_AudioBuffer {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	// should never fail!
	let mut render = d.m_render.lock().expect("Failed to get lock.");

	let len = 64;

	// normal audio buffer
	let mut audiobuf = vec![StereoSample { l: 0.0, r: 0.0 }; len];

	// audiobuffer to send to lua side
	let mut caudiobuf = vec![0.0f64; len * 2];

	render.parse_messages();
	render.process(&mut audiobuf);

	// interlace and convert to i16 as f64 (lua wants doubles anyway)
	for (outsample, gensample) in caudiobuf.chunks_exact_mut(2).zip(audiobuf.iter()) {
		outsample[0] = convert_sample_wav(gensample.l);
		outsample[1] = convert_sample_wav(gensample.r);
	}

	// TODO: replace this by into_raw_parts() when it is in stable
	let ptr = caudiobuf.as_mut_ptr() as *mut f64;
	let len = caudiobuf.len();
	let cap = caudiobuf.capacity();
	std::mem::forget(caudiobuf); // dont drop it

	// build struct that has all the necessary information to call Vec::from_raw_parts
	C_AudioBuffer { ptr, len, cap }
}

#[no_mangle]
pub extern "C" fn block_free(block: C_AudioBuffer) {
	unsafe {
		let d = Vec::from_raw_parts(block.ptr, block.len, block.cap);
		drop(d);
	}
	// println!("Cleaned up block!");
}

fn send_message(d: &mut Userdata, m: AudioMessage) {
	match d.audio_tx.push(m) {
		Ok(()) => (),
		Err(_) => println!("Queue full. Dropped message!"),
	}
}

fn send_paused(d: &mut Userdata, paused: bool) {
	match d.stream_tx.push(paused) {
		Ok(()) => (),
		Err(_) => println!("Stream queue full. Dropped message!"),
	}
}

#[no_mangle]
pub extern "C" fn rx_is_empty(stream_ptr: *mut c_void) -> bool {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	d.lua_rx.is_empty()
}

#[no_mangle]
pub extern "C" fn rx_pop(stream_ptr: *mut c_void) -> LuaMessage {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	d.lua_rx.pop().unwrap() // caller should make sure its not empty
}
