use assert_no_alloc::*;
use cpal::{
	BackendSpecificError, BufferSize, Device, SampleFormat, Stream, StreamConfig, StreamError,
	traits::{DeviceTrait, HostTrait, StreamTrait},
};
use no_denormals::no_denormals;
use parking_lot::Mutex;
use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd, HeapRb};
use std::error::Error;
use std::panic;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::context::LuaMessage;
use crate::dsp::env::AttackRelease;
use crate::log::{log_error, log_info, log_warn};
use crate::render::Render;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

pub static AUDIO_PANIC: AtomicBool = AtomicBool::new(false);

pub const MAX_BUF_SIZE: usize = 64;
pub const SPECTRUM_SIZE: usize = 4096;

pub fn check_architecture() -> Result<(), String> {
	#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
	if !is_x86_feature_detected!("avx2") {
		return Err("Your CPU is not supported! This release requires at least AVX2.".to_string());
	}
	Ok(())
}

pub fn get_device_and_config(
	host_name: &str,
	output_device_name: &str,
	buffer_size: Option<u32>,
) -> Result<(Device, StreamConfig, SampleFormat), Box<dyn Error>> {
	let output_device = find_output_device(host_name, output_device_name)?;
	let config = output_device.default_output_config()?;

	let mut stream_config: StreamConfig = config.clone().into();
	stream_config.channels = 2; // only allow stereo output
	stream_config.buffer_size = BufferSize::Fixed(buffer_size.unwrap_or(128));

	Ok((output_device, stream_config, config.sample_format()))
}

#[allow(clippy::type_complexity)]
pub fn build_stream(
	device: &Device,
	config: &StreamConfig,
	format: SampleFormat,
	render: Arc<Mutex<Render>>,
) -> Result<(Stream, HeapProd<bool>, HeapCons<bool>), Box<dyn Error>> {
	let (stream_tx, stream_rx) = HeapRb::<bool>::new(8).split();
	let (error_tx, error_rx) = HeapRb::<bool>::new(8).split();

	use SampleFormat::*;
	let stream = match format {
		F64 => build_stream_inner::<f64>(device, config, render, stream_rx, error_tx),
		F32 => build_stream_inner::<f32>(device, config, render, stream_rx, error_tx),
		I64 => build_stream_inner::<i64>(device, config, render, stream_rx, error_tx),
		U64 => build_stream_inner::<u64>(device, config, render, stream_rx, error_tx),
		I32 => build_stream_inner::<i32>(device, config, render, stream_rx, error_tx),
		U32 => build_stream_inner::<u32>(device, config, render, stream_rx, error_tx),
		I16 => build_stream_inner::<i16>(device, config, render, stream_rx, error_tx),
		U16 => build_stream_inner::<u16>(device, config, render, stream_rx, error_tx),
		f => panic!("Unsupported sample format '{f}'"),
	}?;

	// immediately start the stream
	stream.play()?;

	log_info!("Stream set up succesfully!");
	log_info!("Sample rate: {}", config.sample_rate.0);

	Ok((stream, stream_tx, error_rx))
}

pub fn build_stream_inner<T>(
	device: &Device,
	config: &StreamConfig,
	render: Arc<Mutex<Render>>,
	stream_rx: HeapCons<bool>,
	error_tx: HeapProd<bool>,
) -> Result<Stream, Box<dyn Error>>
where
	T: 'static + cpal::SizedSample + cpal::FromSample<f32>,
{
	let audio_closure = build_closure::<T>(stream_rx, render);

	let stream =
		device.build_output_stream(config, audio_closure, error_closure(error_tx), None)?;

	Ok(stream)
}

fn build_closure<T>(
	mut stream_rx: HeapCons<bool>,
	render: Arc<Mutex<Render>>,
) -> impl FnMut(&mut [T], &cpal::OutputCallbackInfo)
where
	T: cpal::Sample + cpal::FromSample<f32>,
{
	// Callback data
	let mut start = false;
	let mut is_rendering = false;
	let process_buffer = [[0.0f32; MAX_BUF_SIZE]; 2];
	let mut cpu_load = AttackRelease::new_direct(0.05, 0.01);

	move |cpal_buffer: &mut [T], _: &cpal::OutputCallbackInfo| {
		let result = panic::catch_unwind(AssertUnwindSafe(|| {
			assert_no_alloc(|| {
				#[cfg(debug_assertions)]
				enable_fpu_traps();

				let cpal_buffer_size = cpal_buffer.len() / 2;
				match render.try_lock() {
					Some(mut render) if !is_rendering => {
						if !start {
							start = true;
							log_info!("Buffer size: {cpal_buffer_size:?}");
						}

						let time = std::time::Instant::now();

						// parse all messages
						for m in stream_rx.pop_iter() {
							is_rendering = m;
						}
						render.parse_messages();

						for buffer_chunk in cpal_buffer.chunks_mut(MAX_BUF_SIZE) {
							let chunk_size = buffer_chunk.len() / 2;
							let [mut l, mut r] = process_buffer;
							let buf_slice = &mut [&mut l[..chunk_size], &mut r[..chunk_size]];

							unsafe {
								no_denormals(|| {
									render.process(buf_slice);
								});
							};

							// interlace and convert
							for (i, outsample) in buffer_chunk.chunks_exact_mut(2).enumerate() {
								outsample[0] = T::from_sample(buf_slice[0][i]);
								outsample[1] = T::from_sample(buf_slice[1][i]);
							}
						}

						let t = time.elapsed();
						let p = t.as_secs_f64()
							/ (cpal_buffer_size as f64 / f64::from(render.sample_rate));
						cpu_load.set(p as f32);
						let load = cpu_load.process();
						render.send(LuaMessage::Cpu { load });
					},
					_ => {
						// Output silence as a fallback when lock fails.

						for m in stream_rx.pop_iter() {
							is_rendering = m;
						}
						// log_info!("Output silent");
						for outsample in cpal_buffer.chunks_exact_mut(2) {
							outsample[0] = T::from_sample(0.0f32);
							outsample[1] = T::from_sample(0.0f32);
						}
					},
				}
			});
		}));
		if let Err(e) = result {
			let msg = match e.downcast_ref::<&'static str>() {
				Some(s) => *s,
				None => match e.downcast_ref::<String>() {
					Some(s) => &**s,
					None => "Box<Any>",
				},
			};
			log_error!("Audio thread panic: {msg}");

			AUDIO_PANIC.store(true, Ordering::Relaxed);

			for outsample in cpal_buffer.chunks_exact_mut(2) {
				outsample[0] = T::from_sample(0.0f32);
				outsample[1] = T::from_sample(0.0f32);
			}
		}
	}
}

