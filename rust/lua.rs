use mlua::prelude::*;
use mlua::{UserData, UserDataMethods, Value};
use ringbuf::{HeapConsumer, HeapProducer};
use std::sync::{Arc, Mutex};

use crate::audio;
use crate::render;
use crate::scope::Scope;

struct LuaData(Option<AudioContext>);

pub struct AudioContext {
	pub stream: cpal::Stream,
	pub audio_tx: HeapProducer<AudioMessage>,
	pub stream_tx: HeapProducer<bool>,
	pub lua_rx: HeapConsumer<LuaMessage>,
	pub m_render: Arc<Mutex<render::Render>>,
	pub scope: Scope,
	pub paused: bool,
}

// Message struct to pass to audio thread
// Should not contain any boxed values
#[derive(Debug)]
pub enum AudioMessage {
	CV(usize, f32, f32),
	Note(usize, f32, f32, usize),
	Parameter(usize, usize, usize, f32),
	Mute(usize, bool),
	// Bypass(usize, usize, bool),
	// Swap(?),
	//
}

#[derive(Debug)]
pub enum LuaMessage {
	Cpu(f32),
	Meter(f32, f32),
}

impl Drop for AudioContext {
	fn drop(&mut self) {
		println!("Stream dropped");
	}
}

fn init(_: &Lua, _: ()) -> LuaResult<LuaData> {
	Ok(LuaData(None))
}

impl UserData for LuaData {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method_mut(
			"setup",
			|_, data, (host_name, device_name): (String, String)| match audio::run(
				&host_name,
				&device_name,
			) {
				Ok(ud) => {
					*data = LuaData(Some(ud));
					Ok(())
				}
				Err(e) => {
					println!("{e}");
					*data = LuaData(None);
					Ok(())
				}
			},
		);

		methods.add_method_mut("quit", |_, data, _: ()| {
			*data = LuaData(None);
			Ok(())
		});

		methods.add_method_mut("setWorkingDirectory", |_, _, path: String| {
			std::env::set_current_dir(std::path::Path::new(&path))?;
			Ok(())
		});

		methods.add_method("running", |_, LuaData(ud), _: ()| Ok(ud.is_some()));

		methods.add_method_mut("sendCv", |_, data, (ch, pitch, pres): (usize, f32, f32)| {
			if let LuaData(Some(ud)) = data {
				ud.send_message(AudioMessage::CV(ch, pitch, pres));
			}
			Ok(())
		});

		methods.add_method_mut(
			"sendNote",
			|_, data, (ch, pitch, vel, id): (usize, f32, f32, Option<usize>)| {
				if let LuaData(Some(ud)) = data {
					ud.send_message(AudioMessage::Note(ch, pitch, vel, id.unwrap_or(0)));
				}
				Ok(())
			},
		);

		methods.add_method_mut("sendMute", |_, data, (ch, mute): (usize, bool)| {
			if let LuaData(Some(ud)) = data {
				ud.send_message(AudioMessage::Mute(ch, mute));
			}
			Ok(())
		});

		methods.add_method_mut(
			"sendParameter",
			|_, data, (ch_index, device_index, index, value): (usize, usize, usize, f32)| {
				if let LuaData(Some(ud)) = data {
					ud.send_message(AudioMessage::Parameter(
						ch_index,
						device_index,
						index,
						value,
					));
				}
				Ok(())
			},
		);

		methods.add_method_mut("setPaused", |_, data, paused: bool| {
			if let LuaData(Some(ud)) = data {
				ud.send_paused(paused);
			}
			Ok(())
		});

		methods.add_method("paused", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				Ok(ud.paused)
			} else {
				Ok(true)
			}
		});

		methods.add_method("addChannel", |_, data, instrument_number: usize| {
			if let LuaData(Some(ud)) = data {
				let mut render = ud.m_render.lock().expect("Failed to get lock.");
				render.add_channel(instrument_number);
			}
			Ok(())
		});

		methods.add_method(
			"addEffect",
			|_, data, (channel_index, effect_number): (usize, usize)| {
				if let LuaData(Some(ud)) = data {
					let mut render = ud.m_render.lock().expect("Failed to get lock.");
					render.add_effect(channel_index, effect_number);
				}
				Ok(())
			},
		);

		methods.add_method("renderBlock", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				let len = 64;
				let buffer: &mut [&mut [f32]; 2] = &mut [&mut vec![0.0; len], &mut vec![0.0; len]];
				let mut out_buffer = vec![0.0f64; len * 2];

				let mut render = ud.m_render.lock().expect("Failed to get lock.");
				// TODO: need to check here if the stream is *actually* paused

				render.parse_messages();
				let res = render.process(buffer);
				if let Err(e) = res {
					// return Err(LuaError::RuntimeError(e));
					eprintln!("{e:?}");
					return Ok(None);
				}
				// interlace and convert to i16 as f64 (lua wants doubles anyway)
				for (i, outsample) in out_buffer.chunks_exact_mut(2).enumerate() {
					outsample[0] = convert_sample_wav(buffer[0][i]);
					outsample[1] = convert_sample_wav(buffer[1][i]);
				}
				Ok(Some(out_buffer))
			} else {
				Ok(None)
			}
		});

		methods.add_method_mut("updateScope", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				ud.scope.update();
			}
			Ok(())
		});
		methods.add_method("getSpectrum", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				Ok(Some(ud.scope.get_spectrum()))
			} else {
				Ok(None)
			}
		});
		methods.add_method("getScope", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				Ok(Some(ud.scope.get_oscilloscope()))
			} else {
				Ok(None)
			}
		});

		methods.add_method_mut("pop", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				Ok(ud.lua_rx.pop())
			} else {
				Ok(None)
			}
		});
	}
}

#[mlua::lua_module]
fn rust_backend(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;
	exports.set("init", lua.create_function(init)?)?;
	Ok(exports)
}

fn dither() -> f64 {
	// Don't know if this is the correct scaling for dithering, but it sounds good
	(fastrand::f64() - fastrand::f64()) / (2.0 * f64::from(i16::MAX))
}

fn convert_sample_wav(x: f32) -> f64 {
	let z = (f64::from(x) + dither()).clamp(-1.0, 1.0);

	if z >= 0.0 {
		z * f64::from(i16::MAX)
	} else {
		-z * f64::from(i16::MIN)
	}
}

impl AudioContext {
	fn send_message(&mut self, m: AudioMessage) {
		if self.audio_tx.push(m).is_err() {
			println!("Queue full. Dropped message!");
		}
	}

	fn send_paused(&mut self, paused: bool) {
		self.paused = paused;
		if self.stream_tx.push(paused).is_err() {
			println!("Stream queue full. Dropped message!");
		}
	}
}

impl<'lua> ToLua<'lua> for LuaMessage {
	fn to_lua(self, lua: &'lua Lua) -> LuaResult<Value<'lua>> {
		use LuaMessage::*;

		let table = Lua::create_table(lua)?;

		match self {
			Cpu(cpu_load) => {
				table.set("tag", "cpu")?;
				table.set("cpu_load", cpu_load)?;
			}
			Meter(l, r) => {
				table.set("tag", "meter")?;
				table.set("l", l)?;
				table.set("r", r)?;
			}
		}

		Ok(Value::Table(table))
	}
}
