use crate::api::lua_serde;
use crate::tuning::{Definition, NotationInfo, TuningSystem, notations};
use mlua::prelude::*;

lua_serde!(Definition);
lua_serde!(NotationInfo);

impl LuaUserData for TuningSystem {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("len", |_, this, ()| Ok(this.len()));

		methods.add_method("get_notation_info", |_, this, ()| Ok(this.get_notation_info()));

		// size in semitones for each coordinate
		methods.add_method("pitches", |_, this, ()| Ok(this.pitches().to_vec()));

		// one step of an equal temperament, nil otherwise
		methods.add_method("step", |_, this, ()| Ok(this.step()));

		// 1 = diatonic, 2 = chromatic, 3 = fine
		methods.add_method("scale", |_, this, index: usize| {
			if !(1..=3).contains(&index) {
				return Err(LuaError::RuntimeError(format!("Scale index {index} out of range")));
			}
			Ok(this.scale(index - 1).to_vec())
		});
	}
}

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
	let tuning = lua.create_table()?;

	// tessera.tuning.new(definition)
	tuning.set(
		"new",
		lua.create_function(|_, definition: Definition| {
			TuningSystem::new(&definition).map_err(LuaError::RuntimeError)
		})?,
	)?;

	// tessera.tuning.notations(definition)
	tuning.set(
		"notations",
		lua.create_function(|_, definition: Definition| {
			notations(&definition).map_err(LuaError::RuntimeError)
		})?,
	)?;

	Ok(tuning)
}
