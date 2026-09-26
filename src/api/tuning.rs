use crate::api::lua_serde;
use crate::tuning::{
	NotationChoice, NotationOption, NotationStyle, ScaleCandidates, TemperamentDef, TuningSystem,
	notations,
};
use mlua::prelude::*;

lua_serde!(TemperamentDef);
lua_serde!(NotationChoice);
lua_serde!(NotationStyle);
lua_serde!(NotationOption);
lua_serde!(ScaleCandidates);

impl LuaUserData for TuningSystem {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("len", |_, this, ()| Ok(this.len()));
		methods.add_method("choice", |_, this, ()| Ok(this.choice()));
		methods.add_method("style", |_, this, ()| Ok(this.style().clone()));
		methods
			.add_method("generator_pitches", |_, this, ()| Ok(this.generator_pitches().to_vec()));
		methods
			.add_method("simple_ratios", |_, this, note: Vec<i64>| Ok(this.simple_ratios(&note)));

		methods.add_method("simple_spellings", |_, this, note: Vec<i64>| {
			Ok(this.simple_spellings(&note))
		});

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

	// tessera.tuning.new(temperament, notation_choice, scale_candidates)
	tuning.set(
		"new",
		lua.create_function(
			|_,
			 (def, choice, candidates): (
				TemperamentDef,
				NotationChoice,
				Option<ScaleCandidates>,
			)| {
				TuningSystem::new(&def, choice, &candidates.unwrap_or_default())
					.map_err(LuaError::RuntimeError)
			},
		)?,
	)?;

	// tessera.tuning.notations(temperament)
	tuning.set(
		"notations",
		lua.create_function(|_, def: TemperamentDef| {
			notations(&def).map_err(LuaError::RuntimeError)
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
