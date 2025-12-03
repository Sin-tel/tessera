// #![allow(dead_code)]

use crate::app::State;
use mlua::prelude::*;

#[derive(Debug, Clone)]
pub struct Project {
	pub name: String,
	pub version: Version,
	pub channels: Vec<Channel>,
	pub transport: Transport,
}

#[derive(Debug, Clone)]
pub struct Channel {
	pub name: String,
	pub instrument: Instrument,
	pub notes: Vec<Note>,
	pub control: Control,
	pub effects: Vec<Effect>,
	pub hue: f64,
	pub visible: bool,
	pub mute: bool,
	pub solo: bool,
	pub armed: bool,
	pub lock: bool,
}

#[derive(Debug, Clone)]
pub struct Instrument {
	pub name: String,
	pub state: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct Effect {
	pub name: String,
	pub state: Vec<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct Control {
	pub sustain: Vec<SustainEvent>,
}

#[derive(Debug, Clone)]
pub struct SustainEvent {
	pub time: f64,
	pub value: bool,
}

#[derive(Debug, Clone)]
pub struct Note {
	pub time: f64,
	pub pitch: Vec<i32>,
	pub vel: f64,
	pub verts: Vec<Vertex>,
}

#[derive(Debug, Clone)]
pub struct Vertex {
	pub x: f64,
	pub y: f64,
	pub w: f64,
}

#[derive(Debug, Clone)]
pub struct Transport {
	pub start_time: f64,
	pub recording: bool,
}

#[derive(Debug, Clone)]
pub struct Version {
	pub major: u32,
	pub minor: u32,
	pub patch: u32,
}

impl FromLua for Project {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		let table = value.as_table().ok_or_else(|| LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "Project".into(),
			message: Some("expected table".into()),
		})?;

		Ok(Project {
			name: table.get("name")?,
			version: table.get("VERSION")?,
			channels: table.get("channels")?,
			transport: table.get("transport")?,
		})
	}
}

impl IntoLua for Project {
	fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
		let table = lua.create_table()?;
		table.set("name", self.name)?;
		table.set("VERSION", self.version)?;
		table.set("channels", self.channels)?;
		table.set("transport", self.transport)?;
		Ok(LuaValue::Table(table))
	}
}

impl FromLua for Channel {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		let table = value.as_table().ok_or_else(|| LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "Channel".into(),
			message: Some("expected table".into()),
		})?;

		Ok(Channel {
			name: table.get("name")?,
			instrument: table.get("instrument")?,
			notes: table.get("notes")?,
			control: table.get("control")?,
			effects: table.get("effects")?,
			hue: table.get("hue")?,
			visible: table.get("visible")?,
			mute: table.get("mute")?,
			solo: table.get("solo")?,
			armed: table.get("armed")?,
			lock: table.get("lock")?,
		})
	}
}

impl IntoLua for Channel {
	fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
		let table = lua.create_table()?;
		table.set("name", self.name)?;
		table.set("instrument", self.instrument)?;
		table.set("notes", self.notes)?;
		table.set("control", self.control)?;
		table.set("effects", self.effects)?;
		table.set("hue", self.hue)?;
		table.set("visible", self.visible)?;
		table.set("mute", self.mute)?;
		table.set("solo", self.solo)?;
		table.set("armed", self.armed)?;
		table.set("lock", self.lock)?;
		Ok(LuaValue::Table(table))
	}
}

impl FromLua for Instrument {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		let table = value.as_table().ok_or_else(|| LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "Instrument".into(),
			message: Some("expected table".into()),
		})?;

		Ok(Instrument { name: table.get("name")?, state: table.get("state")? })
	}
}

impl IntoLua for Instrument {
	fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
		let table = lua.create_table()?;
		table.set("name", self.name)?;
		table.set("state", self.state)?;
		Ok(LuaValue::Table(table))
	}
}

impl FromLua for Effect {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		let table = value.as_table().ok_or_else(|| LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "Effect".into(),
			message: Some("expected table".into()),
		})?;

		Ok(Effect { name: table.get("name")?, state: table.get("state")? })
	}
}

impl IntoLua for Effect {
	fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
		let table = lua.create_table()?;
		table.set("name", self.name)?;
		table.set("state", self.state)?;
		Ok(LuaValue::Table(table))
	}
}

impl FromLua for Control {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		let table = value.as_table().ok_or_else(|| LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "Control".into(),
			message: Some("expected table".into()),
		})?;

		Ok(Control { sustain: table.get("sustain").unwrap_or_default() })
	}
}

impl IntoLua for Control {
	fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
		let table = lua.create_table()?;
		table.set("sustain", self.sustain)?;
		Ok(LuaValue::Table(table))
	}
}

impl FromLua for SustainEvent {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		let table = value.as_table().ok_or_else(|| LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "SustainEvent".into(),
			message: Some("expected table".into()),
		})?;

		Ok(SustainEvent { time: table.get("time")?, value: table.get("value")? })
	}
}

impl IntoLua for SustainEvent {
	fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
		let table = lua.create_table()?;
		table.set("time", self.time)?;
		table.set("value", self.value)?;
		Ok(LuaValue::Table(table))
	}
}

impl FromLua for Note {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		let table = value.as_table().ok_or_else(|| LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "Note".into(),
			message: Some("expected table".into()),
		})?;

		Ok(Note {
			time: table.get("time")?,
			pitch: table.get("pitch")?,
			vel: table.get("vel")?,
			verts: table.get("verts")?,
		})
	}
}

impl IntoLua for Note {
	fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
		let table = lua.create_table()?;
		table.set("time", self.time)?;
		table.set("pitch", self.pitch)?;
		table.set("vel", self.vel)?;
		table.set("verts", self.verts)?;
		Ok(LuaValue::Table(table))
	}
}

