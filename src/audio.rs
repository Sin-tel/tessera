use assert_no_alloc::*;
use cpal::{
	BackendSpecificError, BufferSize, Device, HostId, SampleFormat, Stream, StreamConfig,
	StreamError, SupportedBufferSize, SupportedStreamConfigRange,
	traits::{DeviceTrait, HostTrait, StreamTrait},
};
use no_denormals::no_denormals;
use parking_lot::Mutex;
use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd, HeapRb};
use std::cmp;
use std::error::Error;
use std::panic;
use std::panic::AssertUnwindSafe;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;

use crate::context::{ErrorMessage, LuaMessage};
use crate::dsp::env::AttackRelease;
use crate::log::*;
use crate::render::Render;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

pub static AUDIO_PANIC: AtomicBool = AtomicBool::new(false);

pub const MAX_BUF_SIZE: usize = 64;
pub const SPECTRUM_SIZE: usize = 4096;

pub fn check_architecture() -> Result<(), String> {
	// not enabled for now

	// #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
	// if !is_x86_feature_detected!("avx2") {
	// 	return Err("Your CPU is not supported! This release requires at least AVX2.".to_string());
	// }
	Ok(())
}

// Get the string representation for HostId for available hosts
pub fn get_hosts() -> Vec<String> {
	cpal::available_hosts()
		.into_iter()
		.map(|host| host.to_string())
		.collect()
}

// Get the string representation for HostId for default host
pub fn get_default_host() -> String {
	cpal::default_host().id().to_string()
}

pub fn get_output_devices(host_str: &str) -> Result<Vec<String>, Box<dyn Error>> {
	let host_id = HostId::from_str(host_str)?;
	let host = cpal::host_from_id(host_id)?;

	let mut devices = Vec::new();

	for d in host.output_devices()? {
		match d.description() {
			Ok(d) => {
				let name = d.name();
				// TODO: FL Studio ASIO is broken
				if name != "FL Studio ASIO" {
					devices.push(name.to_string())
				}
			},
			Err(e) => log_warn!("Couldn't get name: {e}"),
		}
	}

	Ok(devices)
}

pub fn get_default_output_device(host_str: &str) -> Result<String, Box<dyn Error>> {
	let host_id = HostId::from_str(host_str)?;
	let host = cpal::host_from_id(host_id)?;

	// It's possible there are no devices at all
	let default_device = match host.default_output_device() {
		Some(device) => device.description()?.name().to_string(),
		None => return Err("No default device found.".into()),
	};

	Ok(default_device)
}

// search output device by name
// TODO: use DeviceId instead to simplify
// We can use device.id()?.to_string() to get a unique id, convert back using from_str
// device_id.0 should contain HostId
pub fn find_output_device(host_str: &str, device_name: &str) -> Result<Device, Box<dyn Error>> {
	let host_id = HostId::from_str(host_str)?;
	let host = cpal::host_from_id(host_id)?;

	log_info!("Using host: {}", host.id().name());

	let mut output_device = None;

	for device in host.output_devices()? {
		if let Ok(description) = device.description()
			&& description.name() == device_name
		{
			output_device = Some(device);
		}
	}

	let output_device =
		output_device.ok_or_else(|| format!("Couldn't find device \"{device_name}\""))?;

	let description = output_device.description()?;
	let name = description.name();
	log_info!("Using output device: \"{name}\"");

	Ok(output_device)
}

fn config_cmp(a: &SupportedStreamConfigRange, b: &SupportedStreamConfigRange) -> cmp::Ordering {
	use SampleFormat::*;

	let format_score = |fmt: SampleFormat| match fmt {
		// Best, no conversion
		F32 => 5,
		// Good
		I32 | U32 | F64 => 4,
		// OK
		I16 | U16 => 3,
		// No
		_ => 0,
	};

	let score_a = format_score(a.sample_format());
	let score_b = format_score(b.sample_format());

	if score_a != score_b {
		return score_a.cmp(&score_b);
	}

	// Prefer ranges that support 44100 or 48000
	let supports_rate = |config: &SupportedStreamConfigRange, target: u32| -> bool {
		config.min_sample_rate() <= target && config.max_sample_rate() >= target
	};

	// Check 44.1k
	let a_44 = supports_rate(a, 44100);
	let b_44 = supports_rate(b, 44100);
	if a_44 != b_44 {
		return a_44.cmp(&b_44);
	}

	// Check 48k
	let a_48 = supports_rate(a, 48000);
	let b_48 = supports_rate(b, 48000);
	if a_48 != b_48 {
		return a_48.cmp(&b_48);
	}

	cmp::Ordering::Equal
}

