use crate::audio::MAX_BUF_SIZE;
use crate::dsp::resample::Resampler;
use crate::embed::Asset;
use crate::log::*;
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
	tx: mpsc::SyncSender<Response>,
	loader: Loader,
}

impl Worker {
	fn new(sample_rate: u32, tx: mpsc::SyncSender<Response>) -> Self {
		Self { tx, loader: Loader::new(sample_rate) }
	}

	fn handle_request(&mut self, req: Request) {
		match req {
			Request::Garbage(_) => {}, // drop
			Request::LoadRequest { channel_index, device_index, data } => {
				match self.loader.load(data) {
					Ok(data) => {
						if let Err(e) = self.send(channel_index, device_index, data) {
							log_error!("Worker Error: {e}");
						}
					},
					Err(e) => log_error!("Worker Error: {e}"),
				}
			},
		}
	}

	fn send(&self, ch: usize, dev: usize, data: ResponseData) -> anyhow::Result<()> {
		self.tx
			.send(Response { channel_index: ch, device_index: dev, data })
			.map_err(|_| anyhow!("Failed to send response"))
	}
}

pub struct Loader {
	sample_rate: u32,
	wavetables: HashMap<String, Arc<Vec<f32>>>,
	samples: HashMap<String, Arc<[Vec<f32>; 2]>>,
}

impl Loader {
	pub fn new(sample_rate: u32) -> Self {
		Self { sample_rate, wavetables: HashMap::new(), samples: HashMap::new() }
	}

	pub fn load(&mut self, data: RequestData) -> Result<ResponseData> {
		match data {
			RequestData::Wavetable(path) => self.load_wavetable(path),
			RequestData::Sample(path) => self.load_sample(path),
			RequestData::IR(path) => self.load_ir(path),
		}
	}

	fn load_wavetable(&mut self, path: &'static str) -> Result<ResponseData> {
		let data = match self.wavetables.entry(path.to_string()) {
			Entry::Occupied(e) => e.get().clone(),
			Entry::Vacant(e) => {
				let table = load_wavetable(path)?;
				e.insert(Arc::new(table)).clone()
			},
		};
		Ok(ResponseData::Wavetable(data))
	}

	fn load_sample(&mut self, path: &'static str) -> Result<ResponseData> {
		let data = match self.samples.entry(path.to_string()) {
			Entry::Occupied(e) => e.get().clone(),
			Entry::Vacant(e) => {
				let (sample, sr) = load_sample(path)?;

				if sr != 44100 {
					log_warn!("Sample {} has sample_rate {}", path, sr);
				}
				// TODO: normalize?
				e.insert(Arc::new(sample)).clone()
			},
		};
		Ok(ResponseData::Sample(data))
	}

	fn load_ir(&mut self, path: &'static str) -> Result<ResponseData> {
		let sample = match self.samples.entry(path.to_string()) {
			Entry::Occupied(e) => e.get().clone(),
			Entry::Vacant(e) => {
				let mut sample = load_and_resample(path, self.sample_rate as f32)?;
				normalize_power(&mut sample);
				e.insert(Arc::new(sample)).clone()
			},
		};
		// Set a reasonable limit for IR length
		let n = sample[0].len();
		if n >= 600_000 {
			bail!("Impulse response \"{path}\" too long: {n}");
		}

		let convolvers = [
			TwoStageFFTConvolver::init(&sample[0], MAX_BUF_SIZE, sample[0].len()),
			TwoStageFFTConvolver::init(&sample[1], MAX_BUF_SIZE, sample[1].len()),
		];

		Ok(ResponseData::IR(Box::new(convolvers)))
	}
}

pub fn load_wavetable(path: &str) -> Result<Vec<f32>> {
	let file_data: &[u8] = &Asset::get(path).ok_or_else(|| anyhow!("Could not find {path}"))?.data;
	let reader = hound::WavReader::new(file_data)?;
	let spec = reader.spec();

	if spec.channels != 1 {
		bail!("Wavetable must be mono.");
	}

	let table = read_samples(reader, spec)?;
	Ok(table)
}

pub fn load_and_resample(path: &str, target_rate: f32) -> Result<[Vec<f32>; 2]> {
	let (mut sample, source_rate) = load_sample(path)?;
	let source_rate = source_rate as f32;

	if (source_rate - target_rate).abs() > 1.0 {
		let resampler = Resampler::new(source_rate, target_rate);
		sample = [resampler.process(&sample[0]), resampler.process(&sample[1])];
	}

	Ok(sample)
}

pub fn load_sample(path: &str) -> Result<([Vec<f32>; 2], u32)> {
	let file_data: &[u8] = &Asset::get(path).ok_or_else(|| anyhow!("Could not find {path}"))?.data;
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
			left.push(*l);
			right.push(*r);
		}
	}
	Ok(([left, right], spec.sample_rate))
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
	// Normalize channels by total energy
	for channel in sample.iter_mut() {
		let sqr_sum = channel.iter().fold(0.0, |sqr_sum, s| sqr_sum + s * s);
		let gain = 0.5 / sqr_sum.sqrt();

		for s in channel.iter_mut() {
			*s *= gain;
		}
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
