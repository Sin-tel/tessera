use crate::log::{log_error, log_info, log_warn};
use midir::{Ignore, MidiInput, MidiInputConnection};
use mlua::Value;
use mlua::prelude::*;
use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd, HeapRb};

// TODO: some kind of de-duplication logic in case we have two ports with the same name. (port.id() is unreliable)

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
	pub connection: MidiInputConnection<HeapProd<Event>>,
	pub midi_rx: HeapCons<Event>,
	pub name: String,
}

// Create a midi input "session" and keep it alive for querying port_names
pub fn open_midi() -> Option<MidiInput> {
	match MidiInput::new("midir input") {
		Ok(midi_in) => Some(midi_in),
		Err(e) => {
			log_error!("{e}");
			None
		},
	}
}

pub fn port_names(midi_in: &MidiInput) -> Vec<String> {
	let ports = midi_in.ports();
	ports.iter().map(|p| midi_in.port_name(p).unwrap()).collect()
}

pub fn connect(port_name: &str) -> Option<Connection> {
	if let Some(mut midi_in) = open_midi() {
		// ignore sysex and such
		midi_in.ignore(Ignore::All);

		for p in &midi_in.ports() {
			let name = midi_in.port_name(p).unwrap();

			if name == port_name {
				let (midi_tx, midi_rx) = HeapRb::<Event>::new(256).split();

				let connect_result = midi_in.connect(
					p,
					"midir-test",
					|_stamp, message, midi_rx| {
						let event = Event::from_bytes(message);
						if let Some(e) = event
							&& midi_rx.try_push(e).is_err()
						{
							log_warn!("Midi queue full!");
						}
					},
					midi_tx,
				);

				match connect_result {
					Ok(connection) => {
						log_info!("Opened midi port \"{name}\".");
						return Some(Connection { connection, midi_rx, name });
					},
					Err(err) => {
						log_error!("Failed to open midi port \"{port_name}\".");
						log_error!("\t{err}");
						return None;
					},
				}
			}
		}
	}
	None
}

impl Event {
	pub fn from_bytes(data: &[u8]) -> Option<Self> {
		use Message::*;
		if data.len() < 2 || data.len() > 3 {
			log_error!("Malformed midi message: {:?}", data);
			return None;
		}

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
			12 => return None, // Program change
			13 => Pressure(f32::from(a) / 127.0),
			14 => PitchBend((i32::from(a) + i32::from(b) * 128 - 8192) as f32 / 8192.0),
			s => {
				log_warn!("Unparsed midi event ({s}): {data:?}");
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
