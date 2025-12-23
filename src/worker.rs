use crate::audio::MAX_BUF_SIZE;
use crate::embed::Asset;
use crate::log::log_error;
use anyhow::{Result, bail};
use fft_convolver::FFTConvolver;
use hound::SampleFormat;
use std::sync::Arc;
use std::sync::mpsc;

// TODO: resample to get sample rate independence

// Worker thread for loading samples and impulse responses
pub fn spawn_worker() -> (mpsc::SyncSender<Request>, mpsc::Receiver<Response>) {
	let (request_tx, request_rx) = mpsc::sync_channel::<Request>(256);
	let (response_tx, response_rx) = mpsc::sync_channel::<Response>(256);

	std::thread::Builder::new()
		.name("worker".to_string())
		.spawn(move || {
			// sleeps until there is some work to do
			while let Ok(req) = request_rx.recv() {
				match req {
					Request::Garbage(_) => {}, // drop
					Request::LoadRequest { channel_index, device_index, data } => match data {
						RequestData::IR(s) => match load_ir(s) {
							Ok(convolver) => {
								let response = Response {
									channel_index,
									device_index,
									data: ResponseData::IR(Box::new(convolver)),
								};

								if let Err(e) = response_tx.send(response) {
									log_error!("{e}");
								}
							},
							Err(e) => log_error!("{e}"),
						},
						RequestData::Sample(_) => todo!(),
					},
				}
			}
		})
		.unwrap();

	(request_tx, response_rx)
}

pub fn load_sample(file_data: &[u8]) -> Result<[Vec<f32>; 2]> {
	let reader = hound::WavReader::new(file_data).unwrap();
	let spec = reader.spec();

	if spec.sample_rate != 44100 {
		bail!("Unsupported sample rate: {}", spec.sample_rate);
	}

	// Accept only mono or stereo
	if spec.channels > 2 {
		bail!("Unsupported channel count: {}", spec.channels);
	}

	let capacity = reader.len() as usize / spec.channels as usize;
	let mut left = Vec::with_capacity(capacity);
	let mut right = Vec::with_capacity(capacity);

	match spec.sample_format {
		SampleFormat::Float => {
			if spec.bits_per_sample != 32 {
				bail!("Only f32 supported.");
			}
			let samples = reader.into_samples::<f32>();
			if spec.channels == 1 {
				for s in samples {
					let s = s.unwrap();
					left.push(s);
					right.push(s);
				}
			} else {
				let samples: Vec<_> = samples.map(|s| s.unwrap()).collect();
				let (chunks, remainder) = samples.as_chunks::<2>();
				assert!(remainder.is_empty());
				for [l, r] in chunks {
					left.push(*l);
					right.push(*r);
				}
			}
		},
		SampleFormat::Int => {
			let samples = reader.into_samples::<i32>();

			let norm = match spec.bits_per_sample {
				16 => f32::from(i16::MAX),
				24 => 8388607.0, // 2^23 - 1
				32 => i32::MAX as f32,
				b => bail!("Unsupported bit depth: {b}"),
			};

			if spec.channels == 1 {
				for s in samples {
					let s = s.unwrap();
					left.push(s as f32 / norm);
					right.push(s as f32 / norm);
				}
			} else {
				let samples: Vec<_> = samples.map(|s| s.unwrap()).collect();
				let (chunks, remainder) = samples.as_chunks::<2>();
				assert!(remainder.is_empty());
				for [l, r] in chunks {
					left.push(*l as f32 / norm);
					right.push(*r as f32 / norm);
				}
			}
		},
	}

	Ok([left, right])
}

pub fn load_ir(path: &str) -> Result<FFTConvolver<f32>> {
	let file_data: &[u8] = &Asset::get(path).unwrap().data;

	let mut sample = load_sample(file_data)?;

	// take left channel
	// TODO: support stereo
	let impulse_response = &mut sample[0];

	// for now we only allow short convolution samples
	assert!(
		impulse_response.len() < 8192,
		"Impulse response {} too long: {}",
		path,
		impulse_response.len()
	);

	// Normalize by total energy
	let sqr_sum = impulse_response.iter().fold(0.0, |sqr_sum, s| sqr_sum + s * s);
	let gain = 0.5 / sqr_sum.sqrt();

	for s in &mut *impulse_response {
		*s *= gain;
	}

	let mut convolver = FFTConvolver::default();
	let init_result = convolver.init(MAX_BUF_SIZE, impulse_response);
	assert!(init_result.is_ok());

	// println!("Loaded convolver: {}", &path);
	Ok(convolver)
}

#[derive(Debug)]
pub enum Request {
	LoadRequest { channel_index: usize, device_index: usize, data: RequestData },
	Garbage(Box<dyn std::any::Any + Send>),
}

#[derive(Debug)]
pub enum RequestData {
	Sample(&'static str),
	IR(&'static str),
}

pub struct Response {
	pub channel_index: usize,
	pub device_index: usize,
	pub data: ResponseData,
}

pub enum ResponseData {
	Sample(Arc<[Vec<f32>; 2]>),
	IR(Box<FFTConvolver<f32>>),
}
