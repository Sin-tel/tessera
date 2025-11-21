use mlua::prelude::*;
use mlua::{UserData, UserDataMethods, Value};
use no_denormals::no_denormals;
use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd};
use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::audio;
use crate::log::{init_logging, log_error, log_info, log_warn};
use crate::midi;
use crate::render::Render;
use crate::scope::Scope;

struct LuaData(Option<AudioContext>);

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

impl Drop for AudioContext {
	fn drop(&mut self) {
		log_info!("Stream dropped");
	}
}

#[allow(clippy::unnecessary_wraps)]
fn init(_: &Lua, _: ()) -> LuaResult<LuaData> {
	init_logging();
	log_info!("Backend initialized");
	Ok(LuaData(None))
}

impl UserData for LuaData {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut(
			"setup",
			|_, data, (host_name, device_name, buffer_size): (String, String, Option<u32>)| {
				match audio::run(&host_name, &device_name, buffer_size) {
					Ok(ud) => {
						*data = LuaData(Some(ud));
						Ok(())
					},
					Err(e) => {
						log_error!("{e}");
						*data = LuaData(None);
						Ok(())
					},
				}
			},
		);

		methods.add_method_mut("quit", |_, data, ()| {
			*data = LuaData(None);
			Ok(())
		});

		methods.add_method_mut("panic", |_, data, ()| {
			if let LuaData(Some(ud)) = data {
				ud.send_message(AudioMessage::Panic);
			}
			Ok(())
		});

		methods.add_method_mut("setWorkingDirectory", |_, _, path: String| {
			std::env::set_current_dir(std::path::Path::new(&path))?;
			Ok(())
		});

		methods.add_method_mut("ok", |_, data, ()| {
			check_lock_poison(data);
			let LuaData(data) = data;
			Ok(data.is_some())
		});

		methods.add_method("getSampleRate", |_, LuaData(data), ()| {
			if let Some(ud) = data {
				return Ok(Some(ud.sample_rate));
			}
			Ok(None)
		});

		methods.add_method_mut(
			"pitch",
			|_, data, (channel_index, pitch, voice): (usize, f32, usize)| {
				if let LuaData(Some(ud)) = data {
					ud.send_message(AudioMessage::Pitch(channel_index - 1, pitch, voice - 1));
				}
				Ok(())
			},
		);

		methods.add_method_mut(
			"pressure",
			|_, data, (channel_index, pressure, voice): (usize, f32, usize)| {
				if let LuaData(Some(ud)) = data {
					ud.send_message(AudioMessage::Pressure(channel_index - 1, pressure, voice - 1));
				}
				Ok(())
			},
		);

		methods.add_method_mut(
			"noteOn",
			|_, data, (channel_index, pitch, vel, voice): (usize, f32, f32, usize)| {
				if let LuaData(Some(ud)) = data {
					ud.send_message(AudioMessage::NoteOn(channel_index - 1, pitch, vel, voice - 1));
				}
				Ok(())
			},
		);

		methods.add_method_mut("noteOff", |_, data, (channel_index, voice): (usize, usize)| {
			if let LuaData(Some(ud)) = data {
				ud.send_message(AudioMessage::NoteOff(channel_index - 1, voice - 1));
			}
			Ok(())
		});

		methods.add_method_mut("sendMute", |_, data, (channel_index, mute): (usize, bool)| {
			if let LuaData(Some(ud)) = data {
				ud.send_message(AudioMessage::Mute(channel_index - 1, mute));
			}
			Ok(())
		});

		methods.add_method_mut(
			"sendParameter",
			|_, data, (channel_index, device_index, index, value): (usize, usize, usize, f32)| {
				if let LuaData(Some(ud)) = data {
					ud.send_message(AudioMessage::Parameter(
						channel_index - 1,
						device_index, // don't need -1 here since device index is 0 for instrument and 1.. for fx
						index - 1,
						value,
					));
				}
				Ok(())
			},
		);

		methods.add_method_mut(
			"bypass",
			|_, data, (channel_index, device_index, bypass): (usize, usize, bool)| {
				if let LuaData(Some(ud)) = data {
					ud.send_message(AudioMessage::Bypass(channel_index, device_index, bypass));
				}
				Ok(())
			},
		);

