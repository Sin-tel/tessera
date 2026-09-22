use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use xen_utils::{Notation, Subgroup, Temperament};

// Number of fifths in the chain scales, and how far down from the root they start.
const DIATONIC: (usize, i64) = (7, 1);
const CHROMATIC: (usize, i64) = (12, 4);
// Range of sizes for the fine chain of fifths. The largest one that has a projection is used.
const FINE_CHAIN: std::ops::RangeInclusive<usize> = 15..=31;
// How many spellings of a scale note to look through for one that reads as the just ratio.
const SPELLING_OPTIONS: usize = 8;

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
}

// How to draw spellings, for notation.lua.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotationStyle {
	// One for every coordinate after the octave and the fifth.
	pub accidentals: Vec<AccidentalStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccidentalStyle {
	// Just ratio of the accidental.
	pub ratio: Ratio,
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
	// size of each notation coordinate in semitones.
	pitches: Vec<f64>,
	// diatonic, chromatic, fine
	scales: [Scale; 3],
}

type Ratio = (u64, u64);

// How a notation writes one prime, reduced to an octave (5/4, 7/4, 11/8, ..).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimeSpelling {
	pub ratio: Ratio,
	// On the nominal just intonation gives it, in notation coordinates.
	// None if the notation can't write it there.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub nominal: Option<Vec<i64>>,
	// The next best way to write it, if that is worth showing next to the nominal.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub alternative: Option<Vec<i64>>,
}

