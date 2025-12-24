use crate::embed::Script;
use crate::embed::setup_lua_loader;
use mlua::prelude::*;
use std::sync::mpsc::Sender;

pub enum ProcessorMessage {
	Midi(Vec<u8>),
	UpdateTuning(String),
	UpdateSequence(Vec<u8>),
	Transport(bool),
	Tick(f32),
}

pub enum ProcessorFeedback {
	NoteRecorded(f32, f32), // Pitch, Velocity (to draw on canvas)
}

pub struct Processor {
	tx: Sender<ProcessorMessage>,
}

impl Processor {
	pub fn new() -> Self {
		let (tx, rx) = std::sync::mpsc::channel();

		std::thread::Builder::new()
			.name("engine".to_string())
			.spawn(move || {
				let lua = Lua::new();

				setup_lua_loader(&lua).unwrap();

				let engine = lua.create_table().unwrap();
				lua.globals().set("engine", engine).unwrap();

				let lua_main = &*Script::get("processor.lua")
					.ok_or("processor.lua not found.")
					.unwrap()
					.data;
				lua.load(lua_main).set_name("@lua/processor.lua").exec().unwrap();

				let engine: LuaTable = lua.globals().get("engine").unwrap();
				let load: LuaFunction = engine.get("load").unwrap();
				let update: LuaFunction = engine.get("update").unwrap();

				load.call::<()>(()).unwrap();

				while let Ok(msg) = rx.recv() {
					match msg {
						ProcessorMessage::Tick(dt) => {
							update.call::<()>(dt).unwrap();
						},
						_ => todo!(),
					}
				}
			})
			.unwrap();

		Self { tx }
	}

	pub fn send(&self, msg: ProcessorMessage) {
		self.tx.send(msg).unwrap();
	}
}
