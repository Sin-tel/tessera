use crate::log::{log_info, log_warn};
use crate::midi;
use crate::render::Render;
use crate::scope::Scope;
use mlua::Value;
use mlua::prelude::*;
use parking_lot::Mutex;
use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd};
use std::sync::Arc;

pub struct AudioContext {
	pub stream: cpal::Stream,
	pub audio_tx: HeapProd<AudioMessage>,
	pub stream_tx: HeapProd<bool>,
	pub lua_rx: HeapCons<LuaMessage>,
	pub m_render: Arc<Mutex<Render>>,
	pub scope: Scope,
	pub is_rendering: bool,
	pub sample_rate: u32,
	pub midi_connections: Vec<midi::Connection>,
	pub render_buffer: Vec<f32>,
}

impl AudioContext {
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
