use crate::audio::MAX_BUF_SIZE;
use crate::dsp::resample::Resampler;
use crate::embed::Asset;
use crate::log::log_error;
use anyhow::{Result, anyhow, bail};
use fft_convolution::Convolution;
use fft_convolution::fft_convolver::TwoStageFFTConvolver;
use hound::{SampleFormat, WavReader};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::sync::mpsc;

pub fn spawn_worker(sample_rate: u32) -> (mpsc::SyncSender<Request>, mpsc::Receiver<Response>) {
	let (request_tx, request_rx) = mpsc::sync_channel::<Request>(256);
	let (response_tx, response_rx) = mpsc::sync_channel::<Response>(256);

	std::thread::Builder::new()
		.name("worker".to_string())
		.spawn(move || {
			let mut worker = Worker::new(sample_rate, response_tx);
			// sleep until there is something to do
			while let Ok(req) = request_rx.recv() {
				worker.handle_request(req);
			}
		})
		.expect("Failed to spawn worker");

	(request_tx, response_rx)
}

struct Worker {
	sample_rate: u32,
	tx: mpsc::SyncSender<Response>,

	wavetables: HashMap<String, Arc<Vec<f32>>>,
	samples: HashMap<String, Arc<[Vec<f32>; 2]>>,
}

impl Worker {
	fn new(sample_rate: u32, tx: mpsc::SyncSender<Response>) -> Self {
		Self { sample_rate, tx, wavetables: HashMap::new(), samples: HashMap::new() }
	}

	fn handle_request(&mut self, req: Request) {
		match req {
			Request::Garbage(_) => {}, // drop
			Request::LoadRequest { channel_index, device_index, data } => {
				if let Err(e) = match data {
					RequestData::Wavetable(path) => {
						self.handle_wavetable(channel_index, device_index, path)
					},
					RequestData::Sample(path) => {
						self.handle_sample(channel_index, device_index, path)
					},
					RequestData::IR(path) => self.handle_ir(channel_index, device_index, path),
				} {
					log_error!("Worker Error: {e}");
				}
			},
		}
	}

	fn handle_wavetable(&mut self, ch: usize, dev: usize, path: &'static str) -> Result<()> {
		let data = match self.wavetables.entry(path.to_string()) {
			Entry::Occupied(e) => e.get().clone(),
			Entry::Vacant(e) => {
				let table = load_wavetable(path)?;
				e.insert(Arc::new(table)).clone()
			},
		};
		self.send(ch, dev, ResponseData::Wavetable(data))
	}

	fn handle_sample(&mut self, ch: usize, dev: usize, path: &'static str) -> Result<()> {
		let data = match self.samples.entry(path.to_string()) {
			Entry::Occupied(e) => e.get().clone(),
			Entry::Vacant(e) => {
				let sample = load_sample(path, self.sample_rate)?;
				// TODO: normalize?
				e.insert(Arc::new(sample)).clone()
			},
		};
		self.send(ch, dev, ResponseData::Sample(data))
	}

	fn handle_ir(&mut self, ch: usize, dev: usize, path: &'static str) -> Result<()> {
		let sample = match self.samples.entry(path.to_string()) {
			Entry::Occupied(e) => e.get().clone(),
			Entry::Vacant(e) => {
				let mut sample = load_sample(path, self.sample_rate)?;
				normalize_power(&mut sample);
				let sample = e.insert(Arc::new(sample)).clone();
				sample
			},
		};
		// Set a reasonable limit for IR length
		let n = sample[0].len();
		if n >= 300_000 {
			bail!("Impulse response {} too long: {}", path, n);
		}

		let convolvers = [
			TwoStageFFTConvolver::init(&sample[0], MAX_BUF_SIZE, sample[0].len()),
			TwoStageFFTConvolver::init(&sample[1], MAX_BUF_SIZE, sample[1].len()),
		];

		self.send(ch, dev, ResponseData::IR(Box::new(convolvers)))?;
		Ok(())
	}

