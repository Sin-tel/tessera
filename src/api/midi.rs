use crate::app::State;
use crate::log::{log_error, log_info};
use crate::midi;
use mlua::{UserData, UserDataMethods};
use ringbuf::traits::*;

pub struct Midi;

impl UserData for Midi {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_function("ports", |_, ()| {
			let list = midi::port_names();
			Ok(list)
		});

		methods.add_function("open_connection", |lua, port_name: String| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				let connection = midi::connect(&port_name);
				if let Some(c) = connection {
					let name = c.name.clone();
					let index = ctx.midi_connections.len() + 1;
					ctx.midi_connections.push(c);
					return Ok((Some(name), Some(index)));
				}
			}
			Ok((None, None))
		});

		methods.add_function("close_connection", |lua, connection_index: usize| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
				if ctx.midi_connections.len() < connection_index - 1 {
					log_error!("Bad midi connection index: {connection_index}");
				} else {
					let connection = ctx.midi_connections.remove(connection_index - 1);
					connection.connection.close();
					log_info!("Closed connection \"{0}\"", connection.name);
				}
			}
			Ok(())
		});

		methods.add_function("poll", |lua, connection_index: usize| {
			if let Some(ctx) = lua.app_data_mut::<State>().unwrap().audio_mut() {
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
		});
	}
}
