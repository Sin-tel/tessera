use crate::audio;
use crate::defs::*;
use crate::render;
use crate::scope::Scope;
use mlua::prelude::*;
use mlua::{UserData, UserDataMethods, Value};
use ringbuf::{Consumer, Producer};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

impl<'lua> ToLua<'lua> for LuaMessage {
	fn to_lua(self, lua: &'lua Lua) -> LuaResult<Value<'lua>> {
		let table = Lua::create_table(lua)?;

		use LuaMessage::*;

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

// big struct that lua side holds ptr to
pub struct StreamUserData {
	pub stream: cpal::Stream,
	pub audio_tx: Producer<AudioMessage>,
	pub stream_tx: Producer<bool>,
	pub lua_rx: Consumer<LuaMessage>,
	pub m_render: Arc<Mutex<render::Render>>,
	pub scope: Scope,
}

impl Drop for StreamUserData {
	fn drop(&mut self) {
		println!("Stream dropped")
	}
}

fn stream_new(
	_: &Lua,
	(host_name, device_name): (String, String),
) -> LuaResult<Option<Rc<RefCell<StreamUserData>>>> {
	match audio::run(&host_name, &device_name) {
		Ok(ud) => Ok(Some(Rc::new(RefCell::new(ud)))),
		Err(e) => {
			println!("{:}", e.to_string());
			Ok(None)
		}
	}
}

impl UserData for StreamUserData {
	// fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
	// 	fields.add_field_method_get("paused", |_, this| Ok(this.paused));
	// }

	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method("test", |_, _, _: ()| {
			println!("hello!");
			Ok(())
		});

		methods.add_method_mut("send_cv", |_, ud, (ch, pitch, vel): (usize, f32, f32)| {
			send_message(ud, AudioMessage::CV(ch, pitch, vel));
			Ok(())
		});

		methods.add_method_mut(
			"send_note_on",
			|_, ud, (ch, pitch, vel, id): (usize, f32, f32, usize)| {
				send_message(ud, AudioMessage::Note(ch, pitch, vel, id));
				Ok(())
			},
		);

		methods.add_method_mut("send_pan", |_, ud, (ch, gain, pan): (usize, f32, f32)| {
			send_message(ud, AudioMessage::Pan(ch, gain, pan));
			Ok(())
		});

		methods.add_method_mut("send_mute", |_, ud, (ch, mute): (usize, bool)| {
			send_message(ud, AudioMessage::Mute(ch, mute));
			Ok(())
		});

		methods.add_method_mut(
			"send_param",
			|_, ud, (ch_index, device_index, index, value): (usize, usize, usize, f32)| {
				send_message(
					ud,
					AudioMessage::SetParam(ch_index, device_index, index, value),
				);
				Ok(())
			},
		);

		methods.add_method_mut("play", |_, ud, _: ()| {
			send_paused(ud, false);
			Ok(())
		});

		methods.add_method_mut("pause", |_, ud, _: ()| {
			send_paused(ud, true);
			Ok(())
		});

		methods.add_method_mut("add_channel", |_, ud, instrument_number: usize| {
			let mut render = ud.m_render.lock().expect("Failed to get lock.");
			render.add_channel(instrument_number);
			Ok(())
		});

		methods.add_method_mut(
			"add_effect",
			|_, ud, (channel_index, effect_number): (usize, usize)| {
				let mut render = ud.m_render.lock().expect("Failed to get lock.");
				render.add_effect(channel_index, effect_number);
				Ok(())
			},
		);

		methods.add_method_mut("render_block", |_, ud, _: ()| {
			let mut render = ud.m_render.lock().expect("Failed to get lock.");
			let len = 64;
			let buffer: &mut [&mut [f32]; 2] = &mut [&mut vec![0.0; len], &mut vec![0.0; len]];
			let mut out_buffer = vec![0.0f64; len * 2];
			render.parse_messages();
			render.process(buffer);
			// interlace and convert to i16 as f64 (lua wants doubles anyway)
			for (i, outsample) in out_buffer.chunks_exact_mut(2).enumerate() {
				outsample[0] = convert_sample_wav(buffer[0][i]);
				outsample[1] = convert_sample_wav(buffer[1][i]);
			}
			Ok(out_buffer)
		});

		methods.add_method_mut("get_spectrum", |_, ud, _: ()| {
			ud.scope.update();

			let spectrum = ud.scope.get_spectrum();
			Ok(spectrum)
		});

		methods.add_method_mut("rx_is_empty", |_, ud, _: ()| Ok(ud.lua_rx.is_empty()));

		methods.add_method_mut("rx_pop", |_, ud, _: ()| Ok(ud.lua_rx.pop()));
	}
}

#[mlua::lua_module]
fn rust_backend(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;
	exports.set("stream_new", lua.create_function(stream_new)?)?;
	Ok(exports)
}

#[inline]
fn dither() -> f64 {
	// Don't know if this is the correct scaling for dithering, but it sounds good
	(fastrand::f64() - fastrand::f64()) / (2.0 * f64::from(i16::MAX))
}

#[inline]
fn convert_sample_wav(x: f32) -> f64 {
	let z = (f64::from(x) + dither()).clamp(-1.0, 1.0);

	if z >= 0.0 {
		z * f64::from(i16::MAX)
	} else {
		-z * f64::from(i16::MIN)
	}
}

fn send_message(ud: &mut StreamUserData, m: AudioMessage) {
	if ud.audio_tx.push(m).is_err() {
		println!("Queue full. Dropped message!");
	}
}

fn send_paused(ud: &mut StreamUserData, paused: bool) {
	if ud.stream_tx.push(paused).is_err() {
		println!("Stream queue full. Dropped message!");
	}
}
