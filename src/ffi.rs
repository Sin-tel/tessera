use std::ffi::{c_void, CStr};
use std::os::raw::c_char;

use rand::prelude::*;

use std::sync::{Arc, Mutex};

use ringbuf::{Consumer, Producer};

use crate::audio::*;
use crate::defs::*;
use crate::render::*;

// big struct that lua side holds ptr to
// dead_code is allowed because compiler doesn't know lua holds it
// #[allow(dead_code)]
pub struct Userdata {
	pub stream: cpal::Stream,
	pub audio_tx: Producer<AudioMessage>,
	pub stream_tx: Producer<bool>,
	pub lua_rx: Consumer<LuaMessage>,
	pub m_render: Arc<Mutex<Render>>,
}

#[no_mangle]
pub extern "C" fn stream_new(host_ptr: *const c_char, device_ptr: *const c_char) -> *mut c_void {
	let host_name = unsafe { CStr::from_ptr(host_ptr).to_str().unwrap() };
	// dbg!(host_name);
	let device_name = unsafe { CStr::from_ptr(device_ptr).to_str().unwrap() };
	// dbg!(device_name);

	Box::into_raw(Box::new(audio_run(host_name, device_name).unwrap())) as *mut c_void
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
pub extern "C" fn send_CV(stream_ptr: *mut c_void, ch: usize, freq: f32, vol: f32) {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	send_message(d, AudioMessage::CV(ch, CV { freq, vol }));
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
pub extern "C" fn add(stream_ptr: *mut c_void) {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	send_message(d, AudioMessage::Add);
}

#[inline]
fn dither(rng: &mut ThreadRng) -> f64 {
	// Don't know if this is the correct scaling for dithering, but it sounds good
	(rng.gen::<f64>() - rng.gen::<f64>()) / (2.0 * (i16::MAX as f64))
}

#[inline]
fn convert_sample_wav(rng: &mut ThreadRng, x: f32) -> f64 {
	let z = (x as f64) + dither(rng);
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

	let mut rng = thread_rng();

	let len = 64;

	// normal audio buffer
	let mut audiobuf = vec![StereoSample { l: 0.0, r: 0.0 }; len];

	// audiobuffer to send to lua side
	let mut caudiobuf = vec![0.0f64; len * 2];

	render.parse_messages();
	render.process(&mut audiobuf);

	// interlace and convert to i16 as f64 (lua wants doubles anyway)
	for (outsample, gensample) in caudiobuf.chunks_exact_mut(2).zip(audiobuf.iter()) {
		outsample[0] = convert_sample_wav(&mut rng, gensample.l);
		outsample[1] = convert_sample_wav(&mut rng, gensample.r);
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
	if !d.audio_tx.is_full() {
		d.audio_tx.push(m).unwrap();
	} else {
		println!("Queue full. Dropped message!")
	}
}

fn send_paused(d: &mut Userdata, paused: bool) {
	if !d.stream_tx.is_full() {
		d.stream_tx.push(paused).unwrap();
	} else {
		println!("Stream queue full. Dropped message!")
	}
}

#[no_mangle]
pub extern "C" fn rx_is_empty(stream_ptr: *mut c_void) -> bool {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	d.lua_rx.is_empty()
}

#[no_mangle]
pub extern "C" fn rx_pop(stream_ptr: *mut c_void) -> f32 {
	let d = unsafe { &mut *(stream_ptr as *mut Userdata) };

	match d.lua_rx.pop().unwrap() {
		LuaMessage::Test(v) => v,
	}
}