impl TuningSystem {
	// Without a notation choice, the recommended one is used.
	pub fn new(
		def: &TemperamentDef,
		choice: Option<NotationChoice>,
		candidates: &ScaleCandidates,
	) -> Result<Self, String> {
		let options = notation_options(&temperament(def)?)?;
		let index = if let Some(choice) = choice {
			options
				.iter()
				.position(|(_, c)| *c == choice)
				.ok_or_else(|| format!("Notation {choice:?} is not available"))?
		} else {
			options.iter().position(|(n, _)| n.keeps_nominals()).unwrap_or(0)
		};
		let (notation, choice) = options.into_iter().nth(index).expect("index is in range");

		let pitches = (0..notation.len())
			.map(|i| {
				notation
					.pitch(&basis_vec(i, notation.len()))
					.map(|cents| cents / 100.0)
					.map_err(err)
			})
			.collect::<Result<Vec<_>, _>>()?;

		let style = notation_style(&notation, choice.half_sharp);
		let mut system = Self { notation, choice, style, pitches, scales: Default::default() };
		let diatonic = system
			.first_candidate(&candidates.diatonic)
			.unwrap_or_else(|| system.chain_scale(DIATONIC.0, DIATONIC.1));
		let chromatic = system
			.first_candidate(&candidates.chromatic)
			.unwrap_or_else(|| system.chain_scale(CHROMATIC.0, CHROMATIC.1));
		let fine = system.fine_scale(&candidates.fine)?;
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
	pub fn pitches(&self) -> &[f64] {
		&self.pitches
	}

	// Scale by index (0 = diatonic, 1 = chromatic, 2 = fine).
	pub fn scale(&self, index: usize) -> &Scale {
		&self.scales[index]
	}

	// One step up of an equal temperament, spelled in the notation.
	// None for higher rank temperaments, or if the notation can't write it.
	pub fn step(&self) -> Option<Vec<i64>> {
		if self.notation.temperament().rank() != 1 {
			return None;
		}
		let step = self.notation.spell(&[1]).ok()?;
		if self.pitch(&step) < 0.0 {
			return Some(step.iter().map(|x| -x).collect());
		}
		Some(step)
	}

	fn octave(&self) -> f64 {
		self.pitches[0]
	}

	fn pitch(&self, note: &[i64]) -> f64 {
		note.iter().zip(&self.pitches).map(|(&n, p)| n as f64 * p).sum()
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
			if seen.insert(self.temper(&note)) {
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
		let octave = basis_vec(0, subgroup.dim());

		let unison = vec![0; self.len()];
		let mut seen = HashSet::from([self.temper(&unison)]);
		let mut notes = vec![unison];
		for ratio in ratios {
			let interval = subgroup.parse_ratio(ratio).ok()?;
			if interval == octave {
				continue;
			}
			let mut note = self.spell_literal(&interval)?;
			self.reduce(&mut note);
			if !seen.insert(self.temper(&note)) {
				return None;
			}
			notes.push(note);
		}
		self.sort(&mut notes);
		let map = self.projection(&notes)?;
		Some(Scale { notes, map: Some(map) })
	}

	// Spell a just interval as it is written literally if possible, like ^Db for 16/15
	// in 41et. Otherwise use the best spelling of its tempered image.
	fn spell_literal(&self, interval: &[i64]) -> Option<Vec<i64>> {
		let mut options = self.notation.spellings_interval(interval, SPELLING_OPTIONS).ok()?;
		let literal = options
			.iter()
			.position(|s| self.notation.to_interval(s).is_ok_and(|i| i == interval));
		Some(options.swap_remove(literal.unwrap_or(0)))
	}

	fn fine_scale(&self, candidates: &[Vec<String>]) -> Result<Scale, String> {
		if self.notation.temperament().rank() > 1 {
			if let Some(scale) = self.first_candidate(candidates) {
				return Ok(scale);
			}
			// Largest chain of fifths that has a projection.
			for n in FINE_CHAIN.rev() {
				let scale = self.chain_scale(n, chain_offset(n));
				if scale.map.is_some() {
					return Ok(scale);
				}
			}
			let n = *FINE_CHAIN.end();
			return Ok(self.chain_scale(n, chain_offset(n)));
		}

		// Equal temperament: every step, spelled by the notation.
		// The temperament mapping itself is the projection.
		let mut map: Vec<i64> = (0..self.len())
			.map(|i| self.temper(&basis_vec(i, self.len()))[0])
			.collect();
		let sign = map[0].signum();
		map.iter_mut().for_each(|x| *x *= sign);

		let notes = (0..map[0])
			.map(|k| self.notation.spell(&[k * sign]).map_err(err))
			.collect::<Result<Vec<_>, _>>()?;
		Ok(Scale { notes, map: Some(map) })
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
		let map: Vec<i64> = (0..self.len())
			.map(|i| {
				let interval = self
					.notation
					.to_interval(&basis_vec(i, self.len()))
					.expect("right length");
				dot(&interval, &val)
			})
			.collect();

		let consistent = notes.iter().enumerate().all(|(i, note)| dot(&map, note) == i as i64);
		consistent.then_some(map)
	}

	fn temper(&self, note: &[i64]) -> Vec<i64> {
		self.notation.temper(note).expect("note has the right length")
	}

	fn sort(&self, notes: &mut [Vec<i64>]) {
		notes.sort_by(|a, b| self.pitch(a).total_cmp(&self.pitch(b)));
	}
}

// Every notation that can be used for a temperament.
pub fn notations(def: &TemperamentDef) -> Result<Vec<NotationOption>, String> {
	let options = notation_options(&temperament(def)?)?;
	Ok(options
		.into_iter()
		.map(|(notation, choice)| NotationOption {
			choice,
			recommended: notation.keeps_nominals(),
			style: notation_style(&notation, choice.half_sharp),
			spellings: prime_spellings(&notation),
		})
		.collect())
}

// The notations xen_utils offers, and for those that can use a half sharp, whether they do.
fn notation_options(temperament: &Temperament) -> Result<Vec<(Notation, NotationChoice)>, String> {
	let mut options = Vec::new();
	for notation in Notation::options(temperament).map_err(err)? {
		let accidentals = notation.len() - 2;
		let plain = NotationChoice { accidentals, half_sharp: false };
		match half_sharp_index(&notation) {
			Some(i) => {
				// 33/32 is always written as a half sharp, others get the choice.
				if generator_ratios(&notation)[i] != (33, 32) {
					options.push((notation.clone(), plain));
				}
				options.push((notation, NotationChoice { accidentals, half_sharp: true }));
			},
			None => options.push((notation, plain)),
		}
	}
	Ok(options)
}

fn notation_style(notation: &Notation, half_sharp: bool) -> NotationStyle {
	let half_sharp_index = if half_sharp { half_sharp_index(notation) } else { None };
	let accidentals = generator_ratios(notation)
		.into_iter()
		.enumerate()
		.skip(2)
		.map(|(i, ratio)| AccidentalStyle { ratio, half_sharp: Some(i) == half_sharp_index })
		.collect();
	NotationStyle { accidentals }
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
	let apotome = notation.temper(&apotome).ok()?;

	(2..notation.len()).find(|&i| {
		let image = notation.temper(&basis_vec(i, notation.len())).expect("right length");
		image.iter().zip(&apotome).all(|(a, b)| 2 * a == *b)
	})
}

// How every prime beyond 3 is written, on its nominal and the way `spell` writes it.
fn prime_spellings(notation: &Notation) -> Vec<PrimeSpelling> {
	let basis = notation.subgroup().basis();
	(2..basis.len())
		.zip(notation.nominal_spellings())
		.map(|(i, nominal)| {
			let prime = basis[i];
			let octaves = i64::from(prime.ilog2());
			let mut interval = vec![0; basis.len()];
			(interval[0], interval[i]) = (-octaves, 1);

			// nominal_spellings writes the prime itself, so bring it down the same octaves.
			let nominal = nominal.map(|mut note| {
				note[0] -= octaves;
				note
			});
			let cost = |note: &[i64]| notation.cost(note).expect("always a valid spelling");
			// The cheapest spelling that isn't the one on the nominal.
			// It earns a place next to the nominal by being easier to read: either it is
			// cheaper, or it needs fewer accidentals, like A# next to vBb in meantone.
			let alternative = notation
				.spellings_interval(&interval, 2)
				.expect("always a valid interval")
				.into_iter()
				.find(|spelled| Some(spelled) != nominal.as_ref())
				.filter(|spelled| match nominal.as_deref() {
					Some(nominal) => {
						marks(spelled) < marks(nominal) || cost(spelled) < cost(nominal)
					},
					None => true,
				});

			PrimeSpelling { ratio: (u64::from(prime), 1 << octaves), nominal, alternative }
		})
		.collect()
}

// How many accidentals a spelling writes, not counting sharps and flats.
fn marks(spelling: &[i64]) -> i64 {
	spelling[2..].iter().map(|c| c.abs()).sum()
}

fn generator_ratios(notation: &Notation) -> Vec<(u64, u64)> {
	let mut generators = Vec::new();
	for g in notation.generators() {
		generators.push(
			notation
				.temperament()
				.subgroup()
				.to_ratio(g)
				.expect("always a valid ratio"),
		);
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

fn basis_vec(i: usize, n: usize) -> Vec<i64> {
	let mut unit = vec![0; n];
	unit[i] = 1;
	unit
}
