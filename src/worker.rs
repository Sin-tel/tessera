use crate::audio::MAX_BUF_SIZE;
use crate::embed::Asset;
use crate::log::log_error;
use fft_convolver::FFTConvolver;
use std::sync::Arc;
use std::sync::mpsc;

// TODO: abstract out wav handling into seperate module
// TODO: handle more kinds of sample specs
// TODO: resample to get sample rate independence

// Worker thread for loading samples and impulse responses
pub fn spawn_worker() -> (mpsc::SyncSender<Request>, mpsc::Receiver<Response>) {
	let (request_tx, request_rx) = mpsc::sync_channel::<Request>(256);
	let (response_tx, response_rx) = mpsc::sync_channel::<Response>(256);

	std::thread::spawn(move || {
		// sleeps until there is some work to do
		while let Ok(req) = request_rx.recv() {
			match req {
				Request::Garbage(_) => {}, // drop
				Request::LoadRequest { channel_index, device_index, data } => match data {
					RequestData::IR(s) => {
						let convolver = load_ir(s);

						let response = Response {
							channel_index,
							device_index,
							data: ResponseData::IR(Box::new(convolver)),
						};

						if let Err(e) = response_tx.send(response) {
							log_error!("{e}");
						}
					},
					RequestData::Sample(_) => todo!(),
				},
			}
		}
	});

	(request_tx, response_rx)
}

pub fn load_ir(path: &str) -> FFTConvolver<f32> {
	let file: &[u8] = &Asset::get(path).unwrap().data;
	let reader = hound::WavReader::new(file).unwrap();

	let spec = reader.spec();

	assert!(spec.sample_rate == 44100);
	assert!(spec.bits_per_sample == 16);
	assert!(spec.sample_format == hound::SampleFormat::Int);

	let mut impulse_response: Vec<f32> = reader
		.into_samples::<i16>()
		.step_by(spec.channels.into()) // stereo samples are interleaved
		.map(|s| f32::from(s.unwrap()) / f32::from(i16::MAX))
		.collect();

	// for now we only allow short convolution samples
	assert!(impulse_response.len() < 2048);

	let sqr_sum = impulse_response.iter().fold(0.0, |sqr_sum, s| sqr_sum + s * s);
	let gain = 0.5 / sqr_sum.sqrt();

	for s in &mut impulse_response {
		*s *= gain;
	}

	let mut convolver = FFTConvolver::default();
	let init_result = convolver.init(MAX_BUF_SIZE, &impulse_response);
	assert!(init_result.is_ok());

	// println!("Loaded convolver: {}", &path);
	convolver
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
