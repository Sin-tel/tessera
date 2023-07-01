use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use no_denormals::no_denormals;
use ringbuf::{HeapConsumer, HeapRb};
use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::dsp::env::SmoothedEnv;
use crate::lua::{AudioContext, AudioMessage, LuaMessage};
use crate::render::Render;
use crate::scope::Scope;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

pub const MAX_BUF_SIZE: usize = 128;
pub const SPECTRUM_SIZE: usize = 4096;

pub fn run(host_name: &str, output_device_name: &str) -> Result<AudioContext, Box<dyn Error>> {
	let output_device = find_output_device(host_name, output_device_name)?;

	let config = output_device.default_output_config()?;
	let mut config2: cpal::StreamConfig = config.clone().into();
	config2.channels = 2; // only allow stereo output

	// WASAPI doesn't actually return a buffer this size
	// it only guarantees it to be at least this size
	config2.buffer_size = cpal::BufferSize::Fixed(128);

	// for x in output_device.supported_output_configs().unwrap() {
	// 	dbg!(x);
	// }

	// Build streams.
	// println!("{config:?}");
	// println!("{config2:?}");

	let userdata = match config.sample_format() {
		cpal::SampleFormat::F64 => build_stream::<f64>(&output_device, &config2),
		cpal::SampleFormat::F32 => build_stream::<f32>(&output_device, &config2),

		cpal::SampleFormat::I64 => build_stream::<i64>(&output_device, &config2),
		cpal::SampleFormat::U64 => build_stream::<u64>(&output_device, &config2),

		cpal::SampleFormat::I32 => build_stream::<i32>(&output_device, &config2),
		cpal::SampleFormat::U32 => build_stream::<u32>(&output_device, &config2),

		cpal::SampleFormat::I16 => build_stream::<i16>(&output_device, &config2),
		cpal::SampleFormat::U16 => build_stream::<u16>(&output_device, &config2),
		sample_format => panic!("Unsupported sample format '{sample_format}'"),
	}?;

	userdata.stream.play()?;

	println!("Stream set up succesfully!");
	Ok(userdata)
}

pub fn build_stream<T>(
	device: &cpal::Device,
	config: &cpal::StreamConfig,
) -> Result<AudioContext, Box<dyn Error>>
where
	T: 'static + cpal::SizedSample + cpal::FromSample<f32>,
{
	let (audio_tx, audio_rx) = HeapRb::<AudioMessage>::new(256).split();
	let (stream_tx, stream_rx) = HeapRb::<bool>::new(8).split();
	let (lua_tx, lua_rx) = HeapRb::<LuaMessage>::new(256).split();
	let (scope_tx, scope_rx) = HeapRb::<f32>::new(2048).split();
	let scope = Scope::new(scope_rx);

	let sample_rate = config.sample_rate.0 as f32;

	let m_render = Arc::new(Mutex::new(Render::new(
		sample_rate,
		audio_rx,
		lua_tx,
		scope_tx,
	)));
	let m_render_clone = Arc::clone(&m_render);

	let audio_closure = build_closure::<T>(stream_rx, m_render_clone);

	let stream = device.build_output_stream(config, audio_closure, err_fn, None)?;

	Ok(AudioContext {
		stream,
		audio_tx,
		stream_tx,
		lua_rx,
		m_render,
		scope,
		paused: false,
	})
}

fn build_closure<T>(
	mut stream_rx: HeapConsumer<bool>,
	m_render: Arc<Mutex<Render>>,
) -> impl FnMut(&mut [T], &cpal::OutputCallbackInfo)
where
	T: cpal::Sample + cpal::FromSample<f32>,
{
	// Callback data
	let mut start = false;

	let mut paused = false;

	let process_buffer = [[0.0f32; MAX_BUF_SIZE]; 2];

	let mut cpu_load = SmoothedEnv::new_direct(0.05, 0.01);

	move |cpal_buffer: &mut [T], _: &cpal::OutputCallbackInfo| {
		no_denormals(|| {
			assert_no_alloc(|| {
				let cpal_buffer_size = cpal_buffer.len() / 2;
				match m_render.try_lock() {
					Ok(mut render) if !paused => {
						if !start {
							start = true;
							println!("Buffer size: {cpal_buffer_size:?}");
						}

						let time = std::time::Instant::now();

						// parse all messages
						for m in stream_rx.pop_iter() {
							paused = m;
						}
						render.parse_messages();

						for buffer_chunk in cpal_buffer.chunks_mut(MAX_BUF_SIZE) {
							let chunk_size = buffer_chunk.len() / 2;
							let [mut l, mut r] = process_buffer;
							let buf_slice = &mut [&mut l[..chunk_size], &mut r[..chunk_size]];

							let res = render.process(buf_slice);
							if let Err(e) = res {
								eprintln!("{e:?}");
								paused = true;
								for outsample in buffer_chunk.chunks_exact_mut(2) {
									outsample[0] = T::from_sample(0.0f32);
									outsample[1] = T::from_sample(0.0f32);
								}
							} else {
								// interlace and convert
								for (i, outsample) in buffer_chunk.chunks_exact_mut(2).enumerate() {
									outsample[0] = T::from_sample(buf_slice[0][i]);
									outsample[1] = T::from_sample(buf_slice[1][i]);
								}
							}
						}

						let t = time.elapsed();
						let p = t.as_secs_f64()
							/ (cpal_buffer_size as f64 / f64::from(render.sample_rate));
						cpu_load.set(p as f32);
						let load = cpu_load.process();
						render.send(LuaMessage::Cpu(load));
					}
					_ => {
						// Output silence as a fallback when lock fails.

						for m in stream_rx.pop_iter() {
							paused = m;
						}
						// println!("Output silent");

						for outsample in cpal_buffer.chunks_exact_mut(2) {
							outsample[0] = T::from_sample(0.0f32);
							outsample[1] = T::from_sample(0.0f32);
						}
					}
				}
			});
		});
	}
}

fn find_output_device(
	host_name: &str,
	output_device_name: &str,
) -> Result<cpal::Device, Box<dyn Error>> {
	let available_hosts = cpal::available_hosts();
	println!("Available hosts:\n  {available_hosts:?}");

	let mut host = None;
	if host_name == "default" {
		host = Some(cpal::default_host());
	} else {
		for host_id in available_hosts {
			if host_id
				.name()
				.to_lowercase()
				.contains(&host_name.to_lowercase())
			{
				host = Some(cpal::host_from_id(host_id)?);
				break;
			}
		}
	}
	let host = match host {
		Some(h) => h,
		None => {
			println!("Couldn't find {host_name}. Using default instead");
			cpal::default_host()
		}
	};

	println!("Using host: {}", host.id().name());

	println!("Avaliable output devices:");
	for d in host.output_devices()? {
		println!(" - \"{}\"", d.name()?);
	}

	let mut output_device = None;

	if output_device_name == "default" {
		output_device = host.default_output_device();
	} else {
		for device in host.output_devices().expect("No output devices found.") {
			if let Ok(name) = device.name() {
				if name
					.to_lowercase()
					.contains(&output_device_name.to_lowercase())
				{
					output_device = Some(device);
				}
			}
		}
	}

	let output_device = match output_device {
		Some(d) => d,
		None => {
			println!("Couldn't find {output_device_name}. Using default instead");
			host.default_output_device()
				.expect("No default output device found.")
		}
	};

	println!("Using output device: \"{}\"", output_device.name()?);

	Ok(output_device)
}

#[allow(clippy::needless_pass_by_value)]
fn err_fn(err: cpal::StreamError) {
	eprintln!("an error occurred on stream: {err}");
}
