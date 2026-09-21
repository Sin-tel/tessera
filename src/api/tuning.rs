use crate::api::lua_serde;
use crate::tuning::{Definition, NotationInfo, ScaleCandidates, TuningSystem, notations};
use mlua::prelude::*;

lua_serde!(Definition);
lua_serde!(NotationInfo);
lua_serde!(ScaleCandidates);

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
			Ok(this.scale(scale_index(index)?).notes.clone())
		});

		// Linear map from a note to its index in the scale, or nil if there is none.
		methods.add_method("scale_map", |_, this, index: usize| {
			Ok(this.scale(scale_index(index)?).map.clone())
		});
	}
}

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
	let tuning = lua.create_table()?;

	// tessera.tuning.new(definition, scale_candidates)
	tuning.set(
		"new",
		lua.create_function(
			|_, (definition, candidates): (Definition, Option<ScaleCandidates>)| {
				TuningSystem::new(&definition, &candidates.unwrap_or_default())
					.map_err(LuaError::RuntimeError)
			},
		)?,
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

// Lua scale index (1 = diatonic, 2 = chromatic, 3 = fine) to 0-based index.
fn scale_index(index: usize) -> LuaResult<usize> {
	if !(1..=3).contains(&index) {
		return Err(LuaError::RuntimeError(format!("Scale index {index} out of range")));
	}
	Ok(index - 1)
}
