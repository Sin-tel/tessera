// Currently broken! Not going to fix until more stable

use crate::app::State;
use crate::log_error;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
	pub name: String,
	#[serde(rename = "VERSION")]
	pub version: Version,
	pub channels: Vec<Channel>,
	pub transport: Transport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
	pub name: String,
	pub state: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
	pub name: String,
	pub state: Vec<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Control {
	#[serde(default)]
	pub sustain: Vec<SustainEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SustainEvent {
	pub time: f64,
	pub value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transport {
	pub start_time: f64,
	pub recording: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
	#[serde(rename = "MAJOR")]
	pub major: u32,
	#[serde(rename = "MINOR")]
	pub minor: u32,
	#[serde(rename = "PATCH")]
	pub patch: u32,
}

impl Serialize for Vertex {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeSeq;
		let mut seq = serializer.serialize_seq(Some(3))?;
		seq.serialize_element(&self.x)?;
		seq.serialize_element(&self.y)?;
		seq.serialize_element(&self.w)?;
		seq.end()
	}
}

impl<'de> Deserialize<'de> for Vertex {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let arr: [f64; 3] = Deserialize::deserialize(deserializer)?;
		Ok(Vertex { x: arr[0], y: arr[1], w: arr[2] })
	}
}

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
	let project = lua.create_table()?;

	project.set(
		"set",
		lua.create_function(|lua, project: LuaValue| {
			let state = &mut *lua.app_data_mut::<State>().unwrap();
			state.project = match lua.from_value(project) {
				Ok(p) => Some(p),
				Err(e) => {
					log_error!("{e}");
					None
				},
			};
			// dbg!(&state.project);
			Ok(())
		})?,
	)?;

	project.set(
		"get",
		lua.create_function(|lua, ()| {
			let state = &lua.app_data_ref::<State>().unwrap();
			if let Some(project) = &state.project {
				Ok(Some(lua.to_value(&project)?))
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

		// quick test function
		let project = lua.create_function(|lua, project: LuaValue| {
			let project_data: Project = lua.from_value(project)?;
			assert!(project_data.name == "Test Project");
			Ok(())
		})?;

		lua.globals().set("load_project", project)?;

		// Load and call with your project table
		lua.load(
			r#"
			local project = {
				transport = {
					recording = true,
					start_time = 0,
				},
				name = "Test Project",
				VERSION = {
					MAJOR = 0,
					MINOR = 0,
					PATCH = 1,
				},
				channels = {
				},
			}
            load_project(project)
        "#,
		)
		.exec()?;

		Ok(())
	}

	#[test]
	fn test_project_to_lua() -> LuaResult<()> {
		let lua = Lua::new();

		let project = Project {
			channels: Vec::new(),
			name: "Example Project".into(),
			version: Version { major: 1, minor: 0, patch: 0 },
			transport: Transport { recording: false, start_time: 0.0 },
		};
		lua.globals().set("p", lua.to_value(&project)?)?;

		Ok(())
	}

	#[test]
	fn test_vertex_to_lua() -> LuaResult<()> {
		let lua = Lua::new();

		let vertex = Vertex { x: 1.0, y: 2.0, w: 3.0 };
		lua.globals().set("v", lua.to_value(&vertex)?)?;

		lua.load(
			r#"
            assert(v[1] == 1.0, "x mismatch")
            assert(v[2] == 2.0, "y mismatch")
            assert(v[3] == 3.0, "w mismatch")
        "#,
		)
		.exec()?;

		Ok(())
	}

	#[test]
	fn test_vertex_from_lua() -> LuaResult<()> {
		let lua = Lua::new();
		lua.load(
			r#"
            verts = {
                {0.5, 1.0, 0.25},
                {1.5, 0.8, 0.5},
            }
        "#,
		)
		.exec()?;

		let verts: Vec<Vertex> = lua.from_value(lua.globals().get("verts")?)?;
		assert_eq!(verts.len(), 2);
		assert_eq!(verts[0].x, 0.5);
		assert_eq!(verts[0].y, 1.0);
		assert_eq!(verts[0].w, 0.25);
		Ok(())
	}
}
