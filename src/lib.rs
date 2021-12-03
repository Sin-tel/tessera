// extern crate anyhow;
// extern crate cpal;
// extern crate rand;
// extern crate ringbuf;

use rand::prelude::*;

use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{Consumer, Producer, RingBuffer};

use assert_no_alloc::*;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

mod defs;
use defs::*;

mod message;
use message::*;

mod render;
use render::*;

mod instrument;

// big struct that lua side holds ptr to
// dead_code is allowed because compiler doesn't know lua holds it
#[allow(dead_code)]
pub struct Userdata {
	stream: cpal::Stream,
	prod: Producer<Message>,
	prod_stream: Producer<bool>,
	m_render: Arc<Mutex<Render>>,
}

#[no_mangle]
pub extern "C" fn stream_new() -> *mut c_void {
	Box::into_raw(Box::new(audio_run().unwrap())) as *mut c_void
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

	send_message(d, Message::CV(ch, CV { freq, vol }));
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

	send_message(d, Message::Add);
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

fn send_message(d: &mut Userdata, m: Message) {
	if !d.prod.is_full() {
		d.prod.push(m).unwrap();
	} else {
		println!("Queue full. Dropped message!")
	}
}

fn send_paused(d: &mut Userdata, paused: bool) {
	if !d.prod_stream.is_full() {
		d.prod_stream.push(paused).unwrap();
	} else {
		println!("Stream queue full. Dropped message!")
	}
}

fn audio_run() -> Result<Userdata, anyhow::Error> {
	let output_device = "default";
	let buf_size = 64;

	// force ASIO for now
	let host;
	#[cfg(target_os = "windows")]
	{
		host = cpal::host_from_id(cpal::HostId::Asio).expect("failed to initialise ASIO host");
	};

	// let host = cpal::default_host(); // wasapi

	dbg!(host.id());

	let output_device = if output_device == "default" {
		host.default_output_device()
	} else {
		host.output_devices()?
			.find(|x| x.name().map(|y| y == output_device).unwrap_or(false))
	}
	.expect("failed to find output device");

	for d in host.devices()? {
		println!("{}", d.name()?);
	}

	println!("Using output device: \"{}\"", output_device.name()?);

	let config = output_device.default_output_config()?;
	let mut config2: cpal::StreamConfig = config.clone().into();
	config2.channels = 2; // only allow stereo output

	config2.buffer_size = cpal::BufferSize::Fixed(buf_size);
	// config2.buffer_size = cpal::BufferSize::Default; // wasapi

	// Build streams.
	println!("{:?}", config);
	println!("{:?}", config2);

	let userdata = match config.sample_format() {
		cpal::SampleFormat::F32 => build_stream::<f32>(&output_device, &config2),
		cpal::SampleFormat::I16 => build_stream::<i16>(&output_device, &config2),
		cpal::SampleFormat::U16 => build_stream::<u16>(&output_device, &config2),
	};

	userdata.stream.play()?;

	println!("Stream set up succesfully!");
	Ok(userdata)
}

pub fn build_stream<T: 'static>(device: &cpal::Device, config: &cpal::StreamConfig) -> Userdata
where
	T: cpal::Sample,
{
	let rb = RingBuffer::<Message>::new(256);
	let (prod, cons) = rb.split();

	let rb = RingBuffer::<bool>::new(8);
	let (prod_stream, cons_stream) = rb.split();

	let buf_size: usize = match config.buffer_size {
		cpal::BufferSize::Fixed(framecount) => framecount.try_into().unwrap(),
		cpal::BufferSize::Default => panic!("Don't know buffer size!"),
	};
	// let buf_size = 448; // wasapi

	let sample_rate = config.sample_rate.0 as f32;

	let m_render = Arc::new(Mutex::new(Render::new(sample_rate, buf_size, cons)));
	let m_render_clone = Arc::clone(&m_render);

	let audio_closure = build_closure::<T>(cons_stream, m_render_clone, buf_size);

	let stream = device
		.build_output_stream(config, audio_closure, err_fn)
		.unwrap();

	Userdata {
		stream,
		prod,
		prod_stream,
		m_render,
	}
}

fn build_closure<T>(
	mut cons_stream: Consumer<bool>,
	m_render: Arc<Mutex<Render>>,
	buf_size: usize,
) -> impl FnMut(&mut [T], &cpal::OutputCallbackInfo)
where
	T: cpal::Sample,
{
	// Callback data
	let mut start = false;

	let mut paused = false;

	let mut audiobuf: Vec<StereoSample> = vec![StereoSample { l: 0.0, r: 0.0 }; buf_size];

	move |buffer: &mut [T], _: &cpal::OutputCallbackInfo| {
		assert_no_alloc(|| {
			// dbg!(buffer.len());
			assert!(buf_size * 2 == buffer.len());

			let time = std::time::Instant::now();

			let opt_render = m_render.try_lock();

			match opt_render {
				Ok(mut render) if !paused => {
					if !start {
						start = true;
						fix_denorms();
						render.add();
					}

					// parse all messages
					cons_stream.pop_each(
						|m| {
							paused = m;
							true
						},
						None,
					);
					render.parse_messages();
					// write to output buffer
					render.process(&mut audiobuf);

					for (outsample, gensample) in buffer.chunks_exact_mut(2).zip(audiobuf.iter()) {
						outsample[0] = cpal::Sample::from::<f32>(&gensample.l);
						outsample[1] = cpal::Sample::from::<f32>(&gensample.r);
					}
				}
				_ => {
					paused = true;
					cons_stream.pop_each(
						|m| {
							paused = m;
							true
						},
						None,
					);

					for outsample in buffer.chunks_exact_mut(2) {
						outsample[0] = cpal::Sample::from::<f32>(&0.0f32);
						outsample[1] = cpal::Sample::from::<f32>(&0.0f32);
					}
				}
			}

			// dbg!(buf_size)
			let t = time.elapsed();
			// println!("Cpu load {:.2}%", 100.0*t.as_secs_f64() / buf_time);
			// println!("{:?}", t);
			// dbg!(t);
		});
	}
}

fn err_fn(err: cpal::StreamError) {
	eprintln!("an error occurred on stream: {}", err);
}

#[inline]
fn fix_denorms() {
	unsafe {
		use std::arch::x86_64::*;

		let mut mxcsr = _mm_getcsr();

		// Denormals & underflows are flushed to zero
		mxcsr |= (1 << 15) | (1 << 6);

		// All exceptions are masked
		mxcsr |= ((1 << 6) - 1) << 7;

		_mm_setcsr(mxcsr);
	}
}