		methods.add_method_mut(
			"reorderEffect",
			|_, data, (channel_index, old_index, new_index): (usize, usize, usize)| {
				if let LuaData(Some(ud)) = data {
					ud.send_message(AudioMessage::ReorderEffect(
						channel_index - 1,
						old_index - 1,
						new_index - 1,
					));
				}
				Ok(())
			},
		);

		methods.add_method_mut("setRendering", |_, data, rendering: bool| {
			if let LuaData(Some(ud)) = data {
				ud.send_rendering(rendering);
			}
			Ok(())
		});

		methods.add_method("isRendering", |_, data, ()| {
			if let LuaData(Some(ud)) = data {
				Ok(ud.is_rendering)
			} else {
				Ok(true)
			}
		});

		methods.add_method_mut(
			"insertChannel",
			|_, data, (index, instrument_name): (usize, String)| {
				check_lock_poison(data);
				if let LuaData(Some(ud)) = data {
					let mut render = ud.m_render.lock().expect("Failed to get lock.");
					render.insert_channel(index - 1, &instrument_name);
				}
				Ok(())
			},
		);

		methods.add_method_mut("removeChannel", |_, data, index: usize| {
			check_lock_poison(data);
			if let LuaData(Some(ud)) = data {
				let mut render = ud.m_render.lock().expect("Failed to get lock.");
				render.remove_channel(index - 1);
			}
			Ok(())
		});

		methods.add_method_mut(
			"insertEffect",
			|_, data, (channel_index, effect_index, name): (usize, usize, String)| {
				check_lock_poison(data);
				if let LuaData(Some(ud)) = data {
					let mut render = ud.m_render.lock().expect("Failed to get lock.");
					render.insert_effect(channel_index - 1, effect_index - 1, &name);
				}
				Ok(())
			},
		);

		methods.add_method_mut(
			"removeEffect",
			|_, data, (channel_index, effect_index): (usize, usize)| {
				check_lock_poison(data);
				if let LuaData(Some(ud)) = data {
					let mut render = ud.m_render.lock().expect("Failed to get lock.");
					render.remove_effect(channel_index - 1, effect_index - 1);
				}
				Ok(())
			},
		);

		methods.add_method_mut("renderBlock", |_, data, ()| {
			check_lock_poison(data);
			if let LuaData(Some(ud)) = data {
				let len = 64;
				let buffer: &mut [&mut [f32]; 2] = &mut [&mut vec![0.0; len], &mut vec![0.0; len]];

				let mut render = ud.m_render.lock().expect("Failed to get lock.");
				// TODO: need to check here if the stream is *actually* paused

				render.parse_messages();
				unsafe { no_denormals(|| render.process(buffer)) };

				// interlace
				for i in 0..len {
					ud.render_buffer.push(buffer[0][i]);
					ud.render_buffer.push(buffer[1][i]);
				}
				Ok(true)
			} else {
				Ok(false)
			}
		});

		methods.add_method_mut("renderFinish", |_, data, ()| {
			if let LuaData(Some(ud)) = data {
				let filename = "../out/render.wav";
				let sample_rate = ud.sample_rate;

				match write_wav(filename, &ud.render_buffer, sample_rate) {
					Ok(()) => {
						log_info!("Wrote \"{filename}\".");
					},
					Err(e) => {
						log_error!("Failed to write wav!");
						log_error!("{e}");
					},
				}
				// reset the buffer
				ud.render_buffer = Vec::new();
			} else {
				log_error!("Failed to write wav, backend offline.");
			}
			Ok(())
		});

		methods.add_method_mut("renderCancel", |_, data, ()| {
			if let LuaData(Some(ud)) = data {
				ud.render_buffer = Vec::new();
			}
			Ok(())
		});

		methods.add_method_mut("flush", |_, data, ()| {
			check_lock_poison(data);
			if let LuaData(Some(ud)) = data {
				let mut render = ud.m_render.lock().expect("Failed to get lock.");
				render.flush();
			}
			Ok(())
		});

