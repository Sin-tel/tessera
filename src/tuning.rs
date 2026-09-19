use serde::{Deserialize, Serialize};
use xen_utils::{Notation, Subgroup, Temperament};

// Number of fifths in the chain scales, and how far down from the root they start.
const DIATONIC: (usize, i64) = (7, 1);
const CHROMATIC: (usize, i64) = (12, 4);
const FINE: (usize, i64) = (31, 13);

// What gets serialized to save file.
// Either give `commas` to temper out, or `et` for an equal temperament.
// If neither is present, it is just intonation.
//
// n_accidentals and half_sharp match fields in NotationInfo

// TODO: why is 'skip_serializing_if' necessary on half_sharp but not other options?
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Definition {
	// Subgroup in shorthand, e.g. "2.3.5.7"
	pub subgroup: String,
	// Commas to temper out as "p/q" strings.
	#[serde(default)]
	pub commas: Vec<String>,
	// Equal division of the octave.
	#[serde(default)]
	pub et: Option<i64>,
	// missing in presets since it depends on notation chosen
	// TODO: kind of bad
	#[serde(default)]
	pub n_accidentals: usize,
	// Prime of an accidental that should be written as half a sharp.
	// mlua turns None into a null value that is truthy in Lua, so leave it out.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub half_sharp: Option<usize>,
}

#[derive(Debug)]
pub struct TuningSystem {
	notation: Notation,
	// size of each notation coordinate in semitones.
	pitches: Vec<f64>,
	// diatonic, chromatic, fine
	scales: [Vec<Vec<i64>>; 3],
	half_sharp: Option<usize>,
}

// info necessary to display notation options in menu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotationInfo {
	pub n_accidentals: usize,
	pub generators: Vec<(u64, u64)>,
	pub recommended: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub half_sharp: Option<usize>,
	pub spellings: Vec<PrimeSpelling>,
}

