use crate::app::State;
use crate::audio;
use crate::context::AudioMessage;
use crate::log::{log_error, log_info};
use mlua::prelude::*;
use mlua::{UserData, UserDataMethods};
use no_denormals::no_denormals;
use ringbuf::traits::*;
use std::error::Error;

pub struct Backend;

impl Backend {
	pub fn register(lua: &mut Lua) -> LuaResult<()> {
		log_info!("Backend initialized");
		lua.globals().set("backend", Backend {})?;
		Ok(())
	}
}

impl UserData for Backend {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_function(
			"setup",
			|lua, (host_name, device_name, buffer_size): (String, String, Option<u32>)| {
				let state = &mut *lua.app_data_mut::<State>().unwrap();
				match audio::run(&host_name, &device_name, buffer_size) {
					Ok(ctx) => {
						state.audio = Some(ctx);
						Ok(())
					},
					Err(e) => {
						log_error!("{e}");
						state.audio = None;
						Ok(())
					},
				}
			},
		);

		methods.add_function("quit", |lua, ()| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			state.audio = None;
			Ok(())
		});

		methods.add_function("panic", |lua, ()| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				ctx.send_message(AudioMessage::Panic);
			}
			Ok(())
		});

		methods.add_function("isRelease", |_, ()| {
			#[cfg(debug_assertions)]
			return Ok(false);

			#[cfg(not(debug_assertions))]
			return Ok(true);
		});

		methods.add_function("ok", |lua, ()| {
			Ok(lua.app_data_mut::<State>().unwrap().audio_mut().is_some())
		});

		methods.add_function("getSampleRate", |lua, ()| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				return Ok(Some(ctx.sample_rate));
			}
			Ok(None)
		});

		methods.add_function("pitch", |lua, (channel_index, pitch, voice): (usize, f32, usize)| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				ctx.send_message(AudioMessage::Pitch(channel_index - 1, pitch, voice - 1));
			}
			Ok(())
		});

		methods.add_function(
			"pressure",
			|lua, (channel_index, pressure, voice): (usize, f32, usize)| {
				if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
					ctx.send_message(AudioMessage::Pressure(
						channel_index - 1,
						pressure,
						voice - 1,
					));
				}
				Ok(())
			},
		);

		methods.add_function(
			"noteOn",
			|lua, (channel_index, pitch, vel, voice): (usize, f32, f32, usize)| {
				if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
					ctx.send_message(AudioMessage::NoteOn(
						channel_index - 1,
						pitch,
						vel,
						voice - 1,
					));
				}
				Ok(())
			},
		);

		methods.add_function("noteOff", |lua, (channel_index, voice): (usize, usize)| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				ctx.send_message(AudioMessage::NoteOff(channel_index - 1, voice - 1));
			}
			Ok(())
		});

		methods.add_function("sendMute", |lua, (channel_index, mute): (usize, bool)| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				ctx.send_message(AudioMessage::Mute(channel_index - 1, mute));
			}
			Ok(())
		});

		methods.add_function(
			"sendParameter",
			|lua, (channel_index, device_index, index, value): (usize, usize, usize, f32)| {
				if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
					ctx.send_message(AudioMessage::Parameter(
						channel_index - 1,
						device_index, // don't need -1 here since device index is 0 for instrument and 1.. for fx
						index - 1,
						value,
					));
				}
				Ok(())
			},
		);

		methods.add_function(
			"bypass",
			|lua, (channel_index, device_index, bypass): (usize, usize, bool)| {
				if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
					ctx.send_message(AudioMessage::Bypass(channel_index, device_index, bypass));
				}
				Ok(())
			},
		);

		methods.add_function(
			"reorderEffect",
			|lua, (channel_index, old_index, new_index): (usize, usize, usize)| {
				if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
					ctx.send_message(AudioMessage::ReorderEffect(
						channel_index - 1,
						old_index - 1,
						new_index - 1,
					));
				}
				Ok(())
			},
		);

		methods.add_function("setRendering", |lua, rendering: bool| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				ctx.send_rendering(rendering);
			}
			Ok(())
		});

		methods.add_function("isRendering", |lua, ()| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				Ok(ctx.is_rendering)
			} else {
				Ok(true)
			}
		});

		methods.add_function("insertChannel", |lua, (index, instrument_name): (usize, String)| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				let mut render = ctx.m_render.lock().expect("Failed to get lock.");
				render.insert_channel(index - 1, &instrument_name);
			}
			Ok(())
		});

		methods.add_function("removeChannel", |lua, index: usize| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				let mut render = ctx.m_render.lock().expect("Failed to get lock.");
				render.remove_channel(index - 1);
			}
			Ok(())
		});

		methods.add_function(
			"insertEffect",
			|lua, (channel_index, effect_index, name): (usize, usize, String)| {
				if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
					let mut render = ctx.m_render.lock().expect("Failed to get lock.");
					render.insert_effect(channel_index - 1, effect_index - 1, &name);
				}
				Ok(())
			},
		);

		methods.add_function(
			"removeEffect",
			|lua, (channel_index, effect_index): (usize, usize)| {
				if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
					let mut render = ctx.m_render.lock().expect("Failed to get lock.");
					render.remove_effect(channel_index - 1, effect_index - 1);
				}
				Ok(())
			},
		);

		methods.add_function("renderBlock", |lua, ()| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				let len = 64;
				let buffer: &mut [&mut [f32]; 2] = &mut [&mut vec![0.0; len], &mut vec![0.0; len]];

				let mut render = ctx.m_render.lock().expect("Failed to get lock.");
				// TODO: need to check here if the stream is *actually* paused

				render.parse_messages();
				unsafe { no_denormals(|| render.process(buffer)) };

				// interlace
				for i in 0..len {
					ctx.render_buffer.push(buffer[0][i]);
					ctx.render_buffer.push(buffer[1][i]);
				}
				Ok(true)
			} else {
				Ok(false)
			}
		});

		methods.add_function("renderFinish", |lua, ()| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				let filename = "../out/render.wav";
				let sample_rate = ctx.sample_rate;

				match write_wav(filename, &ctx.render_buffer, sample_rate) {
					Ok(()) => {
						log_info!("Wrote \"{filename}\".");
					},
					Err(e) => {
						log_error!("Failed to write wav!");
						log_error!("{e}");
					},
				}
				// reset the buffer
				ctx.render_buffer = Vec::new();
			} else {
				log_error!("Failed to write wav, backend offline.");
			}
			Ok(())
		});

		methods.add_function("renderCancel", |lua, ()| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				ctx.render_buffer = Vec::new();
			}
			Ok(())
		});

		methods.add_function("flush", |lua, ()| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				let mut render = ctx.m_render.lock().expect("Failed to get lock.");
				render.flush();
			}
			Ok(())
		});

		methods.add_function("updateScope", |lua, ()| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				ctx.scope.update();
			}
			Ok(())
		});
		methods.add_function("getSpectrum", |lua, ()| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				Ok(Some(ctx.scope.get_spectrum()))
			} else {
				Ok(None)
			}
		});
		methods.add_function("getScope", |lua, ()| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				Ok(Some(ctx.scope.get_oscilloscope()))
			} else {
				Ok(None)
			}
		});

		methods.add_function("pop", |lua, ()| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				Ok(ctx.lua_rx.try_pop())
			} else {
				Ok(None)
			}
		});
	}
}

// fn tessera(lua: &Lua) -> LuaResult<LuaTable> {
// 	let exports = lua.create_table()?;
// 	exports.set("init", lua.create_function(init)?)?;
// 	Ok(exports)
// }

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
