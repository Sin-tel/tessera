use crate::audio::build_stream;
use crate::audio::get_device_and_config;
use crate::log::{log_info, log_warn};
use crate::midi;
use crate::render::Render;
use crate::scope::Scope;
use mlua::Value;
use mlua::prelude::*;
use parking_lot::Mutex;
use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd, HeapRb};
use std::error::Error;
use std::sync::Arc;

pub struct AudioContext {
	pub stream: Option<cpal::Stream>,
	pub audio_tx: HeapProd<AudioMessage>,
	pub stream_tx: HeapProd<bool>,
	pub error_rx: HeapCons<bool>,
	pub lua_rx: HeapCons<LuaMessage>,
	pub render: Arc<Mutex<Render>>,
	pub scope: Scope,
	pub is_rendering: bool,
	pub sample_rate: u32,
	pub midi_connections: Vec<midi::Connection>,
	pub render_buffer: Vec<f32>,
}

impl AudioContext {
	pub fn new(
		host_name: &str,
		output_device_name: &str,
		buffer_size: Option<u32>,
	) -> Result<AudioContext, Box<dyn Error>> {
		let (device, config, format) =
			get_device_and_config(host_name, output_device_name, buffer_size)?;
		let sample_rate = config.sample_rate.0;

		let (audio_tx, audio_rx) = HeapRb::<AudioMessage>::new(256).split();
		let (lua_tx, lua_rx) = HeapRb::<LuaMessage>::new(256).split();
		let (scope_tx, scope_rx) = HeapRb::<f32>::new(2048).split();
		let scope = Scope::new(scope_rx);

		let render = Render::new(sample_rate as f32, audio_rx, lua_tx, scope_tx);
		let render = Arc::new(Mutex::new(render));

		let (stream, stream_tx, error_rx) =
			build_stream(&device, &config, format, Arc::clone(&render))?;

		Ok(AudioContext {
			stream: Some(stream),
			audio_tx,
			stream_tx,
			error_rx,
			lua_rx,
			render,
			scope,
			is_rendering: false,
			sample_rate,
			midi_connections: Vec::new(),
			render_buffer: Vec::new(),
		})
	}

	pub fn rebuild_stream(
		&mut self,
		host_name: &str,
		output_device_name: &str,
		buffer_size: Option<u32>,
	) -> Result<(), Box<dyn Error>> {
		// drop old stream
		self.stream = None;

		let (device, config, format) =
			get_device_and_config(host_name, output_device_name, buffer_size)?;

		// TODO: handle this properly
		assert_eq!(config.sample_rate.0, self.sample_rate, "Sample rate mismatch during rebuild");

		let (stream, stream_tx, error_rx) =
			build_stream(&device, &config, format, Arc::clone(&self.render))?;

		self.stream_tx = stream_tx;
		self.error_rx = error_rx;
		self.stream = Some(stream);

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

	pub fn check_should_rebuild(&mut self) -> bool {
		let mut rebuild = false;
		for m in self.error_rx.pop_iter() {
			rebuild = m;
		}
		rebuild
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
	NoteOn(usize, f32, f32, usize),
	NoteOff(usize, usize),
	Pitch(usize, f32, usize),
	Pressure(usize, f32, usize),
	Parameter(usize, usize, usize, f32),
	Mute(usize, bool),
	Bypass(usize, usize, bool),
	ReorderEffect(usize, usize, usize),
	// Swap(?),
}

#[derive(Debug)]
pub enum LuaMessage {
	Cpu(f32),
	Meter(f32, f32),
}

impl IntoLua for LuaMessage {
	fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
		use LuaMessage::*;

		let table = Lua::create_table(lua)?;

		match self {
			Cpu(cpu_load) => {
				table.set("tag", "cpu")?;
				table.set("cpu_load", cpu_load)?;
			},
			Meter(l, r) => {
				table.set("tag", "meter")?;
				table.set("l", l)?;
				table.set("r", r)?;
			},
		}

		Ok(Value::Table(table))
	}
}
