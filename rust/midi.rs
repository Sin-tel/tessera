use crate::log::{log_info, log_warn};
use midir::{Ignore, MidiInput, MidiInputConnection};
use mlua::prelude::*;
use mlua::Value;
use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd, HeapRb};

#[derive(Debug)]
pub struct Event {
	pub channel: u8,
	pub message: Message,
}

#[derive(Debug)]
pub enum Message {
	NoteOff { note: u8 },
	NoteOn { note: u8, vel: f32 },
	Controller { controller: u8, value: f32 },
	Pressure(f32),
	PitchBend(f32),
}

pub struct Connection {
	#[allow(dead_code)] // must be kept alive
	connection: MidiInputConnection<HeapProd<Event>>,

	pub midi_rx: HeapCons<Event>,
	pub name: String,
}

pub fn list_ports() {
	let midi_in = MidiInput::new("midir test input").unwrap();
	let ports = midi_in.ports();

	if ports.is_empty() {
		log_info!("No midi input ports available.");
	} else {
		log_info!("Available midi input ports:");
		for (i, p) in ports.iter().enumerate() {
			let name = midi_in.port_name(p).unwrap();
			log_info!(" - {i}: \"{name}\"");
		}
	}
}

pub fn connect(port_name: &str) -> Option<Connection> {
	let mut midi_in = MidiInput::new("midir input").unwrap();
	// ignore sysex and such
	midi_in.ignore(Ignore::All);

	for p in &midi_in.ports() {
		let name = midi_in.port_name(p).unwrap();

		if name.to_lowercase().contains(&port_name.to_lowercase()) {
			let (midi_tx, midi_rx) = HeapRb::<Event>::new(256).split();

			let connection = midi_in
				.connect(
					p,
					"midir-test",
					|_stamp, message, midi_rx| {
						// println!("{message:?}");
						let event = Event::from_bytes(message);
						if let Some(e) = event {
							if midi_rx.try_push(e).is_err() {
								log_warn!("Midi queue full!");
							}
						}
					},
					midi_tx,
				)
				.unwrap();

			log_info!("Succesfully opened midi port \"{name}\".");
			return Some(Connection { connection, midi_rx, name });
		}
	}

	log_warn!("Failed to open midi port \"{port_name}\".");
	None
}

impl Event {
	pub fn from_bytes(data: &[u8]) -> Option<Self> {
		use Message::*;
		assert!(data.len() == 2 || data.len() == 3);

		let status: u8 = data[0] >> 4;
		let channel: u8 = data[0] & 0x0f;

		let a = data[1];
		let mut b = 0;

		if data.len() > 2 {
			b = data[2];
		}

		let message = match status {
			8 => NoteOff { note: a },
			9 => {
				if b == 0 {
					NoteOff { note: a }
				} else {
					NoteOn { note: a, vel: f32::from(b) / 127.0 }
				}
			},
			11 => Controller { controller: a, value: f32::from(b) / 127.0 },
			13 => Pressure(f32::from(a)),
			14 => PitchBend((i32::from(a) + i32::from(b) * 128 - 8192) as f32 / 8192.0),
			_ => {
				log_warn!("Unparsed midi event: {data:?}");
				return None;
			},
		};

		// println!("{channel:?} {message:?}");

		Some(Event { channel, message })
	}
}

impl IntoLua for Event {
	fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
		use Message::*;
		let table = Lua::create_table(lua)?;
		table.set("channel", self.channel)?;

		match self.message {
			NoteOn { note, vel } => {
				table.set("name", "note_on")?;
				table.set("note", note)?;
				table.set("vel", vel)?;
			},
			NoteOff { note } => {
				table.set("name", "note_off")?;
				table.set("note", note)?;
			},
			Controller { controller, value } => {
				table.set("name", "controller")?;
				table.set("controller", controller)?;
				table.set("value", value)?;
			},
			Pressure(p) => {
				table.set("name", "pressure")?;
				table.set("pressure", p)?;
			},
			PitchBend(p) => {
				table.set("name", "pitchbend")?;
				table.set("pitchbend", p)?;
			},
		}

		Ok(Value::Table(table))
	}
}