		methods.add_method_mut("updateScope", |_, data, ()| {
			if let LuaData(Some(ud)) = data {
				ud.scope.update();
			}
			Ok(())
		});
		methods.add_method("getSpectrum", |_, data, ()| {
			if let LuaData(Some(ud)) = data {
				Ok(Some(ud.scope.get_spectrum()))
			} else {
				Ok(None)
			}
		});
		methods.add_method("getScope", |_, data, ()| {
			if let LuaData(Some(ud)) = data {
				Ok(Some(ud.scope.get_oscilloscope()))
			} else {
				Ok(None)
			}
		});

		methods.add_method_mut("pop", |_, data, ()| {
			if let LuaData(Some(ud)) = data {
				Ok(ud.lua_rx.try_pop())
			} else {
				Ok(None)
			}
		});

		methods.add_method("midiPorts", |_, _, ()| {
			let list = midi::port_names();
			Ok(list)
		});

		methods.add_method_mut("midiOpenConnection", |_, data, port_name: String| {
			if let LuaData(Some(ud)) = data {
				let connection = midi::connect(&port_name);
				if let Some(c) = connection {
					let name = c.name.clone();
					let index = ud.midi_connections.len() + 1;
					ud.midi_connections.push(c);
					return Ok((Some(name), Some(index)));
				}
			}
			Ok((None, None))
		});

		methods.add_method_mut("midiCloseConnection", |_, data, connection_index: usize| {
			if let LuaData(Some(ud)) = data {
				if ud.midi_connections.len() < connection_index - 1 {
					log_error!("Bad midi connection index: {connection_index}");
				} else {
					let connection = ud.midi_connections.remove(connection_index - 1);
					connection.connection.close();
					log_info!("Closed connection \"{0}\"", connection.name);
				}
			}
			Ok(())
		});

		methods.add_method_mut("midiPoll", |_, data, connection_index: usize| {
			if let LuaData(Some(ud)) = data {
				let connection = ud.midi_connections.get_mut(connection_index - 1);
				match connection {
					Some(c) => {
						let events: Vec<midi::Event> = c.midi_rx.pop_iter().collect();
						return Ok(Some(events));
					},
					None => {
						log_error!("Bad midi connection index: {connection_index}");
					},
				}
			}
			Ok(None)
		});
	}
}

#[mlua::lua_module]
fn tessera(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;
	exports.set("init", lua.create_function(init)?)?;
	Ok(exports)
}

fn write_wav(filename: &str, samples: &[f32], sample_rate: u32) -> Result<(), Box<dyn Error>> {
	let spec = hound::WavSpec {
		channels: 2,
		sample_rate,
		bits_per_sample: 16,
		sample_format: hound::SampleFormat::Int,
	};

	let mut writer = hound::WavWriter::create(filename, spec)?;
	for s in samples {
		writer.write_sample(convert_sample_wav(*s))?;
	}
	writer.finalize()?;

	Ok(())
}

fn convert_sample_wav(x: f32) -> i16 {
	// TPDF dither in range [-1, 1] quantization levels
	let dither = (fastrand::f32() - fastrand::f32()) / f32::from(u16::MAX);
	let x = (x + dither).clamp(-1.0, 1.0);
	(if x >= 0.0 { x * f32::from(i16::MAX) } else { -x * f32::from(i16::MIN) }) as i16
}

fn check_lock_poison(data: &mut LuaData) {
	if let LuaData(Some(ud)) = data {
		if ud.m_render.is_poisoned() {
			log_error!("Lock was poisoned. Killing backend.");
			*data = LuaData(None);
		}
	}
}

impl AudioContext {
	fn send_message(&mut self, m: AudioMessage) {
		if self.audio_tx.try_push(m).is_err() {
			log_warn!("Queue full. Dropped message!");
		}
	}

	fn send_rendering(&mut self, is_rendering: bool) {
		self.is_rendering = is_rendering;
		if self.stream_tx.try_push(is_rendering).is_err() {
			log_warn!("Stream queue full. Dropped message!");
		}
	}
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
