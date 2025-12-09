use crate::app::State;
use crate::log::{log_error, log_info};
use crate::midi;
use mlua::prelude::*;
use ringbuf::traits::*;

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
	let midi = lua.create_table()?;

	midi.set(
		"init",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				if ctx.midi_session.is_some() {
					log_error!("Midi already initialized");
					return Ok(false);
				}
				ctx.midi_session = midi::open_midi();
				return Ok(ctx.midi_session.is_some());
			}
			Ok(false)
		})?,
	)?;

	midi.set(
		"ports",
		lua.create_function(|lua, ()| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio
				&& let Some(midi_session) = &ctx.midi_session
			{
				let list = midi::port_names(midi_session);
				return Ok(list);
			}
			Ok(vec![])
		})?,
	)?;

	midi.set(
		"open_connection",
		lua.create_function(|lua, port_name: String| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let connection = midi::connect(&port_name);
				if let Some(c) = connection {
					let index = ctx.midi_connections.len() + 1;
					ctx.midi_connections.push(c);
					return Ok(Some(index));
				}
			}
			Ok(None)
		})?,
	)?;

	midi.set(
		"close_connection",
		lua.create_function(|lua, port_name: String| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let index = ctx.midi_connections.iter().position(|v| v.name == port_name);

				if let Some(index) = index {
					let connection = ctx.midi_connections.remove(index);
					connection.connection.close();
					log_info!("Closed connection \"{}\"", connection.name);
					return Ok(Some(index + 1));
				}
			}
			Ok(None)
		})?,
	)?;

	midi.set(
		"poll",
		lua.create_function(|lua, connection_index: usize| {
			if let Some(ctx) = &mut lua.app_data_mut::<State>().unwrap().audio {
				let connection = ctx.midi_connections.get_mut(connection_index - 1);
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
		})?,
	)?;

	Ok(midi)
}
