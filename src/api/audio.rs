use crate::app::State;
use crate::audio;
use crate::audio::check_architecture;
use crate::context::{AudioContext, AudioMessage};
use crate::log::{log_error, log_info};
use crate::voice_manager::Token;
use mlua::prelude::*;
use no_denormals::no_denormals;
use ringbuf::traits::*;

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
	let audio = lua.create_table()?;

	audio.set(
		"setup",
		lua.create_function(
			|lua, (host_name, device_name, buffer_size): (String, String, Option<u32>)| {
				check_architecture().unwrap();

				let state = &mut *lua.app_data_mut::<State>().unwrap();
				match AudioContext::new(&host_name, &device_name, buffer_size) {
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
		)?,
	)?;

	audio.set(
		"rebuild",
		lua.create_function(
			|lua, (host_name, device_name, buffer_size): (String, String, Option<u32>)| {
				#[allow(clippy::collapsible_if)]
				if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
					if let Err(e) = ctx.rebuild_stream(&host_name, &device_name, buffer_size) {
						log_error!("{e}");
					}
				}
				Ok(())
			},
		)?,
	)?;

	audio.set(
		"quit",
		lua.create_function(|lua, ()| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			state.audio = None;
			Ok(())
		})?,
	)?;

	audio.set(
		"check_should_rebuild",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				Ok(ctx.check_should_rebuild())
			} else {
				Ok(false)
			}
		})?,
	)?;

	audio.set(
		"panic",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::Panic);
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"is_release",
		lua.create_function(|_, ()| {
			#[cfg(debug_assertions)]
			return Ok(false);

			#[cfg(not(debug_assertions))]
			return Ok(true);
		})?,
	)?;

	audio.set(
		"ok",
		lua.create_function(|lua, ()| Ok(lua.app_data_mut::<State>().unwrap().audio.is_some()))?,
	)?;

	audio.set(
		"get_samplerate",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &lua.app_data_ref::<State>().unwrap().audio {
				return Ok(Some(ctx.sample_rate));
			}
			Ok(None)
		})?,
	)?;

	audio.set(
		"get_token",
		lua.create_function(|lua, ()| {
			let state = &mut lua.app_data_mut::<State>().unwrap();
			Ok(state.next_token())
		})?,
	)?;

	audio.set(
		"all_notes_off",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::AllNotesOff);
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"clear_messages",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let mut render = ctx.render.lock();
				render.parse_messages();
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"note_on",
		lua.create_function(
			|lua, (channel_index, pitch, vel, token): (usize, f32, f32, Token)| {
				if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
					ctx.send_message(AudioMessage::NoteOn(channel_index - 1, token, pitch, vel));
				}
				Ok(())
			},
		)?,
	)?;

	audio.set(
		"note_off",
		lua.create_function(|lua, (channel_index, token): (usize, Token)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::NoteOff(channel_index - 1, token));
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"pitch",
		lua.create_function(|lua, (channel_index, pitch, token): (usize, f32, Token)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::Pitch(channel_index - 1, token, pitch));
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"pressure",
		lua.create_function(|lua, (channel_index, pressure, token): (usize, f32, Token)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::Pressure(channel_index - 1, token, pressure));
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"sustain",
		lua.create_function(|lua, (channel_index, sustain): (usize, bool)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::Sustain(channel_index - 1, sustain));
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"send_mute_channel",
		lua.create_function(|lua, (channel_index, mute): (usize, bool)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::MuteChannel(channel_index - 1, mute));
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"send_mute_device",
		lua.create_function(|lua, (channel_index, device_index, mute): (usize, usize, bool)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::MuteDevice(channel_index - 1, device_index, mute));
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"send_parameter",
		lua.create_function(
			|lua, (channel_index, device_index, index, value): (usize, usize, usize, f32)| {
				if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
					ctx.send_message(AudioMessage::Parameter(
						channel_index - 1,
						device_index, // don't need -1 here since device index is 0 for instrument and 1.. for fx
						index - 1,
						value,
					));
				}
				Ok(())
			},
		)?,
	)?;

	audio.set(
		"reorder_effect",
		lua.create_function(
			|lua, (channel_index, old_index, new_index): (usize, usize, usize)| {
				if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
					ctx.send_message(AudioMessage::ReorderEffect(
						channel_index - 1,
						old_index - 1,
						new_index - 1,
					));
				}
				Ok(())
			},
		)?,
	)?;

	audio.set(
		"set_rendering",
		lua.create_function(|lua, rendering: bool| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_rendering(rendering);
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"is_rendering",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				Ok(ctx.is_rendering)
			} else {
				Ok(true)
			}
		})?,
	)?;

	audio.set(
		"insert_channel",
		lua.create_function(|lua, (index, instrument_name): (usize, String)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let mut render = ctx.render.lock();
				render.insert_channel(index - 1, &instrument_name);
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"remove_channel",
		lua.create_function(|lua, index: usize| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let mut render = ctx.render.lock();
				render.remove_channel(index - 1);
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"insert_effect",
		lua.create_function(|lua, (channel_index, effect_index, name): (usize, usize, String)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let mut render = ctx.render.lock();
				render.insert_effect(channel_index - 1, effect_index - 1, &name);
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"remove_effect",
		lua.create_function(|lua, (channel_index, effect_index): (usize, usize)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let mut render = ctx.render.lock();
				render.remove_effect(channel_index - 1, effect_index - 1);
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"render_block",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let len = 64;
				let buffer: &mut [&mut [f32]; 2] = &mut [&mut vec![0.0; len], &mut vec![0.0; len]];

				let mut render = ctx.render.lock();
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
		})?,
	)?;

	audio.set(
		"render_finish",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let filename = "out/render.wav";
				let sample_rate = ctx.sample_rate;

				match audio::write_wav(filename, &ctx.render_buffer, sample_rate) {
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
		})?,
	)?;

	audio.set(
		"render_cancel",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.render_buffer = Vec::new();
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"flush",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let mut render = ctx.render.lock();
				render.flush();
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"update_scope",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.scope.update();
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"pop",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				match ctx.lua_rx.try_pop() {
					Some(p) => Ok(Some(lua.to_value(&p)?)),
					None => Ok(None),
				}
			} else {
				Ok(None)
			}
		})?,
	)?;

	Ok(audio)
}
