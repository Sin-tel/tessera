use crate::audio::{build_config, build_stream, die, find_output_device};
use crate::log::*;
use crate::meters::Meters;
use crate::render::Render;
use crate::scope::Scope;
use crate::voice_manager::Token;
use anyhow::Result;
use parking_lot::Mutex;
use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd, HeapRb};
use serde::Serialize;
use std::sync::Arc;

pub struct AudioContext {
	pub stream: Option<cpal::Stream>,
	pub device: Option<cpal::Device>,
	pub audio_tx: HeapProd<AudioMessage>,
	pub stream_tx: HeapProd<bool>,
	pub error_rx: HeapCons<ErrorMessage>,
	pub lua_rx: HeapCons<LuaMessage>,
	pub render: Arc<Mutex<Render>>,
	pub scope: Scope,
	pub is_rendering: bool,
	pub sample_rate: u32,
	pub render_buffer: Vec<f32>,
	pub meters: Meters,
}

impl AudioContext {
	pub fn new(
		host_str: &str,
		device_name: &str,
		buffer_size: Option<u32>,
	) -> Result<AudioContext> {
		let device = find_output_device(host_str, device_name)?;
		let (config, format) = build_config(&device, buffer_size)?;
		let sample_rate = config.sample_rate;

		let (audio_tx, audio_rx) = HeapRb::<AudioMessage>::new(1024).split();
		let (lua_tx, lua_rx) = HeapRb::<LuaMessage>::new(256).split();
		let (scope_tx, scope_rx) = HeapRb::<f32>::new(2048).split();
		let scope = Scope::new(scope_rx);

		let render = Render::new(sample_rate as f32, audio_rx, lua_tx, scope_tx);
		let render = Arc::new(Mutex::new(render));

		let (stream, stream_tx, error_rx) =
			build_stream(&device, &config, format, Arc::clone(&render))?;

		Ok(AudioContext {
			stream: Some(stream),
			device: Some(device),
			audio_tx,
			stream_tx,
			error_rx,
			lua_rx,
			render,
			scope,
			is_rendering: false,
			sample_rate,
			render_buffer: Vec::new(),
			meters: Meters::new(),
		})
	}

	pub fn rebuild_stream(
		&mut self,
		host_str: &str,
		device_name: &str,
		buffer_size: Option<u32>,
	) -> Result<()> {
		// drop old stream
		self.stream = None;
		self.device = None;

		let device = find_output_device(host_str, device_name)?;
		let (config, format) = build_config(&device, buffer_size)?;

		// TODO: handle this properly
		if config.sample_rate != self.sample_rate {
			log_error!("Sample rate mismatch!");
			self.sample_rate = config.sample_rate;
			die();
		}

		let (stream, stream_tx, error_rx) =
			build_stream(&device, &config, format, Arc::clone(&self.render))?;

		self.stream_tx = stream_tx;
		self.error_rx = error_rx;
		self.stream = Some(stream);
		self.device = Some(device);

		Ok(())
	}

	pub fn send_message(&mut self, m: AudioMessage) {
		if self.audio_tx.try_push(m).is_err() {
			log_warn!("Queue full. Dropped message!");
		}
	}

	pub fn send_rendering(&mut self, is_rendering: bool) {
		self.is_rendering = is_rendering;
		if self.stream_tx.try_push(is_rendering).is_err() {
			log_warn!("Stream queue full. Dropped message!");
		}
	}
}

impl Drop for AudioContext {
	fn drop(&mut self) {
		log_info!("Stream dropped");
	}
}

// Message struct to pass to audio thread
// Should not contain any boxed values
#[derive(Debug)]
pub enum AudioMessage {
	Panic,
	AllNotesOff,
	NoteOn(usize, Token, f32, f32),
	NoteOff(usize, Token),
	Pitch(usize, Token, f32),
	Pressure(usize, Token, f32),
	Sustain(usize, bool),
	Parameter(usize, usize, usize, f32),
	DeviceMute(usize, usize, bool),
	ChannelMute(usize, bool),
	ChannelGain(usize, f32),
	ReorderEffect(usize, usize, usize),
	// Swap(?),
}

#[derive(Debug, Serialize)]
#[serde(tag = "tag")]
pub enum LuaMessage {
	Cpu { load: f32 },
	Meter { l: f32, r: f32 },
	StreamSettings { buffer_size: usize, sample_rate: f32 },
}

#[derive(Debug, Serialize)]
#[serde(tag = "tag")]
pub enum ErrorMessage {
	ResetRequest,
	DeviceNotAvailable,
}