pub fn build_config(
	device: &cpal::Device,
	requested_buffer: Option<u32>,
) -> Result<(StreamConfig, SampleFormat), Box<dyn Error>> {
	let supported_configs = device.supported_output_configs()?;

	let best_config = supported_configs
		.filter(|c| c.channels() == 2) // only stereo
		.max_by(config_cmp)
		.ok_or("No supported stereo configuration found on this device.")?;

	let min_rate = best_config.min_sample_rate();
	let max_rate = best_config.max_sample_rate();

	let sample_rate = if min_rate <= 44100 && max_rate >= 44100 {
		44100
	} else if min_rate <= 48000 && max_rate >= 48000 {
		48000
	} else {
		// Fallback: Pick the closest value in range to 44100
		min_rate.max(44100).min(max_rate)
	};

	// 4. Pick the concrete Buffer Size
	let buffer_size = if let Some(req) = requested_buffer {
		match best_config.buffer_size() {
			SupportedBufferSize::Range { min, max } => BufferSize::Fixed(req.clamp(*min, *max)),
			SupportedBufferSize::Unknown => BufferSize::Fixed(req),
		}
	} else {
		BufferSize::Default
	};

	let config = StreamConfig { channels: 2, sample_rate, buffer_size };

	log_info!(
		"Selected config: sample rate: {}Hz, buffer size: {:?}, format: {:?}",
		config.sample_rate,
		config.buffer_size,
		best_config.sample_format(),
	);

	Ok((config, best_config.sample_format()))
}

#[allow(clippy::type_complexity)]
pub fn build_stream(
	device: &Device,
	config: &StreamConfig,
	format: SampleFormat,
	render: Arc<Mutex<Render>>,
) -> Result<(Stream, HeapProd<bool>, HeapCons<ErrorMessage>), Box<dyn Error>> {
	let (stream_tx, stream_rx) = HeapRb::<bool>::new(8).split();
	let (error_tx, error_rx) = HeapRb::<ErrorMessage>::new(8).split();

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
		f => Err(format!("Unsupported sample format '{f}'").into()),
	}?;

	// immediately start the stream
	stream.play()?;

	log_info!("Stream set up succesfully!");

	Ok((stream, stream_tx, error_rx))
}

pub fn build_stream_inner<T>(
	device: &Device,
	config: &StreamConfig,
	render: Arc<Mutex<Render>>,
	stream_rx: HeapCons<bool>,
	error_tx: HeapProd<ErrorMessage>,
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
	let mut process_buffer = [[0.0f32; MAX_BUF_SIZE]; 2];
	let mut cpu_load = AttackRelease::new_direct(0.05, 0.01);

	move |cpal_buffer: &mut [T], _: &cpal::OutputCallbackInfo| {
		let result = panic::catch_unwind(AssertUnwindSafe(|| {
			assert_no_alloc(|| {
				enable_fpu_traps();

				let buffer_size = cpal_buffer.len() / 2;
				match render.try_lock() {
					Some(mut render) if !is_rendering => {
						if !start {
							start = true;
							let sample_rate = render.sample_rate;
							render.send(LuaMessage::StreamSettings { buffer_size, sample_rate });
						}

						let time = std::time::Instant::now();

						// parse all messages
						for m in stream_rx.pop_iter() {
							is_rendering = m;
						}
						render.parse_messages();

						for buffer_chunk in cpal_buffer.chunks_mut(MAX_BUF_SIZE) {
							let chunk_size = buffer_chunk.len() / 2;
							let (l, r) = process_buffer.split_at_mut(1);
							let buf_slice = &mut [&mut l[0][..chunk_size], &mut r[0][..chunk_size]];

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
						let p =
							t.as_secs_f64() / (buffer_size as f64 / f64::from(render.sample_rate));
						cpu_load.set(p as f32);
						let load = cpu_load.process();
						render.send(LuaMessage::Cpu { load });
					},
					_ => {
						// Output silence as a fallback when lock fails.

						for m in stream_rx.pop_iter() {
							is_rendering = m;
						}
						// Output silence
						cpal_buffer.fill(T::EQUILIBRIUM);
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
			die();

			cpal_buffer.fill(T::EQUILIBRIUM);
		}
	}
}

pub fn die() {
	AUDIO_PANIC.store(true, atomic::Ordering::Relaxed);
}

fn error_closure(mut error_tx: HeapProd<ErrorMessage>) -> impl FnMut(StreamError) + Send + 'static {
	move |error| match error {
		StreamError::DeviceNotAvailable => {
			log_error!("Stream error: device not available.");
			error_tx
				.try_push(ErrorMessage::DeviceNotAvailable)
				.expect("Could not send message.");
		},
		StreamError::StreamInvalidated => {
			log_info!("Stream reset request");
			error_tx
				.try_push(ErrorMessage::ResetRequest)
				.expect("Could not send message.");
		},
		StreamError::BufferUnderrun => {}, // Do nothing
		StreamError::BackendSpecific { err: BackendSpecificError { ref description } } => {
			log_error!("Device error: {description}");
			// TODO should we handle these?
		},
	}
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

#[cfg(all(target_arch = "x86_64", feature = "fpu_traps"))]
#[allow(deprecated)]
fn enable_fpu_traps() {
	unsafe {
		use std::arch::x86_64::*;
		let mut mxcsr = _mm_getcsr();

		// clear the mask bits for exceptions that we care about.
		mxcsr &= !(_MM_MASK_INVALID | _MM_MASK_DIV_ZERO | _MM_MASK_OVERFLOW);
		_mm_setcsr(mxcsr);
	}
}

#[cfg(any(not(target_arch = "x86_64"), not(feature = "fpu_traps")))]
fn enable_fpu_traps() {}

#[allow(dead_code)]
fn breakpoint() {
	unsafe {
		std::arch::asm!("int3");
	}
}
