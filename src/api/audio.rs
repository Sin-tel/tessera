use crate::api::lua_serde;
use crate::app::State;
use crate::audio;
use crate::audio::CPU_LOAD;
use crate::audio::{
	check_architecture, get_default_host, get_default_output_device, get_hosts, get_output_devices,
	open_control_panel,
};
use crate::context::{AudioContext, AudioMessage};
use crate::log::{log_error, log_info};
use crate::opengl::UserEvent;
use crate::voice_manager::Token;
use crate::vst3;
use crate::vst3::scan::probe_vst3;
use cpal::Device;
use cpal::traits::DeviceTrait;
use mlua::prelude::*;
use no_denormals::no_denormals;
use ringbuf::traits::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
			Ok(device_info) => Ok(Some(device_info)),
			Err(e) => {
				log_error!("{e}");
				Ok(None)
			},
		})?,
	)?;

	audio.set(
		"setup",
		lua.create_function(|lua, (device_info, buffer_size): (DeviceInfo, Option<u32>)| {
			if let Err(e) = check_architecture() {
				return Err(mlua::Error::RuntimeError(e.to_string()));
			}
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			let lua_tx = state.lua_tx.clone();

			match AudioContext::new(&device_info, buffer_size, lua_tx) {
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
		})?,
	)?;

	audio.set(
		"rebuild",
		lua.create_function(|lua, (device_info, buffer_size): (DeviceInfo, Option<u32>)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			#[allow(clippy::collapsible_if)]
			if let Some(ctx) = &mut state.audio {
				if let Err(e) = ctx.rebuild_stream(&device_info, buffer_size) {
					state.audio = None;
					log_error!("{e}");
				}
			}
			Ok(())
		})?,
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
			|lua, (channel_index, pitch, offset, vel, token): (usize, f32, f32, f32, Token)| {
				if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
					ctx.send_message(AudioMessage::NoteOn(
						channel_index - 1,
						token,
						pitch,
						offset,
						vel,
					));
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
		"send_channel_mute",
		lua.create_function(|lua, (channel_index, mute): (usize, bool)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::ChannelMute(channel_index - 1, mute));
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"send_channel_gain",
		lua.create_function(|lua, (channel_index, gain): (usize, f32)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::ChannelGain(channel_index - 1, gain));
			}
			Ok(())
		})?,
	)?;

	audio.set(
		"send_device_mute",
		lua.create_function(|lua, (channel_index, device_index, mute): (usize, usize, bool)| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::DeviceMute(channel_index - 1, device_index, mute));
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
		"metronome",
		lua.create_function(|lua, accent: bool| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				ctx.send_message(AudioMessage::Metronome(accent));
			}
			Ok(())
		})?,
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
		lua.create_function(|lua, index: usize| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let (meter_handle_channel, meter_id_channel) = ctx.meters.register();
				let mut render = ctx.render.lock();
				render.insert_channel(index - 1, meter_handle_channel);
				Ok(Some(meter_id_channel + 1))
			} else {
				log_error!("Failed to insert channel");
				Ok(None)
			}
		})?,
	)?;

	audio.set(
		"insert_instrument",
		lua.create_function(|lua, (index, instrument_name): (usize, String)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			if let Some(ctx) = &mut state.audio {
				assert!(instrument_name != "vst_wrapper");
				let (meter_handle_instrument, meter_id_instrument) = ctx.meters.register();
				let mut render = ctx.render.lock();
				render.insert_instrument(index - 1, &instrument_name, meter_handle_instrument);
				Ok(Some(meter_id_instrument + 1))
			} else {
				Ok(None)
			}
		})?,
	)?;

	audio.set(
		"insert_instrument_vst",
		lua.create_function(|lua, (index, vst_id, path): (usize, usize, String)| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			if let Some(ctx) = &mut state.audio {
				let mut plugin_info = probe_vst3(&PathBuf::from(path)).unwrap();
				assert!(plugin_info.len() == 1);
				let plugin = plugin_info.pop().unwrap();

				let (editor, processor) = vst3::load(
					&plugin,
					ctx.sample_rate as f32,
					vst_id,
					state.vst_cleanup_tx.clone(),
				)
				.unwrap();

				state.vst_editors.insert(vst_id, editor);

				log_info!("Plugin '{}' loaded succesfully", plugin.name);

				let (meter_handle_instrument, meter_id_instrument) = ctx.meters.register();
				let mut render = ctx.render.lock();
				render.insert_instrument(index - 1, "vst_wrapper", meter_handle_instrument);
				render.set_processor(index - 1, processor);

				Ok(Some(meter_id_instrument + 1))
			} else {
				Ok(None)
			}
		})?,
	)?;

	audio.set(
		"open_vst_window",
		lua.create_function(|lua, vst_id: usize| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			if let Some(editor) = state.vst_editors.get(&vst_id) {
				assert_eq!(editor.id(), vst_id);
				log_info!("Open window '{}' ({})", editor.name(), vst_id);
				if let Err(e) = state.event_loop.send_event(UserEvent::OpenVstWindow(vst_id)) {
					log_error!("{e}");
				}
			} else {
				log_error!("VST id {} not found!", vst_id);
			}

			Ok(())
		})?,
	)?;

	audio.set(
		"poll_vst_destroyed",
		lua.create_function(|lua, ()| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();

			while let Ok(dead_id) = state.vst_cleanup_rx.try_recv() {
				// Processor is already dropped since it triggered this
				state.vst_windows.retain(|_, (id, _)| *id != dead_id);
				state.vst_editors.remove(&dead_id);
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

	audio.set("get_cpu_load", lua.create_function(|_, ()| Ok(CPU_LOAD.load()))?)?;

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
		"pop_error",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				Ok(ctx.error_rx.try_pop())
			} else {
				Ok(None)
			}
		})?,
	)?;

	Ok(audio)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
	pub name: String,
	pub id: String,
}

impl DeviceInfo {
	pub fn from_device(device: &Device) -> anyhow::Result<Self> {
		let description = device.description()?;
		let mut name = description.name().to_string();

		// If available, add driver name.
		// Fixes WASAPI generic names, e.g. replaces "Speakers" with "Speakers (Realtek Audio)"
		// TODO: Submit fix for this in cpal.
		#[cfg(target_os = "windows")]
		if let Some(driver) = description.driver()
			&& driver != name
		{
			name = format!("{name} ({driver})");
		}

		let id = device.id()?.to_string();
		Ok(Self { name, id })
	}
}

lua_serde!(DeviceInfo);