impl FromLua for Vertex {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		let table = value.as_table().ok_or_else(|| LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "Vertex".into(),
			message: Some("expected table".into()),
		})?;

		Ok(Vertex { x: table.get(1)?, y: table.get(2)?, w: table.get(3)? })
	}
}

impl IntoLua for Vertex {
	fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
		let table = lua.create_table()?;
		table.set(1, self.x)?;
		table.set(2, self.y)?;
		table.set(3, self.w)?;
		Ok(LuaValue::Table(table))
	}
}

impl FromLua for Transport {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		let table = value.as_table().ok_or_else(|| LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "Transport".into(),
			message: Some("expected table".into()),
		})?;

		Ok(Transport { start_time: table.get("start_time")?, recording: table.get("recording")? })
	}
}

impl IntoLua for Transport {
	fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
		let table = lua.create_table()?;
		table.set("start_time", self.start_time)?;
		table.set("recording", self.recording)?;
		Ok(LuaValue::Table(table))
	}
}

impl FromLua for Version {
	fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
		let table = value.as_table().ok_or_else(|| LuaError::FromLuaConversionError {
			from: value.type_name(),
			to: "Version".into(),
			message: Some("expected table".into()),
		})?;

		Ok(Version {
			major: table.get("MAJOR")?,
			minor: table.get("MINOR")?,
			patch: table.get("PATCH")?,
		})
	}
}

impl IntoLua for Version {
	fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
		let table = lua.create_table()?;
		table.set("MAJOR", self.major)?;
		table.set("MINOR", self.minor)?;
		table.set("PATCH", self.patch)?;
		Ok(LuaValue::Table(table))
	}
}

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
	let project = lua.create_table()?;

	project.set(
		"set",
		lua.create_function(|lua, project: Project| {
            let state = &mut *lua.app_data_mut::<State>().unwrap();
            state.project = Some(project);
			Ok(())
		})?,
	)?;

	project.set(
		"get",
		lua.create_function(|lua, ()| {
            let state = &lua.app_data_ref::<State>().unwrap();
            if let Some(project) = &state.project {
                Ok(Some(project.clone()))
            } else {
    			Ok(None)
            }
		})?,
	)?;

	Ok(project)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_project_from_lua() -> LuaResult<()> {
		let lua = Lua::new();

		// Register the function
		let project = create(&lua)?;
		lua.globals().set("project", project)?;

		// Load and call with your project table
		lua.load(
			r#"
            local project_data = {
                name = "Test Project",
                VERSION = {
                    MAJOR = 0,
                    MINOR = 0,
                    PATCH = 1,
                },
                channels = {
                    {
                        name = "pluck 0",
                        instrument = {
                            name = "pluck",
                            state = {0.66, 0.5, 0.2, 0.26, 0.4, 0.25, 0.1},
                        },
                        notes = {},
                        control = {
                            sustain = {
                                {
                                    time = 8.531,
                                    value = true,
                                },
                                {
                                    time = 8.860,
                                    value = false,
                                },
                            },
                        },
                        effects = {
                            {
                                name = "gain",
                                state = {1},
                            },
                            {
                                name = "pan",
                                state = {1, 0},
                            },
                        },
                        hue = 312.91,
                        visible = true,
                        mute = false,
                        solo = false,
                        armed = false,
                        lock = false,
                    },
                },
                transport = {
                    start_time = 2.236,
                    recording = true,
                },
            }
            project.set(project_data)
        "#,
		)
		.exec()?;

		Ok(())
	}

	#[test]
	fn test_project_to_lua() -> LuaResult<()> {
		let lua = Lua::new();

		// Register the function
		// let project = create(&lua)?;
		let project = Project {
			channels: vec![Channel {
				name: "synth 0".into(),
				instrument: Instrument { name: "synth".into(), state: vec![0.5, 0.3, 0.8] },
				notes: vec![],
				control: Control::default(),
				effects: vec![Effect { name: "reverb".into(), state: vec![0.4, 0.6] }],
				hue: 180.0,
				visible: true,
				mute: false,
				solo: false,
				armed: false,
				lock: false,
			}],
			name: "Example Project".into(),
			version: Version { major: 1, minor: 0, patch: 0 },
			transport: Transport { recording: false, start_time: 0.0 },
		};
		lua.globals().set("p", project)?;

		// Call get and verify the returned data
		lua.load(
			r#"
            assert(p.name == "Example Project", "Project name mismatch")
            assert(p.VERSION.MAJOR == 1, "Version major mismatch")
            assert(p.VERSION.MINOR == 0, "Version minor mismatch")
            assert(p.VERSION.PATCH == 0, "Version patch mismatch")
            assert(p.transport.start_time == 0.0, "Transport start_time mismatch")
            assert(p.transport.recording == false, "Transport recording mismatch")

            assert(#p.channels == 1, "Expected 1 channel")
            assert(p.channels[1].name == "synth 0", "Channel name mismatch")
            assert(p.channels[1].instrument.name == "synth", "Instrument name mismatch")
            assert(#p.channels[1].instrument.state == 3, "Instrument state length mismatch")
            assert(p.channels[1].hue == 180.0, "Channel hue mismatch")
            assert(p.channels[1].visible == true, "Channel visible mismatch")
            assert(p.channels[1].mute == false, "Channel mute mismatch")

            assert(#p.channels[1].effects == 1, "Expected 1 effect")
            assert(p.channels[1].effects[1].name == "reverb", "Effect name mismatch")

            print("All assertions passed!")
        "#,
		)
		.exec()?;

		Ok(())
	}
}