type Ratio = (u64, u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimeSpelling {
	pub ratio: Ratio,
	// notation coordinates
	pub note: Vec<i64>,
}

impl TuningSystem {
	pub fn new(def: &Definition) -> Result<Self, String> {
		let temperament = temperament(def)?;
		let notation = Notation::with_count(&temperament, def.n_accidentals).map_err(err)?;
		let pitches = (0..notation.len())
			.map(|i| {
				notation
					.pitch(&basis_vec(i, notation.len()))
					.map(|cents| cents / 100.0)
					.map_err(err)
			})
			.collect::<Result<Vec<_>, _>>()?;

		let half_sharp = def.half_sharp;
		let mut system = Self { notation, pitches, scales: Default::default(), half_sharp };
		let diatonic = system.chain_scale(DIATONIC.0, DIATONIC.1);
		let chromatic = system.chain_scale(CHROMATIC.0, CHROMATIC.1);
		let fine = system.fine_scale(&chromatic)?;
		system.scales = [diatonic, chromatic, fine];

		Ok(system)
	}

	pub fn get_notation_info(&self) -> NotationInfo {
		let recommended = self.notation.keeps_nominals();
		let generators = generator_ratios(&self.notation);
		let n_accidentals = self.notation.len() - 2;

		NotationInfo {
			n_accidentals,
			generators,
			recommended,
			half_sharp: self.half_sharp,
			spellings: prime_spellings(&self.notation),
		}
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
	//
	// Notes in one octave, sorted by pitch. The first one is always the unison.
	pub fn scale(&self, index: usize) -> &[Vec<i64>] {
		&self.scales[index]
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
	fn chain_scale(&self, n: usize, offset: i64) -> Vec<Vec<i64>> {
		let mut fifths: Vec<i64> = (0..n as i64).map(|i| i - offset).collect();
		fifths.sort_by_key(|&f| (f.abs(), -f));

		let mut seen = std::collections::HashSet::new();
		let mut scale = Vec::new();
		for f in fifths {
			let mut note = vec![0; self.len()];
			note[1] = f;
			self.reduce(&mut note);
			let image = self.notation.temper(&note).expect("note has the right length");
			if seen.insert(image) {
				scale.push(note);
			}
		}
		scale.sort_by(|a, b| self.pitch(a).total_cmp(&self.pitch(b)));
		scale
	}

	fn fine_scale(&self, chromatic: &[Vec<i64>]) -> Result<Vec<Vec<i64>>, String> {
		let rank = self.notation.temperament().rank();
		if rank > 1 {
			if self.len() == 2 {
				// No accidentals, so the only way to go finer is a longer chain
				return Ok(self.chain_scale(FINE.0, FINE.1));
			}

			// Chromatic scale with every accidental moved up and down once.
			// Notes that are the same in the temperament are only kept once,
			// the one with the fewest accidentals is found first.
			let mut shifts = vec![vec![0; self.len()]];
			for i in 2..self.len() {
				for sign in [1, -1] {
					let mut shift = vec![0; self.len()];
					shift[i] = sign;
					shifts.push(shift);
				}
			}

			let mut seen = std::collections::HashSet::new();
			let mut scale = Vec::new();
			for shift in &shifts {
				for note in chromatic {
					let mut note: Vec<i64> = note.iter().zip(shift).map(|(a, b)| a + b).collect();
					self.reduce(&mut note);
					if seen.insert(self.notation.temper(&note).map_err(err)?) {
						scale.push(note);
					}
				}
			}
			scale.sort_by(|a, b| self.pitch(a).total_cmp(&self.pitch(b)));
			return Ok(scale);
		}

		// Equal temperament: every step, spelled by the notation.
		let mut octave = vec![0; self.len()];
		octave[0] = 1;
		let steps = self.notation.temper(&octave).map_err(err)?[0].abs();

		(0..steps).map(|k| self.notation.spell(&[k]).map_err(err)).collect()
	}
}

pub fn notations(def: &Definition) -> Result<Vec<NotationInfo>, String> {
	let temperament = temperament(def)?;
	let mut notations = Vec::new();

	for notation in Notation::options(&temperament).map_err(err)?.into_iter() {
		let recommended = notation.keeps_nominals();
		let n_accidentals = notation.len() - 2;
		let generators = generator_ratios(&notation);
		let half_sharp = can_use_half_sharp(&notation);
		let spellings = prime_spellings(&notation);

		// If half sharp is available, but not on 33/32, add an extra notation without it.
		if let Some(half_sharp) = half_sharp {
			if generators[half_sharp - 1] != (33, 32) {
				notations.push(NotationInfo {
					n_accidentals,
					generators: generators.clone(),
					recommended,
					half_sharp: None,
					spellings: spellings.clone(),
				});
			}
		}

		notations.push(NotationInfo {
			n_accidentals,
			generators,
			recommended,
			half_sharp,
			spellings,
		});
	}
	Ok(notations)
}

fn temperament(def: &Definition) -> Result<Temperament, String> {
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

// If some accidental can be written as a half-sharp: two of them are the apotome
// (7 fifths down 4 octaves) in the temperament.
// By construction this can only appear at one accidental.
// Returns coordinate index
fn can_use_half_sharp(notation: &Notation) -> Option<usize> {
	let mut apotome = vec![0; notation.len()];
	(apotome[0], apotome[1]) = (-4, 7);
	let Ok(apotome) = notation.temper(&apotome) else {
		return None;
	};

	for i in 2..notation.len() {
		let image = notation.temper(&basis_vec(i, notation.len())).unwrap();
		if image.iter().zip(&apotome).all(|(a, b)| 2 * a == *b) {
			// only used by lua, so compensate for 1-indexing
			return Some(i + 1);
		}
	}
	None
}

// How every prime beyond 3 is written, reduced to an octave (5/4, 7/4, 11/8, ..).
fn prime_spellings(notation: &Notation) -> Vec<PrimeSpelling> {
	// TODO: this should be in xen_utils, use notation.nominal_spellings
	let basis = notation.subgroup().basis();
	(2..basis.len())
		.map(|i| {
			let prime = basis[i];
			let octaves = i64::from(prime.ilog2());
			let mut interval = vec![0; basis.len()];
			(interval[0], interval[i]) = (-octaves, 1);

			let ratio = (u64::from(prime), 1 << octaves);
			PrimeSpelling {
				ratio,
				note: notation.spell_interval(&interval).expect("always a valid interval"),
			}
		})
		.collect()
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

fn basis_vec(i: usize, n: usize) -> Vec<i64> {
	let mut unit = vec![0; n];
	unit[i] = 1;
	unit
}
