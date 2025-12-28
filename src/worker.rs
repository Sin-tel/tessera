use crate::audio::MAX_BUF_SIZE;
use crate::dsp::resample::Resampler;
use crate::embed::Asset;
use crate::log::log_error;
use anyhow::{Result, anyhow, bail};
use fft_convolver::FFTConvolver;
use hound::SampleFormat;
use std::sync::Arc;
use std::sync::mpsc;

// Worker thread for loading samples and impulse responses
pub fn spawn_worker(sample_rate: u32) -> (mpsc::SyncSender<Request>, mpsc::Receiver<Response>) {
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
						RequestData::IR(s) => match load_ir(s, sample_rate) {
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
		.expect("Failed to spawn worker thread.");

	(request_tx, response_rx)
}

pub fn load_sample(file_data: &[u8], sample_rate: u32) -> Result<[Vec<f32>; 2]> {
	let reader = hound::WavReader::new(file_data)?;
	let spec = reader.spec();

	let source_rate = spec.sample_rate as f32;
	let target_rate = sample_rate as f32;

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

	if (source_rate - target_rate).abs() < 1.0 {
		return Ok([left, right]);
	}

	let resampler = Resampler::new(source_rate, target_rate);

	let left = resampler.process(&left);
	let right = resampler.process(&right);

	Ok([left, right])
}

fn normalize(sample: &mut [Vec<f32>; 2]) {
	// Normalize by total energy

	let sqr_sum = sample.iter().flatten().fold(0.0, |sqr_sum, s| sqr_sum + s * s);
	let gain = 1.0 / sqr_sum.sqrt();

	for s in sample.iter_mut().flat_map(|s| s.iter_mut()) {
		*s *= gain;
	}
}

pub fn load_ir(path: &str, sample_rate: u32) -> Result<[FFTConvolver<f32>; 2]> {
	let file_data: &[u8] = &Asset::get(path)
		.ok_or_else(|| anyhow!("Could not find {:?}", path))?
		.data;

	let mut sample = load_sample(file_data, sample_rate)?;

	// for now we only allow short convolution samples
	let n = sample[0].len();
	if n >= 8192 {
		bail!("Impulse response {} too long: {}", path, n);
	}

	normalize(&mut sample);

	let mut conv = [FFTConvolver::default(), FFTConvolver::default()];

	conv[0].init(MAX_BUF_SIZE, &sample[0])?;
	conv[1].init(MAX_BUF_SIZE, &sample[1])?;

	Ok(conv)
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
	IR(Box<[FFTConvolver<f32>; 2]>),
}
