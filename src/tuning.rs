use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use xen_utils::notation::spelling_cost;
use xen_utils::{Notation, Subgroup, Temperament, Tuning};

// Number of fifths in the chain scales, and how far down from the root they start.
const DIATONIC: (usize, i64) = (7, 1);
const CHROMATIC: (usize, i64) = (12, 4);
// Range of sizes for the fine chain of fifths. The largest one that has a projection is used.
const FINE_CHAIN: std::ops::RangeInclusive<usize> = 15..=31;
// How many spellings respelling cycles through, at most.
const RESPELL_COUNT: usize = 3;
// How much more than the best spelling the others may cost (see spelling_cost).
// A mark is 7 and a fifth 2. For G in 31et this keeps vvG# (+24), but not ^^^F# (+27).
const RESPELL_MAX_COST: i64 = 26;

// Which temperament to use.
// Either give `commas` to temper out, or `et` for an equal temperament.
// If neither is present, it is just intonation.
//
// This is part of the tuning definition in the save file, which also has a name and a NotationChoice.
//
// Note: mlua serializes None as a null value that is truthy in Lua,
// so anything that goes to Lua needs `skip_serializing_if` on its options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemperamentDef {
	// Subgroup in shorthand, e.g. "2.3.5.7"
	pub subgroup: String,
	// Commas to temper out as "p/q" strings.
	#[serde(default)]
	pub commas: Vec<String>,
	// Equal division of the octave.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub et: Option<i64>,
}

// Which of the notation options of a temperament to use.
//
// An option is identified by its number of accidentals (see Notation::with_count),
// and whether the one that is half an apotome is written as a half sharp.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotationChoice {
	pub accidentals: usize,
	#[serde(default)]
	pub half_sharp: bool,
	#[serde(default)]
	pub johnston: bool,
}

// How to draw spellings, for notation.lua.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotationStyle {
	// One for every coordinate after the octave and the fifth.
	pub accidentals: Vec<AccidentalStyle>,
	// Use Johnston style, ignores accidentals
	pub johnston: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccidentalStyle {
	// Just ratio of the accidental.
	pub ratio: String,
	// Written as half a sharp, combined with the sharps and flats.
	pub half_sharp: bool,
}

// A notation that can be picked in the settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotationOption {
	pub choice: NotationChoice,
	pub recommended: bool,
	pub style: NotationStyle,
	// How the primes beyond 3 are written.
	pub spellings: Vec<PrimeSpelling>,
}

// Just intonation scales to try for each kind of scale, in order of preference.
// Each scale is a list of ratios above the unison, up to and including the octave.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScaleCandidates {
	#[serde(default)]
	pub diatonic: Vec<Vec<String>>,
	#[serde(default)]
	pub chromatic: Vec<Vec<String>>,
	#[serde(default)]
	pub fine: Vec<Vec<String>>,
}

// Notes in one octave, in order of their steps. The first one is always the unison.
// The order need not be the order of their pitches.
#[derive(Debug, Clone, Default)]
pub struct Scale {
	pub notes: Vec<Vec<i64>>,
	// Linear map from notation coordinates to steps of the scale,
	// that takes every note in the scale to its own index.
	pub map: Vec<i64>,
}

