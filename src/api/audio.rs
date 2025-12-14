use crate::app::State;
use crate::audio;
use crate::audio::{
	check_architecture, get_default_host, get_default_output_device, get_hosts, get_output_devices,
	open_control_panel,
};
use crate::context::{AudioContext, AudioMessage};
use crate::log::{log_error, log_info};
use crate::voice_manager::Token;
use mlua::prelude::*;
use no_denormals::no_denormals;
use ringbuf::traits::*;

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
	let audio = lua.create_table()?;

	audio.set("get_hosts", lua.create_function(|_, ()| Ok(get_hosts()))?)?;
	audio.set("get_default_host", lua.create_function(|_, ()| Ok(get_default_host()))?)?;

	audio.set(
		"get_output_devices",
		lua.create_function(|_, host_name: String| match get_output_devices(&host_name) {
			Ok(devices) => Ok(devices),
			Err(e) => {
				log_error!("{e}");
				Ok(vec![])
			},
		})?,
	)?;

	audio.set(
		"get_default_output_device",
		lua.create_function(|_, host_name: String| match get_default_output_device(&host_name) {
			Ok(name) => Ok(Some(name)),
			Err(e) => {
				log_error!("{e}");
				Ok(None)
			},
		})?,
	)?;

	audio.set(
		"setup",
		lua.create_function(
			|lua, (host_name, device_name, buffer_size): (String, String, Option<u32>)| {
				if let Err(e) = check_architecture() {
					return Err(mlua::Error::RuntimeError(e.to_string()));
				}

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
				let state = &mut *lua.app_data_mut::<State>().unwrap();
				if let Some(ctx) = &mut state.audio
					&& let Err(e) = ctx.rebuild_stream(&host_name, &device_name, buffer_size)
				{
					state.audio = None;
					log_error!("{e}");
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
		"panic",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::Panic);
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"ok",
		lua.create_function(|lua, ()| Ok(lua.app_data_mut::<State>().unwrap().audio.is_some()))?,
	)?;

	audio.set(
		"get_token",
		lua.create_function(|lua, ()| {
			let state = &mut lua.app_data_mut::<State>().unwrap();
			Ok(state.next_token())
		})?,
	)?;

	audio.set(
		"open_control_panel",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio
				&& let Some(device) = &ctx.device
			{
				open_control_panel(device);
			}
			Ok(())
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
				let (meter_handle, meter_id) = ctx.meters.register();
				let mut render = ctx.render.lock();
				render.insert_channel(index - 1, &instrument_name, meter_handle);
				Ok(Some(meter_id + 1))
			} else {
				Ok(None)
			}
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
				let (meter_handle, meter_id) = ctx.meters.register();
				let mut render = ctx.render.lock();
				render.insert_effect(channel_index - 1, effect_index - 1, &name, meter_handle);
				Ok(Some(meter_id + 1))
			} else {
				Ok(None)
			}
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
		"get_meters",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let meters = ctx.meters.collect();
				Ok(Some(meters))
			} else {
				Ok(None)
			}
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

	audio.set(
		"pop_error",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				match ctx.error_rx.try_pop() {
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