fn error_closure(mut error_tx: HeapProd<bool>) -> impl FnMut(StreamError) + Send + 'static {
	move |error| match error {
		StreamError::DeviceNotAvailable => {
			log_error!("Stream error: device not available.");
		},
		StreamError::BackendSpecific { err: BackendSpecificError { ref description } } => {
			// TODO: hopefully future versions of CPAL will handle this
			if description.contains("ASIO reset request") {
				log_info!("ASIO reset request");
				error_tx.try_push(true).expect("Could not send message.");
			} else {
				log_error!("Stream error: {error}.");
			}
		},
	}
}

fn find_output_device(host_name: &str, output_device_name: &str) -> Result<Device, Box<dyn Error>> {
	let available_hosts = cpal::available_hosts();
	log_info!("Available hosts: {available_hosts:?}");

	let mut host = None;
	if host_name == "default" {
		host = Some(cpal::default_host());
	} else {
		for host_id in available_hosts {
			if host_id.name().to_lowercase().contains(&host_name.to_lowercase()) {
				host = Some(cpal::host_from_id(host_id)?);
				break;
			}
		}
	}
	let host = if let Some(h) = host {
		h
	} else {
		log_warn!("Couldn't find {host_name}. Using default instead");
		cpal::default_host()
	};

	log_info!("Using host: {}", host.id().name());

	log_info!("Avaliable output devices:");
	for d in host.output_devices()? {
		#[allow(deprecated)]
		let name = d.name()?;
		log_info!(" - \"{name}\"");
	}

	let mut output_device = None;

	if output_device_name == "default" {
		output_device = host.default_output_device();
	} else {
		for device in host.output_devices().map_err(|_| "No output devices found.")? {
			#[allow(deprecated)]
			if let Ok(name) = device.name()
				&& name.to_lowercase().contains(&output_device_name.to_lowercase())
			{
				output_device = Some(device);
			}
		}
	}

	let output_device = if let Some(d) = output_device {
		d
	} else {
		log_warn!("Couldn't find {output_device_name}. Using default instead");
		host.default_output_device()
			.ok_or("No default output device found.")?
	};

	#[allow(deprecated)]
	let name = output_device.name()?;
	log_info!("Using output device: \"{name}\"");

	Ok(output_device)
}

pub fn write_wav(filename: &str, samples: &[f32], sample_rate: u32) -> Result<(), Box<dyn Error>> {
	let spec = hound::WavSpec {
		channels: 2,
		sample_rate,
		bits_per_sample: 16,
		sample_format: hound::SampleFormat::Int,
	};

	let mut writer = hound::WavWriter::create(filename, spec)?;
	for s in samples {
		writer.write_sample(convert_sample_wav(*s))?;
	}
	writer.finalize()?;

	Ok(())
}

fn convert_sample_wav(x: f32) -> i16 {
	// TPDF dither in range [-1, 1] quantization levels
	let dither = (fastrand::f32() - fastrand::f32()) / f32::from(u16::MAX);
	let x = (x + dither).clamp(-1.0, 1.0);
	(if x >= 0.0 { x * f32::from(i16::MAX) } else { -x * f32::from(i16::MIN) }) as i16
}

#[cfg(debug_assertions)]
#[cfg(target_arch = "x86_64")]
#[allow(deprecated)]
fn enable_fpu_traps() {
	unsafe {
		use std::arch::x86_64::*;
		let mut mxcsr = _mm_getcsr();
		// clear the mask bits for all of the traps except _MM_EXCEPT_INEXACT.
		mxcsr &= !(_MM_MASK_INVALID
			| _MM_EXCEPT_DENORM
			| _MM_MASK_DIV_ZERO
			| _MM_EXCEPT_OVERFLOW
			| _MM_EXCEPT_UNDERFLOW);
		_mm_setcsr(mxcsr);
	}
}

#[cfg(debug_assertions)]
#[cfg(not(target_arch = "x86_64"))]
fn enable_fpu_traps() {}
