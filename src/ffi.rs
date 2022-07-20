use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use std::ptr::null_mut;

use std::sync::{Arc, Mutex};

use ringbuf::{Consumer, Producer};

use crate::audio;
use crate::defs::*;
use crate::render;

// big struct that lua side holds ptr to
pub struct Userdata {
	pub stream: cpal::Stream,
	pub audio_tx: Producer<AudioMessage>,
	pub stream_tx: Producer<bool>,
	pub lua_rx: Consumer<LuaMessage>,
	pub m_render: Arc<Mutex<render::Render>>,
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

	match audio::run(host_name, device_name) {
		Ok(ud) => Box::into_raw(Box::new(ud)).cast::<c_void>(),
		Err(_) => null_mut::<Userdata>().cast::<c_void>(),
	}
}

#[no_mangle]
pub extern "C" fn stream_free(stream_ptr: *mut c_void) {
	unsafe {
		let ud = Box::from_raw(stream_ptr.cast::<Userdata>());
		drop(ud);
	}
	println!("Cleaned up stream!");
}

#[no_mangle]
pub extern "C" fn send_CV(stream_ptr: *mut c_void, ch: usize, pitch: f32, vel: f32) {
	let ud = unsafe { &mut *stream_ptr.cast::<Userdata>() };

	send_message(ud, AudioMessage::CV(ch, pitch, vel));
}

#[no_mangle]
pub extern "C" fn send_noteOn(stream_ptr: *mut c_void, ch: usize, pitch: f32, vel: f32, id: usize) {
	let ud = unsafe { &mut *stream_ptr.cast::<Userdata>() };

	send_message(ud, AudioMessage::Note(ch, pitch, vel, id));
}

#[no_mangle]
pub extern "C" fn send_pan(stream_ptr: *mut c_void, ch: usize, gain: f32, pan: f32) {
	let ud = unsafe { &mut *stream_ptr.cast::<Userdata>() };

	send_message(ud, AudioMessage::Pan(ch, gain, pan));
}

#[no_mangle]
pub extern "C" fn send_mute(stream_ptr: *mut c_void, ch: usize, mute: bool) {
	let ud = unsafe { &mut *stream_ptr.cast::<Userdata>() };

	send_message(ud, AudioMessage::Mute(ch, mute));
}

#[no_mangle]
pub extern "C" fn send_param(
	stream_ptr: *mut c_void,
	ch_index: usize,
	device_index: usize,
	index: usize,
	value: f32,
) {
	let ud = unsafe { &mut *stream_ptr.cast::<Userdata>() };

	send_message(
		ud,
		AudioMessage::SetParam(ch_index, device_index, index, value),
	);
}

#[no_mangle]
pub extern "C" fn play(stream_ptr: *mut c_void) {
	let ud = unsafe { &mut *stream_ptr.cast::<Userdata>() };

	send_paused(ud, false);
}

#[no_mangle]
pub extern "C" fn pause(stream_ptr: *mut c_void) {
	let ud = unsafe { &mut *stream_ptr.cast::<Userdata>() };

	send_paused(ud, true);
}

#[no_mangle]
pub extern "C" fn add_channel(stream_ptr: *mut c_void, instrument_index: usize) {
	let ud = unsafe { &mut *stream_ptr.cast::<Userdata>() };

	// Should never fail
	let mut render = ud.m_render.lock().expect("Failed to get lock.");

	render.add_channel(instrument_index);
}

#[inline]
fn dither() -> f64 {
	// Don't know if this is the correct scaling for dithering, but it sounds good
	(fastrand::f64() - fastrand::f64()) / (2.0 * f64::from(i16::MAX))
}

#[inline]
fn convert_sample_wav(x: f32) -> f64 {
	let z = (f64::from(x) + dither()).clamp(-1.0, 1.0);

	if z >= 0.0 {
		z * f64::from(i16::MAX)
	} else {
		-z * f64::from(i16::MIN)
	}
}

#[no_mangle]
pub extern "C" fn render_block(stream_ptr: *mut c_void) -> C_AudioBuffer {
	let ud = unsafe { &mut *stream_ptr.cast::<Userdata>() };

	// should never fail!
	let mut render = ud.m_render.lock().expect("Failed to get lock.");

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

	// @todo: replace this by into_raw_parts() when it is in stable
	let ptr = caudiobuf.as_mut_ptr();
	let len = caudiobuf.len();
	let cap = caudiobuf.capacity();
	// don't drop it
	std::mem::forget(caudiobuf);

	// build struct that has all the necessary information to call Vec::from_raw_parts
	C_AudioBuffer { ptr, len, cap }
}

#[no_mangle]
pub extern "C" fn block_free(block: C_AudioBuffer) {
	unsafe {
		let caudiobuf = Vec::from_raw_parts(block.ptr, block.len, block.cap);
		drop(caudiobuf);
	}
	// println!("Cleaned up block!");
}

fn send_message(ud: &mut Userdata, m: AudioMessage) {
	if ud.audio_tx.push(m).is_err() {
		println!("Queue full. Dropped message!");
	}
}

fn send_paused(ud: &mut Userdata, paused: bool) {
	if ud.stream_tx.push(paused).is_err() {
		println!("Stream queue full. Dropped message!");
	}
}

#[no_mangle]
pub extern "C" fn rx_is_empty(stream_ptr: *mut c_void) -> bool {
	let ud = unsafe { &mut *stream_ptr.cast::<Userdata>() };
	ud.lua_rx.is_empty()
}

#[no_mangle]
pub extern "C" fn rx_pop(stream_ptr: *mut c_void) -> LuaMessage {
	let ud = unsafe { &mut *stream_ptr.cast::<Userdata>() };
	// caller should make sure its not empty
	ud.lua_rx.pop().unwrap()
}
