use crate::api::lua_serde;
use crate::log::log_error;
use crate::tuning::{
	Conversion, NotationChoice, NotationOption, NotationStyle, ScaleCandidates, TemperamentDef,
	TuningSystem, notations,
};
use mlua::prelude::*;

lua_serde!(TemperamentDef);
lua_serde!(NotationChoice);
lua_serde!(NotationStyle);
lua_serde!(NotationOption);
lua_serde!(ScaleCandidates);
lua_serde!(Conversion);

impl LuaUserData for TuningSystem {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("len", |_, this, ()| Ok(this.len()));
		methods.add_method("choice", |_, this, ()| Ok(this.choice()));
		methods.add_method("style", |_, this, ()| Ok(this.style().clone()));
		methods
			.add_method("generator_pitches", |_, this, ()| Ok(this.generator_pitches().to_vec()));
		methods.add_method("simple_ratios", |_, this, note: Vec<i64>| {
			check_note(this, &note)?;
			Ok(this.simple_ratios(&note))
		});

		// The ways to write a note that respelling cycles through, the same list for every spelling of it.
		methods.add_method("respellings", |_, this, note: Vec<i64>| {
			check_note(this, &note)?;
			Ok(this.respellings(&note))
		});

		// Notes written more simply as a whole, if that saves more than `min_gain` per note.
		methods.add_method("simplify_chord", |_, this, (notes, min_gain): (Vec<Vec<i64>>, f64)| {
			for note in &notes {
				check_note(this, note)?;
			}
			Ok(this.simplify_chord(&notes, min_gain))
		});

		// How complex a spelling is, lower is simpler.
		methods.add_method("spelling_cost", |_, this, note: Vec<i64>| {
			check_note(this, &note)?;
			Ok(this.spelling_cost(&note))
		});

		// what kind of conversion needs to happen
		methods.add_method("conversion", |_, this, from: LuaUserDataRef<TuningSystem>| {
			Ok(this.conversion(&from))
		});

		// Convert notes to new tuning system.
		// Returns the new notes, and how many of them changed.
		methods.add_method(
			"convert",
			|_, this, (from, notes): (LuaUserDataRef<TuningSystem>, Vec<Vec<i64>>)| {
				for note in &notes {
					check_note(&from, note)?;
				}
				Ok(this.convert(&from, &notes))
			},
		);

		// one step of an equal temperament, nil otherwise
		methods.add_method("step", |_, this, ()| Ok(this.step()));

		// 1 = diatonic, 2 = chromatic, 3 = fine
		methods.add_method("scale", |_, this, index: usize| {
			Ok(this.scale(scale_index(index)?).notes.clone())
		});

		// Linear map from a note to its index in the scale.
		methods.add_method("scale_map", |_, this, index: usize| {
			Ok(this.scale(scale_index(index)?).map.clone())
		});
	}
}

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
	let tuning = lua.create_table()?;

	// tessera.tuning.new(temperament, notation_choice, scale_candidates)
	// nil if the tuning can't be built, the reason is logged
	tuning.set(
		"new",
		lua.create_function(
			|_,
			 (def, choice, candidates): (
				TemperamentDef,
				NotationChoice,
				Option<ScaleCandidates>,
			)| {
				Ok(TuningSystem::new(&def, choice, &candidates.unwrap_or_default())
					.inspect_err(|e| log_error!("{e}"))
					.ok())
			},
		)?,
	)?;

	// tessera.tuning.notations(temperament)
	// nil if the temperament is invalid, the reason is logged
	tuning.set(
		"notations",
		lua.create_function(|_, def: TemperamentDef| {
			Ok(notations(&def).inspect_err(|e| log_error!("{e}")).ok())
		})?,
	)?;

	Ok(tuning)
}

// xen_utils panics on a note of the wrong length, so check what comes from Lua.
fn check_note(system: &TuningSystem, note: &[i64]) -> LuaResult<()> {
	if note.len() != system.len() {
		return Err(LuaError::RuntimeError(format!(
			"Note has {} coordinates, expected {}",
			note.len(),
			system.len()
		)));
	}
	Ok(())
}

// Lua scale index (1 = diatonic, 2 = chromatic, 3 = fine) to 0-based index.
fn scale_index(index: usize) -> LuaResult<usize> {
	if !(1..=3).contains(&index) {
		return Err(LuaError::RuntimeError(format!("Scale index {index} out of range")));
	}
	Ok(index - 1)
}
