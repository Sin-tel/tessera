use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use xen_utils::notation::spelling_cost;
use xen_utils::{Notation, Subgroup, Temperament};

// Number of fifths in the chain scales, and how far down from the root they start.
const DIATONIC: (usize, i64) = (7, 1);
const CHROMATIC: (usize, i64) = (12, 4);
// Range of sizes for the fine chain of fifths. The largest one that has a projection is used.
const FINE_CHAIN: std::ops::RangeInclusive<usize> = 15..=31;

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

// Notes in one octave, sorted by pitch. The first one is always the unison.
#[derive(Debug, Clone, Default)]
pub struct Scale {
	pub notes: Vec<Vec<i64>>,
	// Linear map from notation coordinates to steps of the scale,
	// that takes every note in the scale to its own index.
	// None if there is no such map, then lookups have to go by pitch.
	pub map: Option<Vec<i64>>,
}

#[derive(Debug)]
pub struct TuningSystem {
	notation: Notation,
	choice: NotationChoice,
	style: NotationStyle,
	// diatonic, chromatic, fine
	scales: [Scale; 3],
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
		let notation = Notation::with_count(&temperament, choice.accidentals)
			.map_err(|e| format!("Notation {choice:?} is not available: {e}"))?;
		if !choices(&notation).contains(&choice) {
			return Err(format!("Notation {choice:?} is not available"));
		}

		let style = notation_style(&notation, choice);
		let mut system = Self { notation, choice, style, scales: Default::default() };
		let diatonic = system
			.first_candidate(&candidates.diatonic)
			.unwrap_or_else(|| system.chain_scale(DIATONIC.0, DIATONIC.1));
		let chromatic = system
			.first_candidate(&candidates.chromatic)
			.unwrap_or_else(|| system.chain_scale(CHROMATIC.0, CHROMATIC.1));
		let fine = system.fine_scale(&candidates.fine);
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

	pub fn simple_spellings(&self, note: &[i64]) -> Vec<Vec<i64>> {
		// TODO: prefer the literal spelling, like Notation::from_interval.
		let spellings = self.notation.spellings(note, 3);

		let max_cost = spelling_cost(&spellings[0]) + 2;
		spellings
			.into_iter()
			.filter(|s| spelling_cost(s) <= max_cost)
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
	fn chain_scale(&self, n: usize, offset: i64) -> Scale {
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
		let map = self.projection(&notes);
		Scale { notes, map }
	}

	// The first candidate that works in this tuning.
	fn first_candidate(&self, candidates: &[Vec<String>]) -> Option<Scale> {
		candidates.iter().find_map(|ratios| self.ji_scale(ratios))
	}

	// Scale from a list of just ratios, spelled in the notation.
	//
	// None if some ratio isn't in the subgroup, if the temperament merges two notes,
	// or if there is no projection.
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
			let mut note = self.notation.from_interval(&interval);
			self.reduce(&mut note);
			if !seen.insert(self.notation.to_tempered(&note)) {
				return None;
			}
			notes.push(note);
		}
		self.sort(&mut notes);
		let map = self.projection(&notes)?;
		Some(Scale { notes, map: Some(map) })
	}

	fn fine_scale(&self, candidates: &[Vec<String>]) -> Scale {
		if self.notation.temperament().rank() > 1 {
			if let Some(scale) = self.first_candidate(candidates) {
				return scale;
			}
			// Largest chain of fifths that has a projection.
			for n in FINE_CHAIN.rev() {
				let scale = self.chain_scale(n, chain_offset(n));
				if scale.map.is_some() {
					return scale;
				}
			}
			let n = *FINE_CHAIN.end();
			return self.chain_scale(n, chain_offset(n));
		}

		// Equal temperament: every step, spelled by the notation.
		// The temperament mapping itself is the projection.
		let mut map: Vec<i64> = self.notation.images().iter().map(|image| image[0]).collect();
		let sign = map[0].signum();
		map.iter_mut().for_each(|x| *x *= sign);

		let notes = (0..map[0])
			.map(|k| self.notation.from_tempered(&[k * sign]))
			.collect();
		Scale { notes, map: Some(map) }
	}

	// Linear map that takes every note of the scale to its index, if there is one.
	//
	// This is the patent val for the size of the scale, applied to the just reading of
	// each notation coordinate. It exists when the scale is a constant structure that
	// the val agrees with.
	fn projection(&self, notes: &[Vec<i64>]) -> Option<Vec<i64>> {
		let n = notes.len() as f64;
		let val: Vec<i64> = self
			.notation
			.subgroup()
			.log_primes()
			.iter()
			.map(|l| (n * l).round() as i64)
			.collect();
		let map: Vec<i64> = self
			.notation
			.generators()
			.iter()
			.map(|generator| dot(generator, &val))
			.collect();

		let consistent = notes.iter().enumerate().all(|(i, note)| dot(&map, note) == i as i64);
		consistent.then_some(map)
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

fn dot(a: &[i64], b: &[i64]) -> i64 {
	a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn interval_to_string(subgroup: &Subgroup, x: &[i64]) -> String {
	// overflow needs to render something, note font only display capital letters as text
	if let Ok((p, q)) = subgroup.to_ratio(x) { format!("{p}/{q}") } else { "<BIG NUMBER>".into() }
}