#[derive(Debug)]
pub struct TuningSystem {
	notation: Notation,
	choice: NotationChoice,
	style: NotationStyle,
	// diatonic, chromatic, fine
	scales: [Scale; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Conversion {
	// Same temperament, every accidental is still there: nothing changes.
	Unchanged,
	// Same accidentals: notes keep their spelling, the new temperament retunes them.
	Retune,
	// Same temperament: notes keep their pitch, but may be respelled.
	Respell,
	// Accidentals missing in the new notation are dropped: spelling and pitch may change.
	Clip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimeSpelling {
	pub ratio: String,
	// may contain duplicates, deduped in lua side
	pub intervals: Vec<Vec<i64>>,
}

impl TuningSystem {
	pub fn new(
		def: &TemperamentDef,
		choice: NotationChoice,
		candidates: &ScaleCandidates,
	) -> Result<Self, String> {
		let temperament = temperament(def)?;
		let mut notation = Notation::with_count(&temperament, choice.accidentals)
			.map_err(|e| format!("Notation {choice:?} is not available: {e}"))?;
		if def.et.is_some() {
			notation = notation.with_tuning(equal_tuning(&temperament)).map_err(err)?;
		}
		if !choices(&notation).contains(&choice) {
			return Err(format!("Notation {choice:?} is not available"));
		}

		let style = notation_style(&notation, choice);
		let mut system = Self { notation, choice, style, scales: Default::default() };
		// A temperament generated by the octave and the fifth gets plain chains of fifths,
		// the candidates are only a fallback there.
		let chain_first = system.fifth_generated();
		let no_projection = |kind: &str| format!("{kind} scale has no projection");

		// TODO: this is messy

		let diatonic = system
			.candidate_or(chain_first, &candidates.diatonic, || {
				system.chain_scale(DIATONIC.0, DIATONIC.1)
			})
			.ok_or_else(|| no_projection("Diatonic"))?;
		let chromatic = system
			.candidate_or(chain_first, &candidates.chromatic, || {
				system.chain_scale(CHROMATIC.0, CHROMATIC.1)
			})
			.ok_or_else(|| no_projection("Chromatic"))?;
		let fine = system
			.fine_scale(chain_first, &candidates.fine)
			.ok_or_else(|| no_projection("Fine"))?;

		system.scales = [diatonic, chromatic, fine];

		Ok(system)
	}

	pub fn choice(&self) -> NotationChoice {
		self.choice
	}

	pub fn style(&self) -> &NotationStyle {
		&self.style
	}

	#[allow(clippy::len_without_is_empty)]
	pub fn len(&self) -> usize {
		self.notation.len()
	}

	// size of each notation generator, semitones
	pub fn generator_pitches(&self) -> &[f64] {
		self.notation.pitches()
	}

	// Scale by index (0 = diatonic, 1 = chromatic, 2 = fine).
	pub fn scale(&self, index: usize) -> &Scale {
		&self.scales[index]
	}

	pub fn simple_ratios(&self, note: &[i64]) -> Vec<String> {
		let subgroup = self.notation.subgroup();
		let list = self.notation.simplifications(note, 3);

		if list.is_empty() {
			return Vec::new();
		}

		// some heuristics on what to display
		let low = subgroup.sopfr(&list[0]);
		let mut bound = low;
		if low < 18 {
			bound = (low + 7).min(18);
		}

		list.iter()
			.filter(|k| subgroup.sopfr(k) <= bound)
			.map(|k| interval_to_string(subgroup, k))
			.collect()
	}

	// The ways to write a note that respelling cycles through, best first.
	// The same list for every spelling of the note, so it can be cycled through.
	// At least two if the notation has enharmonics, the others only if they're not much worse.
	// Note: may contain spellings that are drawn the same (half sharps), which the Lua side dedups.
	pub fn respellings(&self, note: &[i64]) -> Vec<Vec<i64>> {
		// Ties are broken relative to the input, so start from the same spelling every time.
		let canonical = self.notation.from_tempered(&self.notation.to_tempered(note));
		let list = self.notation.spellings(&canonical, RESPELL_COUNT);
		let max_cost = spelling_cost(&list[0]) + RESPELL_MAX_COST;
		list.into_iter()
			.enumerate()
			.filter(|(i, s)| *i < 2 || spelling_cost(s) <= max_cost)
			.map(|(_, s)| s)
			.collect()
	}

	// Notes written more simply as a whole, all moved by the same enharmonic so a chord stays the same
	// chord (see Notation::spell_chord). Only if that saves more than `min_gain` per note on average,
	// otherwise the notes as given.
	pub fn simplify_chord(&self, notes: &[Vec<i64>], min_gain: f64) -> Vec<Vec<i64>> {
		let total = |notes: &[Vec<i64>]| notes.iter().map(|n| spelling_cost(n)).sum::<i64>();
		let simplified = self.notation.spell_chord(notes);
		let gain = (total(notes) - total(&simplified)) as f64 / notes.len() as f64;
		if gain > min_gain { simplified } else { notes.to_vec() }
	}

	// How complex a spelling is to read, see xen_utils spelling_cost.
	pub fn spelling_cost(&self, note: &[i64]) -> i64 {
		spelling_cost(note)
	}

	// How notes from `from` need to be converted.
	pub fn conversion(&self, from: &TuningSystem) -> Conversion {
		let same_temperament = self.notation.temperament() == from.notation.temperament();
		let keeps_all = self.coordinate_map(from).iter().all(Option::is_some);
		match (keeps_all, same_temperament) {
			(true, true) => Conversion::Unchanged,
			(true, false) => Conversion::Retune,
			(false, true) => Conversion::Respell,
			(false, false) => Conversion::Clip,
		}
	}

	// Notes from `from`, convert to this tuning (see `conversion`).
	// Also returns how many of them changed.
	pub fn convert(&self, from: &TuningSystem, notes: &[Vec<i64>]) -> (Vec<Vec<i64>>, usize) {
		let kind = self.conversion(from);
		let map = self.coordinate_map(from);

		let mut changed = 0;
		let converted = notes
			.iter()
			.map(|note| {
				let new = if kind == Conversion::Respell {
					// Same temperament, so the same subgroup.
					self.notation.from_interval(&from.notation.to_interval(note))
				} else {
					let mut new = vec![0; self.len()];
					for (x, &j) in note.iter().zip(&map) {
						if let Some(j) = j {
							new[j] += x;
						}
					}
					new
				};
				let old_ji = factors(from.notation.subgroup(), &from.notation.to_interval(note));
				let new_ji = factors(self.notation.subgroup(), &self.notation.to_interval(&new));
				if old_ji != new_ji {
					changed += 1;
				}
				new
			})
			.collect();
		(converted, changed)
	}

	// For each coordinate of `from`, the coordinate here with the same generator, if there is one.
	fn coordinate_map(&self, from: &TuningSystem) -> Vec<Option<usize>> {
		let ours: Vec<_> = self
			.notation
			.generators()
			.iter()
			.map(|g| factors(self.notation.subgroup(), g))
			.collect();
		from.notation
			.generators()
			.iter()
			.map(|g| {
				let g = factors(from.notation.subgroup(), g);
				ours.iter().position(|x| *x == g)
			})
			.collect()
	}

	// One step of an equal temperament, spelled in the notation.
	// None if not available.
	pub fn step(&self) -> Option<Vec<i64>> {
		if self.notation.temperament().rank() != 1 {
			return None;
		}
		let step = self.notation.from_tempered(&[1]);
		if self.pitch(&step) < 0.0 {
			return Some(step.iter().map(|x| -x).collect());
		}
		Some(step)
	}

	fn octave(&self) -> f64 {
		self.notation.pitches()[0]
	}

	fn pitch(&self, note: &[i64]) -> f64 {
		self.notation.pitch(note)
	}

	// Move a note into the octave above the unison.
	fn reduce(&self, note: &mut [i64]) {
		// Small offset so notes that are octaves apart in the temperament reduce identically
		note[0] -= ((self.pitch(note) + 1e-9) / self.octave()).floor() as i64;
	}

	// Well-formed scale from a chain of fifths, reduced to one octave.
	//
	// If the temperament closes the circle of fifths early (like 15et, where B is C),
	// there would be duplicates. In that case the note closest to the root is kept.
	//
	// The notes are reduced and sorted by their tempered pitch. None if there is no projection.
	fn chain_scale(&self, n: usize, offset: i64) -> Option<Scale> {
		let mut fifths: Vec<i64> = (0..n as i64).map(|i| i - offset).collect();
		fifths.sort_by_key(|&f| (f.abs(), -f));

		let mut seen = HashSet::new();
		let mut notes = Vec::new();
		for f in fifths {
			let mut note = vec![0; self.len()];
			note[1] = f;
			self.reduce(&mut note);
			if seen.insert(self.notation.to_tempered(&note)) {
				notes.push(note);
			}
		}
		self.sort(&mut notes);
		let map = self.projection(&notes)?;
		Some(Scale { notes, map })
	}

	// The first candidate that works in this tuning.
	fn first_candidate(&self, candidates: &[Vec<String>]) -> Option<Scale> {
		candidates.iter().find_map(|ratios| {
			let scale = self.ji_scale(ratios)?;
			Some(scale)
		})
	}

	// Scale from a list of just ratios, spelled in the notation.
	// The ratios are taken as they are given, in order and inside the octave.
	//
	// None if some ratio isn't in the subgroup, if the temperament merges two notes,
	// if the tuning puts them out of order, or if there is no projection.
	fn ji_scale(&self, ratios: &[String]) -> Option<Scale> {
		let subgroup = self.notation.subgroup();

		let unison = vec![0; self.len()];
		let mut seen = HashSet::from([self.notation.to_tempered(&unison)]);
		let mut notes = vec![unison];
		for ratio in ratios {
			let interval = subgroup.parse_ratio(ratio).ok()?;
			if interval == self.notation.generators()[0] {
				continue;
			}
			let note = self.notation.from_interval(&interval);
			if !seen.insert(self.notation.to_tempered(&note)) {
				return None;
			}
			notes.push(note);
		}
		if notes.windows(2).any(|w| self.pitch(&w[1]) <= self.pitch(&w[0])) {
			return None;
		}
		let map = self.projection(&notes)?;
		Some(Scale { notes, map })
	}

	// Rank 2, with the octave and the fifth as generators.
	fn fifth_generated(&self) -> bool {
		let images = self.notation.images();
		self.notation.temperament().rank() == 2
			&& (images[0][0] * images[1][1] - images[0][1] * images[1][0]).abs() == 1
	}

	// The first candidate that works or a generated scale, whichever comes first.
	fn candidate_or(
		&self,
		chain_first: bool,
		candidates: &[Vec<String>],
		chain: impl Fn() -> Option<Scale>,
	) -> Option<Scale> {
		if chain_first {
			chain().or_else(|| self.first_candidate(candidates))
		} else {
			self.first_candidate(candidates).or_else(chain)
		}
	}

	fn fine_scale(&self, chain_first: bool, candidates: &[Vec<String>]) -> Option<Scale> {
		if self.notation.temperament().rank() > 1 {
			// A candidate, or the largest chain of fifths that has a projection.
			return self.candidate_or(chain_first, candidates, || {
				FINE_CHAIN.rev().find_map(|n| self.chain_scale(n, chain_offset(n)))
			});
		}

		// Equal temperament: every step, spelled by the notation.
		// The temperament mapping itself is the projection.
		let mut map: Vec<i64> = self.notation.images().iter().map(|image| image[0]).collect();
		let sign = map[0].signum();
		map.iter_mut().for_each(|x| *x *= sign);

		let notes = (0..map[0])
			.map(|k| self.notation.from_tempered(&[k * sign]))
			.collect();
		Some(Scale { notes, map })
	}

	// Linear map that takes every note of the scale to its index, if there is one.
	// See Notation::scale_map.
	fn projection(&self, notes: &[Vec<i64>]) -> Option<Vec<i64>> {
		self.notation.scale_map(notes)
	}

	fn sort(&self, notes: &mut [Vec<i64>]) {
		notes.sort_by(|a, b| self.pitch(a).total_cmp(&self.pitch(b)));
	}
}

// Every notation that can be used for a temperament.
pub fn notations(def: &TemperamentDef) -> Result<Vec<NotationOption>, String> {
	let mut options = Vec::new();
	for notation in Notation::options(&temperament(def)?).map_err(err)? {
		let recommended = notation.keeps_nominals();
		let spellings = prime_spellings(&notation);
		for choice in choices(&notation) {
			options.push(NotationOption {
				choice,
				recommended,
				style: notation_style(&notation, choice),
				spellings: spellings.clone(),
			});
		}
	}
	Ok(options)
}

// The ways a notation from xen_utils can be written: Johnston for JI,
// and with or without half sharps where a temperament allows them.
fn choices(notation: &Notation) -> Vec<NotationChoice> {
	let temperament = notation.temperament();
	let accidentals = notation.len() - 2;
	let plain = NotationChoice { accidentals, half_sharp: false, johnston: false };
	if temperament.rank() == temperament.dim() {
		// JI, nothing to do for pythagorean
		if temperament.rank() > 2 {
			return vec![plain, NotationChoice { johnston: true, ..plain }];
		}
		return vec![plain];
	}
	let half_sharp = NotationChoice { half_sharp: true, ..plain };
	match half_sharp_index(notation) {
		// 33/32 is always written as a half sharp, others get the choice.
		Some(i) if generator_ratios(notation)[i] == "33/32" => vec![half_sharp],
		Some(_) => vec![plain, half_sharp],
		None => vec![plain],
	}
}

fn notation_style(notation: &Notation, choice: NotationChoice) -> NotationStyle {
	let half_sharp_index = if choice.half_sharp { half_sharp_index(notation) } else { None };
	let accidentals = generator_ratios(notation)
		.into_iter()
		.enumerate()
		.skip(2)
		.map(|(i, ratio)| AccidentalStyle { ratio, half_sharp: Some(i) == half_sharp_index })
		.collect();
	NotationStyle { accidentals, johnston: choice.johnston }
}

fn temperament(def: &TemperamentDef) -> Result<Temperament, String> {
	let subgroup: Subgroup = def.subgroup.parse().map_err(err)?;

	match (&def.et, def.commas.is_empty()) {
		(Some(_), false) => Err("Can't give both `et` and `commas`".to_string()),
		(Some(n), true) => Temperament::equal(*n, &subgroup).map_err(err),
		(None, true) => Temperament::from_ji(&subgroup).map_err(err),
		(None, false) => {
			let commas = def
				.commas
				.iter()
				.map(|s| subgroup.parse_ratio(s).map_err(err))
				.collect::<Result<Vec<_>, String>>()?;
			Temperament::from_commas(&commas, &subgroup).map_err(err)
		},
	}
}

// Equal temperaments get pure octaves instead of the optimal tuning.
fn equal_tuning(temperament: &Temperament) -> Tuning {
	assert_eq!(temperament.subgroup().basis()[0], 2, "Subgroup must start with 2");
	let steps = temperament.mapping()[0][0];
	Tuning::new(temperament, vec![12.0 / steps as f64]).expect("rank 1 temperament")
}

fn err(e: xen_utils::Error) -> String {
	e.to_string()
}

// Coordinate of the accidental that can be written as a half sharp, if there is one:
// two of them are the apotome (7 fifths down 4 octaves) in the temperament.
// By construction this can only appear at one accidental.
fn half_sharp_index(notation: &Notation) -> Option<usize> {
	let mut apotome = vec![0; notation.len()];
	(apotome[0], apotome[1]) = (-4, 7);
	let apotome = notation.to_tempered(&apotome);

	(2..notation.len())
		.find(|&i| notation.images()[i].iter().zip(&apotome).all(|(a, b)| 2 * a == *b))
}

// How every prime beyond 3 is written, on its nominal and the way `spell` writes it.
fn prime_spellings(notation: &Notation) -> Vec<PrimeSpelling> {
	let subgroup = notation.subgroup();
	(2..subgroup.dim())
		.zip(notation.nominal_spellings())
		.map(|(i, nominal)| {
			let interval = subgroup.reduced_prime(i);
			let mut intervals = Vec::new();
			if let Some(s) = nominal {
				intervals.push(s);
			}

			intervals.extend(notation.spellings(&notation.from_interval(&interval), 4));

			let max_cost = spelling_cost(&intervals[0]) + 2;

			let intervals = intervals
				.into_iter()
				.filter(|s| spelling_cost(s) <= max_cost)
				.collect();

			PrimeSpelling { ratio: interval_to_string(subgroup, &interval), intervals }
		})
		.collect()
}

fn generator_ratios(notation: &Notation) -> Vec<String> {
	let mut generators = Vec::new();
	for g in notation.generators() {
		generators.push(interval_to_string(notation.subgroup(), g));
	}
	generators
}

// Fifths down from the root for a chain of n, so that it is centered around D.
fn chain_offset(n: usize) -> i64 {
	n as i64 / 2 - 2
}

// An interval as (prime, exponent) pairs, so intervals over different subgroups can be compared.
fn factors(subgroup: &Subgroup, interval: &[i64]) -> Vec<(u32, i64)> {
	subgroup
		.basis()
		.iter()
		.copied()
		.zip(interval.iter().copied())
		.filter(|&(_, x)| x != 0)
		.collect()
}

fn interval_to_string(subgroup: &Subgroup, x: &[i64]) -> String {
	// overflow needs to render something, note font only display capital letters as text
	if let Ok((p, q)) = subgroup.to_ratio(x) { format!("{p}/{q}") } else { "<BIG NUMBER>".into() }
}

#[cfg(test)]
mod tests {
	use super::*;

	fn system(
		subgroup: &str,
		commas: &[&str],
		et: Option<i64>,
		accidentals: usize,
	) -> TuningSystem {
		let def = TemperamentDef {
			subgroup: subgroup.into(),
			commas: commas.iter().map(|&c| c.into()).collect(),
			et,
		};
		let choice = NotationChoice { accidentals, ..Default::default() };
		TuningSystem::new(&def, choice, &ScaleCandidates::default()).unwrap()
	}

	// Respelling cycles through the list, so every spelling in it has to give the same list.
	#[test]
	fn spellings_cycle() {
		let systems = [
			system("2.3.5.7", &["81/80", "126/125"], None, 0),
			system("2.3.5.7", &[], Some(31), 1),
			system("2.3.5.7", &[], Some(22), 1),
			system("2.3.5.7.11", &["225/224", "385/384"], None, 1),
		];
		for system in &systems {
			for fifths in -12..=12 {
				for acc in -2..=2 {
					let mut note = vec![0; system.len()];
					note[1] = fifths;
					if system.len() > 2 {
						note[2] = acc;
					}
					let list = system.respellings(&note);
					// meantone with plain sharps and flats has no enharmonics
					if system.notation.temperament().rank() == 1 {
						assert!(list.len() >= 2, "{note:?} has only {} spellings", list.len());
					}
					for s in &list {
						assert_eq!(system.respellings(s), list, "from {s:?}");
						assert!((system.pitch(s) - system.pitch(&note)).abs() < 1e-9);
					}
				}
			}
		}
	}
}
