use std::error::Error;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{Consumer, RingBuffer};

use assert_no_alloc::*;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

use crate::defs::*;
use crate::dsp::*;
use crate::ffi::*;
use crate::render::*;

pub fn audio_run(host_name: &str, output_device_name: &str) -> Result<Userdata, Box<dyn Error>> {
	let output_device = find_output_device(host_name, output_device_name)?;

	let config = output_device.default_output_config()?;
	let mut config2: cpal::StreamConfig = config.clone().into();
	config2.channels = 2; // only allow stereo output

	// config2.buffer_size = cpal::BufferSize::Fixed(128);
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

	// Err("Something wrong".into())

	println!("Stream set up succesfully!");
	Ok(userdata)
}

pub fn build_stream<T: 'static>(device: &cpal::Device, config: &cpal::StreamConfig) -> Userdata
where
	T: cpal::Sample,
{
	let (audio_tx, audio_rx) = RingBuffer::<AudioMessage>::new(256).split();

	let (stream_tx, stream_rx) = RingBuffer::<bool>::new(8).split();

	let (lua_tx, lua_rx) = RingBuffer::<LuaMessage>::new(256).split();

	let sample_rate = config.sample_rate.0 as f32;

	let m_render = Arc::new(Mutex::new(Render::new(sample_rate, audio_rx, lua_tx)));
	let m_render_clone = Arc::clone(&m_render);

	let audio_closure = build_closure::<T>(stream_rx, m_render_clone);

	let stream = device
		.build_output_stream(config, audio_closure, err_fn)
		.unwrap();

	Userdata {
		stream,
		audio_tx,
		stream_tx,
		lua_rx,
		m_render,
	}
}

fn build_closure<T>(
	mut stream_rx: Consumer<bool>,
	m_render: Arc<Mutex<Render>>,
) -> impl FnMut(&mut [T], &cpal::OutputCallbackInfo)
where
	T: cpal::Sample,
{
	// Callback data
	let mut start = false;

	let mut paused = false;

	let mut audiobuf = [StereoSample { l: 0.0, r: 0.0 }; MAX_BUF_SIZE];

	let mut cpu_load = SmoothedEnv::new(0.0, 0.2, 0.0005);

	move |buffer: &mut [T], _: &cpal::OutputCallbackInfo| {
		assert_no_alloc(|| {
			let buf_size = buffer.len() / 2;

			// dbg!(buf_size);

			assert!(buf_size <= MAX_BUF_SIZE);

			let mut buf_slice = &mut audiobuf[..buf_size];

			match m_render.try_lock() {
				Ok(mut render) if !paused => {
					if !start {
						start = true;
						fix_denorms();
					}
					let time = std::time::Instant::now();

					// parse all messages
					stream_rx.pop_each(
						|m| {
							paused = m;
							true
						},
						None,
					);
					render.parse_messages();
					// write to output buffer
					render.process(&mut buf_slice);

					for (outsample, gensample) in buffer.chunks_exact_mut(2).zip(buf_slice.iter()) {
						outsample[0] = cpal::Sample::from::<f32>(&gensample.l);
						outsample[1] = cpal::Sample::from::<f32>(&gensample.r);
					}

					let t = time.elapsed();
					let p = t.as_secs_f64() / ((buf_size as f64) / render.sample_rate as f64);
					cpu_load.set(p as f32);
					cpu_load.update();
					render.send(LuaMessage::Cpu(cpu_load.value));
				}
				_ => {
					stream_rx.pop_each(
						|m| {
							paused = m;
							true
						},
						None,
					);
					println!("Lock failed. Output silence.");

					for outsample in buffer.chunks_exact_mut(2) {
						outsample[0] = cpal::Sample::from::<f32>(&0.0f32);
						outsample[1] = cpal::Sample::from::<f32>(&0.0f32);
					}
				}
			}
		});
	}
}

fn find_output_device(
	host_name: &str,
	output_device_name: &str,
) -> Result<cpal::Device, Box<dyn Error>> {
	let available_hosts = cpal::available_hosts();
	println!("Available hosts:\n  {:?}", available_hosts);

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
			println!("Couldn't find {}. Using default instead", host_name);
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
			println!(
				"Couldn't find {}. Using default instead",
				output_device_name
			);
			host.default_output_device()
				.expect("No default output device found.")
		}
	};

	println!("Using output device: \"{}\"", output_device.name()?);

	Ok(output_device)
}

fn err_fn(err: cpal::StreamError) {
	eprintln!("an error occurred on stream: {}", err);
}

pub fn fix_denorms() {
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