	fn send(&self, ch: usize, dev: usize, data: ResponseData) -> anyhow::Result<()> {
		self.tx
			.send(Response { channel_index: ch, device_index: dev, data })
			.map_err(|_| anyhow!("Failed to send response"))
	}
}

pub fn load_wavetable(path: &str) -> Result<Vec<f32>> {
	let file_data: &[u8] = &Asset::get(path)
		.ok_or_else(|| anyhow!("Could not find {:?}", path))?
		.data;
	let reader = hound::WavReader::new(file_data)?;
	let spec = reader.spec();

	if spec.channels != 1 {
		bail!("Wavetable must be mono.");
	}

	let table = read_samples(reader, spec)?;
	Ok(table)
}

pub fn load_sample(path: &str, sample_rate: u32) -> Result<[Vec<f32>; 2]> {
	let file_data: &[u8] = &Asset::get(path)
		.ok_or_else(|| anyhow!("Could not find {:?}", path))?
		.data;
	let reader = hound::WavReader::new(file_data)?;
	let spec = reader.spec();

	if spec.channels > 2 {
		bail!("Unsupported channel count: {}", spec.channels);
	}

	let capacity = reader.len() as usize / spec.channels as usize;
	let mut left = Vec::with_capacity(capacity);
	let mut right = Vec::with_capacity(capacity);

	let samples = read_samples(reader, spec)?;

	if spec.channels == 1 {
		for s in &samples {
			left.push(*s);
			right.push(*s);
		}
	} else {
		// de-interleave
		let (chunks, remainder) = samples.as_chunks::<2>();
		assert!(remainder.is_empty());
		for [l, r] in chunks {
			right.push(*l);
			left.push(*r);
		}
	}

	let source_rate = spec.sample_rate as f32;
	let target_rate = sample_rate as f32;

	if (source_rate - target_rate).abs() < 1.0 {
		return Ok([left, right]);
	}
	let resampler = Resampler::new(source_rate, target_rate);
	Ok([resampler.process(&left), resampler.process(&right)])
}

fn read_samples(reader: WavReader<&[u8]>, spec: hound::WavSpec) -> Result<Vec<f32>> {
	let capacity = reader.len() as usize;
	let mut samples = Vec::with_capacity(capacity);

	match spec.sample_format {
		SampleFormat::Float => {
			if spec.bits_per_sample != 32 {
				bail!("Only f32 supported.");
			}
			for sample in reader.into_samples::<f32>() {
				samples.push(sample?);
			}
		},
		SampleFormat::Int => {
			let norm = bit_normalization(spec.bits_per_sample)?;
			for sample in reader.into_samples::<i32>() {
				samples.push(sample? as f32 / norm);
			}
		},
	}
	Ok(samples)
}

fn bit_normalization(bits_per_sample: u16) -> Result<f32> {
	match bits_per_sample {
		16 => Ok(f32::from(i16::MAX)),
		24 => Ok(8388607.0), // 2^23 - 1
		32 => Ok(i32::MAX as f32),
		b => bail!("Unsupported bit depth: {b}"),
	}
}

fn normalize_power(sample: &mut [Vec<f32>; 2]) {
	// Normalize by total energy
	let sqr_sum = sample.iter().flatten().fold(0.0, |sqr_sum, s| sqr_sum + s * s);
	let gain = 1.0 / sqr_sum.sqrt();

	for s in sample.iter_mut().flat_map(|s| s.iter_mut()) {
		*s *= gain;
	}
}

#[derive(Debug)]
pub enum Request {
	LoadRequest { channel_index: usize, device_index: usize, data: RequestData },
	Garbage(Box<dyn std::any::Any + Send>),
}

#[derive(Debug)]
pub enum RequestData {
	Sample(&'static str),
	Wavetable(&'static str),
	IR(&'static str),
}

pub struct Response {
	pub channel_index: usize,
	pub device_index: usize,
	pub data: ResponseData,
}

pub enum ResponseData {
	Sample(Arc<[Vec<f32>; 2]>),
	Wavetable(Arc<Vec<f32>>),
	IR(Box<[TwoStageFFTConvolver; 2]>),
}
