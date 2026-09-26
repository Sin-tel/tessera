//! Notation systems as linear maps.

use diophantine::{Matrix, cvp_l1_top_k, eye, kernel_left, lll, solve_diophantine, transpose};

use crate::Error;
use crate::notation_options::NotationOptions;
use crate::primes::Subgroup;
use crate::temperament::Temperament;
use crate::tuning::Tuning;
use crate::util::{LLL_DELTA, MAX_SEARCH_NODES, column, combination, first_column, subtract};

/// The nominals, in order of the fifth chain.
/// Nominals for note at `f` fifths spells `NOMINALS[(f + 1) mod 7]`.
const NOMINALS: [char; 7] = ['F', 'C', 'G', 'D', 'A', 'E', 'B'];

/// The octave number of the note with notation coordinates `(0, 0, ..)`.
const CENTRE_OCTAVE: i64 = 5;

/// Raising and lowering symbols for each accidental.
/// These are just arbitrary debug symbols. We have 11 symbols, so max 41-limit JI.
const ACCIDENTAL_SYMBOLS: [(char, char); 11] = [
    ('^', 'v'),
    ('>', '<'),
    ('t', 'd'),
    ('/', '\\'),
    ('+', '~'),
    ('*', '%'),
    ('\'', ','),
    ('!', '?'),
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
];

/// The largest interval, in cents, that counts as an accidental:
/// half an apotome, about 56.8 cents.
///
/// This lets every prime be written without augmented or diminished intervals.
/// 1200*log2(sqrt(2187/2048))
const MAX_ACCIDENTAL_CENTS: f64 = 56.842_503_028_855_52;

/// A notation system: a set of symbols, and what each one maps to in the temperament.
///
/// Notation coordinates are counts of notational generators. The first two are
/// always the octave `2/1` and the fifth `3/2`, which together give the
/// nominals and the sharps and flats. Other coordinates count accidentals,
/// which raises or lowers by a small interval that is not a sharp.
#[derive(Debug, Clone)]
pub struct Notation {
    /// The octave, the fifth, then the accidentals, as prime interval vectors.
    generators: Matrix<i64>,
    /// The accidental symbols.
    accidentals: Vec<(char, char)>,
    /// What each generator maps to in the temperament, one per row.
    /// This is the map from notation coordinates to tempered intervals.
    images: Matrix<i64>,
    /// A reduced basis of the written notes that map to unison, in notation coordinates.
    enharmonics: Matrix<i64>,
    /// The quadratic form standing in for [`spelling_cost`].
    weights: Matrix<f64>,
    temperament: Temperament,
    tuning: Tuning,
    /// Cents per notation coordinate under `tuning`.
    pitches: Vec<f64>,
}

impl Notation {
    /// Builds the notation of `subgroup` as just intonation, with an accidental
    /// for every prime beyond 3.
    ///
    /// Spelling is a bijection here: nothing is tempered out, so no two written
    /// spellings temper to the same thing and there are no enharmonics.
    ///
    /// # Errors
    /// Returns [`Error::Unsupported`] if the subgroup needs more accidentals
    /// than there are symbols.
    pub fn from_ji(subgroup: &Subgroup) -> Result<Self, Error> {
        let accidentals = derive_accidentals(subgroup);
        Notation::build(&Temperament::from_ji(subgroup)?, &accidentals)
    }

    /// Builds the recommended notation of `temperament`: the smallest of
    /// the run [`options`](Self::options) gives that
    /// [keeps every nominal](Self::keeps_nominals).
    ///
    /// # Errors
    /// Returns [`Error::Unsupported`] if no notation can be derived at all.
    pub fn from_temperament(temperament: &Temperament) -> Result<Self, Error> {
        Self::from_temperament_with_accidentals(
            temperament,
            &derive_accidentals(temperament.subgroup()),
        )
    }

    /// Builds the recommended notation of `temperament` given a set of `accidentals`.
    pub fn from_temperament_with_accidentals(
        temperament: &Temperament,
        accidentals: &[Vec<i64>],
    ) -> Result<Self, Error> {
        let options = Notation::options_with_accidentals(temperament, accidentals)?;
        for option in &options {
            if option.keeps_nominals() {
                return Ok(option.clone());
            }
        }
        Ok(options
            .first()
            .expect("there is always at least one option")
            .clone())
    }

    /// The best notation `temperament` offers of each size, smallest first.
    ///
    /// Every subset of the accidentals is a candidate. An accidental tempered
    /// out is never kept, nor one worth what an earlier accidental is worth,
    /// up to direction: in 41et `64/63` is worth what `81/80` is. A subset
    /// must reach every tempered interval, and an equal temperament keeping any
    /// accidental must keep one worth a single step.
    ///
    /// Of each size the best is the one with fewest primes off their nominals
    /// (see [`keeps_nominals`](Self::keeps_nominals)), then the lowest total
    /// [`nominal_costs`](Self::nominal_costs), then those costs compared one
    /// prime at a time, lowest first. The best of one size need not keep what
    /// the best of the size below keeps.
    ///
    /// # Errors
    /// Returns [`Error::Unsupported`] if no notation exists at all - an equal
    /// temperament whose fifth chain does
    /// not reach every note and which has no accidental worth a single step of
    /// it has none.
    pub fn options(temperament: &Temperament) -> Result<Vec<Self>, Error> {
        Self::options_with_accidentals(temperament, &derive_accidentals(temperament.subgroup()))
    }

    /// [`options`](Self::options) over the given `accidentals`.
    pub fn options_with_accidentals(
        temperament: &Temperament,
        accidentals: &[Vec<i64>],
    ) -> Result<Vec<Self>, Error> {
        NotationOptions::new(temperament, accidentals)?.search()
    }

    /// The best notation of `temperament` keeping exactly `count` accidentals,
    /// as ranked by [`options`](Self::options).
    ///
    /// # Errors
    /// Returns [`Error::Unsupported`] if no `count` of them make a notation.
    pub fn with_count(temperament: &Temperament, count: usize) -> Result<Self, Error> {
        Self::with_count_and_accidentals(
            temperament,
            &derive_accidentals(temperament.subgroup()),
            count,
        )
    }

    /// [`with_count`](Self::with_count) over the given `accidentals`.
    pub fn with_count_and_accidentals(
        temperament: &Temperament,
        accidentals: &[Vec<i64>],
        count: usize,
    ) -> Result<Self, Error> {
        NotationOptions::new(temperament, accidentals)?.with_count(count)
    }

    /// Builds the notation of `temperament` with `accidentals` as its extra
    /// generators.
    ///
    /// This is internal to crate since it doesn't do any checks.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if an accidental is not an interval
    /// of the temperament's subgroup, and [`Error::Unsupported`] if there are
    /// more accidentals than symbols, or if the generators do not reach every
    /// tempered interval.
    pub(crate) fn build(
        temperament: &Temperament,
        accidentals: &[Vec<i64>],
    ) -> Result<Self, Error> {
        let subgroup = temperament.subgroup();
        if accidentals.len() > ACCIDENTAL_SYMBOLS.len() {
            return Err(Error::Unsupported(format!(
                "{subgroup} needs {} accidentals, and there are only {} symbols",
                accidentals.len(),
                ACCIDENTAL_SYMBOLS.len()
            )));
        }

        let mut generators = fifth_chain(subgroup.dim());
        generators.extend(accidentals.to_vec());
        let images = temperament.temper_all(&generators)?;
        if !spans(&images, temperament.rank()) {
            return Err(Error::Unsupported(format!(
                "the octave, the fifth and these accidentals of {subgroup} do not reach every tempered interval of this rank {} temperament",
                temperament.rank()
            )));
        }

        // The written notes that map to unison.
        // Reduced so the CVP search has a good basis to work with.
        let weights = spelling_cost_l2(generators.len());
        let enharmonics = kernel_left(&images).expect("kernel is valid");
        let enharmonics = lll(&enharmonics, LLL_DELTA, &weights).unwrap_or(enharmonics);

        let tuning = Tuning::weil_euclidean(temperament);
        let pitches = generator_pitches(&images, &tuning).expect("pitches are valid");

        let symbols = ACCIDENTAL_SYMBOLS
            .iter()
            .take(accidentals.len())
            .cloned()
            .collect();

        Ok(Notation {
            generators,
            accidentals: symbols,
            images,
            enharmonics,
            weights,
            temperament: temperament.clone(),
            tuning,
            pitches,
        })
    }

    pub fn tuning(&self) -> &Tuning {
        &self.tuning
    }

    pub fn with_tuning(mut self, tuning: Tuning) -> Result<Self, Error> {
        if tuning.temperament() != &self.temperament {
            return Err(Error::Unsupported(
                "the tuning is of a different temperament".into(),
            ));
        }
        self.pitches = generator_pitches(&self.images, &tuning)?;
        self.tuning = tuning;
        Ok(self)
    }

    /// The size in cents of a spelling under the notation's tuning.
    pub fn pitch(&self, spelling: &[i64]) -> Result<f64, Error> {
        self.check(spelling)?;
        Ok(spelling
            .iter()
            .zip(&self.pitches)
            .map(|(&count, cents)| count as f64 * cents)
            .sum())
    }

    /// What a spelling costs to read: a sharp is worth two accidental marks, and
    /// the fifths count from `D`. [`spellings`](Self::spellings) returns the
    /// cheapest spellings of a tempered interval first, and this compares
    /// spellings it did not rank against each other, such as the one
    /// [`nominal_spellings`](Self::nominal_spellings) picks.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `spelling` does not have one
    /// entry per notation coordinate.
    pub fn cost(&self, spelling: &[i64]) -> Result<i64, Error> {
        self.check(spelling)?;
        Ok(spelling_cost(spelling))
    }

    /// The notational generators as prime interval vectors, in the order their
    /// notation coordinates count them: the octave, the fifth, then the
    /// accidentals. Every accidental is an ascending interval.
    pub fn generators(&self) -> &Matrix<i64> {
        &self.generators
    }

    /// A reduced basis of the enharmonic lattice, in notation coordinates: the
    /// written notes the temperament calls a unison.
    ///
    /// This is the freedom the notation has over the temperament, so it has
    /// `rank() - temperament.rank()` members and is empty exactly when spelling
    /// is a bijection. Every notation of an equal temperament has one, since a
    /// notation is free on its octave, its fifth and its accidentals and so can
    /// never close the circle of fifths.
    pub fn enharmonics(&self) -> &Matrix<i64> {
        &self.enharmonics
    }

    /// The temperament being notated.
    pub fn temperament(&self) -> &Temperament {
        &self.temperament
    }

    /// The just intonation subgroup of the temperament being notated.
    pub fn subgroup(&self) -> &Subgroup {
        self.temperament.subgroup()
    }

    /// The number of notation coordinates: the octave, the fifth, then one per
    /// accidental. The length of a spelling.
    // A notation always has at least the octave and the fifth.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.generators.len()
    }

    /// The dimension of the subgroup being notated: the length of a just
    /// interval.
    pub fn dim(&self) -> usize {
        self.subgroup().dim()
    }

    /// The tempered interval a spelling stands for.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `spelling` does not have one
    /// entry per notation coordinate.
    pub fn temper(&self, spelling: &[i64]) -> Result<Vec<i64>, Error> {
        self.check(spelling)?;
        Ok(combination(spelling, &self.images, self.temperament.rank()))
    }

    /// The just interval a spelling reads as, taking each accidental as the
    /// interval it is named for.
    ///
    /// This is the spelling's literal reading, one of the many just intervals
    /// that temper to the same thing. [`Simplifier`](crate::Simplifier) finds
    /// the simplest.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `spelling` does not have one
    /// entry per notation coordinate.
    pub fn to_interval(&self, spelling: &[i64]) -> Result<Vec<i64>, Error> {
        self.check(spelling)?;
        Ok(combination(spelling, &self.generators, self.dim()))
    }

    /// The best way to write a tempered interval.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `tempered` does not have one
    /// entry per generator of the temperament, and [`Error::Unsupported`] if
    /// the notation cannot write it.
    pub fn spell(&self, tempered: &[i64]) -> Result<Vec<i64>, Error> {
        Ok(self.spellings(tempered, 1)?.swap_remove(0))
    }

    /// [`spell`](Self::spell) the tempered interval a just interval maps to.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `interval` does not have one
    /// entry per basis element of the subgroup, and [`Error::Unsupported`] if
    /// the notation cannot write it.
    pub fn spell_interval(&self, interval: &[i64]) -> Result<Vec<i64>, Error> {
        self.spell(&self.temperament.temper(interval)?)
    }

    /// The `count` best ways to write a tempered interval, best first: the
    /// cheapest under the spelling cost, ties going to the quadratic form and
    /// then to the enharmonic. At least one, and exact, so the search only
    /// closes once it holds `count` spellings: ask for the handful wanted.
    ///
    /// Exact within [`MAX_SEARCH_NODES`]. A tempered
    /// interval absurdly far from a unison exhausts that budget and is answered
    /// with the best spellings found, which may be fewer than `count`.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `tempered` does not have one
    /// entry per generator of the temperament.
    pub fn spellings(&self, tempered: &[i64], count: usize) -> Result<Matrix<i64>, Error> {
        Ok(self.spellings_within_budget(tempered, count)?.0)
    }

    /// [`spellings`](Self::spellings), and whether the search closed rather than
    /// running out of [`MAX_SEARCH_NODES`]. Only a test reads the flag.
    pub(crate) fn spellings_within_budget(
        &self,
        tempered: &[i64],
        count: usize,
    ) -> Result<(Matrix<i64>, bool), Error> {
        let rank = self.temperament.rank();
        if tempered.len() != rank {
            return Err(Error::InvalidDimensions(format!(
                "tempered interval has {} entries, expected {rank}",
                tempered.len()
            )));
        }
        // `build` refuses a notation whose generators do not span, so every
        // tempered interval has a spelling.
        let solution = solve_diophantine(&transpose(&self.images), &column(tempered))
            .expect("the generators reach every tempered interval");
        Ok(cheapest(&first_column(&solution), &self.enharmonics, count))
    }

    /// [`spellings`](Self::spellings) of the tempered interval a just interval
    /// maps to.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `interval` does not have one
    /// entry per basis element of the subgroup, and [`Error::Unsupported`] if
    /// the notation cannot write it.
    pub fn spellings_interval(&self, interval: &[i64], count: usize) -> Result<Matrix<i64>, Error> {
        self.spellings(&self.temperament.temper(interval)?, count)
    }

    /// The cheapest way to write each prime beyond 3 on its just nominal: at
    /// the degree just intonation gives it, or `None` where no spelling of the
    /// tempered prime has that degree.
    ///
    /// The degree is exact rather than modulo the seven nominals: a note ten
    /// sharps up an octave lower is not on the nominal, though its letter is.
    ///
    /// This asks for the prime on its letter, whatever [`spell`](Self::spell)
    /// would choose: 41et's largest notation spells `7/4` as `tA`, but writes
    /// it on its nominal as `vBb`.
    pub fn nominal_spellings(&self) -> Vec<Option<Vec<i64>>> {
        // The notation coordinates and their degree side by side, so that one
        // solve finds a spelling of the right tempered interval at the right degree.
        let with_degree: Matrix<i64> = self
            .images
            .iter()
            .enumerate()
            .map(|(index, image)| {
                let mut row = image.clone();
                row.push(GENERATOR_DEGREES.get(index).copied().unwrap_or(0));
                row
            })
            .collect();
        // The enharmonics that keep the degree, reduced for the search.
        let level = kernel_left(&with_degree).expect("the kernel is always valid");
        let level = lll(&level, LLL_DELTA, &self.weights).unwrap_or(level);

        let mut spellings = Vec::new();
        for (index, nominal) in (2..self.dim()).zip(just_nominals(self.subgroup())) {
            let mut prime = vec![0i64; self.dim()];
            prime[index] = 1;
            let mut target = self
                .temperament
                .temper(&prime)
                .expect("accidental can always be tempered");
            target.push(nominal.degree);
            let spelling = match solve_diophantine(&transpose(&with_degree), &column(&target)) {
                Ok(solution) => Some(cheapest(&first_column(&solution), &level, 1).0.remove(0)),
                Err(_) => None,
            };
            spellings.push(spelling);
        }
        spellings
    }

    /// What [`nominal_spellings`](Self::nominal_spellings) cost to read.
    pub fn nominal_costs(&self) -> Vec<Option<i64>> {
        self.nominal_spellings()
            .iter()
            .map(|spelling| spelling.as_deref().map(spelling_cost))
            .collect()
    }

    /// Whether every prime beyond 3 can be written on the nominal just
    /// intonation gives it, at a cost no worse than either just intonation
    /// spends on it or this notation spends on its cheapest spelling of it.
    pub fn keeps_nominals(&self) -> bool {
        self.nominal_verdict().failures == 0
    }

    /// [`nominal_costs`](Self::nominal_costs), and how many primes fail
    /// [`keeps_nominals`](Self::keeps_nominals).
    pub(crate) fn nominal_verdict(&self) -> NominalVerdict {
        let costs = self.nominal_costs();
        let nominals = just_nominals(self.subgroup());
        let mut failures = 0;
        for ((index, cost), nominal) in (2..self.dim()).zip(&costs).zip(nominals) {
            let mut prime = vec![0i64; self.dim()];
            prime[index] = 1;
            let best = spelling_cost(
                &self
                    .spell_interval(&prime)
                    .expect("Prime is a proper interval"),
            );
            if cost.is_none_or(|cost| cost > best.max(nominal.cost)) {
                failures += 1;
            }
        }
        NominalVerdict { costs, failures }
    }

    /// Checks that `spelling` has one entry per notation coordinate.
    fn check(&self, spelling: &[i64]) -> Result<(), Error> {
        if spelling.len() != self.len() {
            return Err(Error::InvalidDimensions(format!(
                "spelling has {} entries, expected {}",
                spelling.len(),
                self.len()
            )));
        }
        Ok(())
    }

    /// Writes a spelling as a note in scientific pitch notation, such as `C5`,
    /// `Eb4`, `vE5`.
    ///
    /// The spelling is read as an interval up from `C5`, so `(0, 0, ..)` is
    /// `C5` itself and `(0, 1, 0, ..)`, a fifth up, is `G5`. Accidentals come
    /// before the nominal and sharps and flats after it. The symbols are
    /// placeholders for debugging, since real microtonal accidentals are not in
    /// unicode.
    ///
    /// # Panics
    /// Panics if `spelling` does not have one entry per notation coordinate.
    pub fn note(&self, spelling: &[i64]) -> String {
        assert_eq!(
            spelling.len(),
            self.len(),
            "spelling must have one entry per notation coordinate"
        );
        let (octaves, fifths) = (spelling[0], spelling[1]);

        let mut name = String::new();
        for (&count, (up, down)) in spelling[2..].iter().zip(&self.accidentals) {
            let symbol = if count < 0 { down } else { up };
            name.extend(std::iter::repeat_n(symbol, count.unsigned_abs() as usize));
        }

        // The fifth chain runs F C G D A E B and then wraps round to F#, so
        // both the nominal and the number of sharps come from `fifths + 1`.
        name.push(NOMINALS[(fifths + 1).rem_euclid(7) as usize]);
        let sharps = (fifths + 1).div_euclid(7);
        name.push_str(&if sharps < 0 { "b" } else { "#" }.repeat(sharps.unsigned_abs() as usize));

        // An octave is seven nominals and a fifth is four, which fixes how far
        // up the staff the note sits and so which octave it lands in.
        let steps = 7 * octaves + 4 * fifths;
        name.push_str(&(CENTRE_OCTAVE + steps.div_euclid(7)).to_string());

        name
    }
}

/// The octave `2/1` and the fifth `3/2` as interval vectors over a subgroup of
/// `dim` primes. Every notation is generated by these two and its accidentals.
fn generator_pitches(images: &Matrix<i64>, tuning: &Tuning) -> Result<Vec<f64>, Error> {
    images.iter().map(|image| tuning.pitch(image)).collect()
}

pub(crate) fn fifth_chain(dim: usize) -> Matrix<i64> {
    let mut octave = vec![0i64; dim];
    octave[0] = 1;
    let mut fifth = vec![0i64; dim];
    (fifth[0], fifth[1]) = (-1, 1);
    vec![octave, fifth]
}

/// How well a notation keeps its nominals.
pub(crate) struct NominalVerdict {
    /// [`Notation::nominal_costs`].
    pub(crate) costs: Vec<Option<i64>>,
    /// How many primes fail [`Notation::keeps_nominals`].
    pub(crate) failures: usize,
}

/// Where just intonation writes a prime: its degree, and what that costs.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JustNominal {
    /// Letters up from the unison, seven to the octave.
    pub(crate) degree: i64,
    /// [`spelling_cost`] of the just spelling.
    pub(crate) cost: i64,
}

/// The degree of the octave and the fifth. Every accidental has degree zero.
const GENERATOR_DEGREES: [i64; 2] = [7, 4];

/// The degree of notation coordinates: how many letters up they sit.
pub(crate) fn degree(coordinates: &[i64]) -> i64 {
    GENERATOR_DEGREES
        .iter()
        .zip(coordinates)
        .map(|(d, c)| d * c)
        .sum()
}

/// Where just intonation writes each prime beyond 3.
///
/// Just intonation writes a prime as its own accidental on top of a stretch of
/// the fifth chain. The accidental `a` has exponent `s = +-1` on the prime, so
/// the prime is `s * a` less `s * a[1]` threes and `s * a[0]` twos: the fifth
/// coordinate `-s * a[1]`, the octave coordinate `-s * (a[0] + a[1])` and one
/// mark. Together with 7 for the octave and 11 for the three, these degrees
/// are a linear map from interval vectors to degrees, `(7, 11, 16, 20, 24, ..)`,
/// whose kernel holds the apotome and every derived accidental.
pub(crate) fn just_nominals(subgroup: &Subgroup) -> Vec<JustNominal> {
    (2..subgroup.dim())
        .map(|index| {
            let a = derive_accidental_vector(subgroup, index);
            let s = a[index];
            let spelling = [-s * (a[0] + a[1]), -s * a[1], 1];
            JustNominal {
                degree: degree(&spelling),
                cost: spelling_cost(&spelling),
            }
        })
        .collect()
}

/// Derives the default accidentals for a subgroup.
pub fn derive_accidentals(subgroup: &Subgroup) -> Matrix<i64> {
    let mut result = Vec::new();
    for index in 2..subgroup.dim() {
        result.push(derive_accidental_vector(subgroup, index));
    }
    result
}

// How far up and down the fifth chain to look for an accidental.
// Since we guarantee no augmented/diminshed offsets, this should never be larger than 7.
// Make it bigger for some slack if we ever try different offsets.
const MAX_FIFTH_OFFSET: i64 = 12;

/// Chooses the accidental for the prime at `index` of `subgroup`: the smallest
/// detour along the fifth chain that lands within [`MAX_ACCIDENTAL_CENTS`] of
/// the prime.
///
/// The prime is compared to an interval of 0, 1, -1, 2, -2, .. fifths,
/// each reduced by octaves. The first one small enough wins, returned as an
/// ascending interval.
/// This gives `81/80` for 5, `64/63` for 7 and `33/32` for 11.
///
/// # Errors
/// Returns [`Error::Unsupported`] if no offset within [`MAX_FIFTH_OFFSET`]
/// gives a small enough interval.
pub(crate) fn derive_accidental_vector(subgroup: &Subgroup, index: usize) -> Vec<i64> {
    let offsets = std::iter::once(0).chain((1..=MAX_FIFTH_OFFSET).flat_map(|k| [k, -k]));
    for offset in offsets {
        let mut interval = vec![0i64; subgroup.dim()];
        interval[index] = 1;
        // We subtract k fifths, so negative
        interval[1] = -offset;

        // Octave reduction: whichever power of two lands closest to unison.
        interval[0] = -(subgroup.to_cents(&interval) / 1200.0).round_ties_even() as i64;

        if subgroup.to_cents(&interval).abs() < MAX_ACCIDENTAL_CENTS {
            return subgroup.ascending(&interval);
        }
    }
    unreachable!("We should always found an accidental within {MAX_FIFTH_OFFSET} fifths");
}

/// The middle of the seven naturals, as a fifth coordinate.
///
/// `F C G D A E B` are the fifth coordinates `-1` to `5`, so `D` at `2` is the
/// middle of them.
const NOMINAL_CENTRE: i64 = 2;

/// A sharp is worth two accidental marks.
const COST_FIFTH: i64 = 2;
const COST_MARK: i64 = 7;

/// What a written note costs to read.
///
/// The fifths are counted from [`NOMINAL_CENTRE`] (D) rather than from `C`, since
/// measuring from `C` makes the flat side cheaper.
/// The octave does not appear at all.
fn spelling_cost(coordinates: &[i64]) -> i64 {
    let marks: i64 = coordinates[2..].iter().map(|c| c.abs()).sum();
    COST_MARK * marks + COST_FIFTH * (coordinates[1] - NOMINAL_CENTRE).abs()
}

/// What each notation coordinate costs, once for each unit away from the
/// centre: nothing for the octave, [`COST_FIFTH`] for the fifth and
/// [`COST_MARK`] for each accidental. [`spelling_cost`] is the weighted L1 norm
/// under these, around [`NOMINAL_CENTRE`].
fn spelling_cost_weights(len: usize) -> Vec<i64> {
    (0..len)
        .map(|slot| match slot {
            0 => 0,
            1 => COST_FIFTH,
            _ => COST_MARK,
        })
        .collect()
}

/// The `count` cheapest spellings in the coset of `spelling` modulo `lattice`,
/// best first, and at least one.
///
/// [`spelling_cost`] is a weighted L1 norm, so this is an exact closest vector
/// search under it. Ties go to the quadratic form, then to the lattice vector.
/// `lattice` should be reduced under [`spelling_cost_l2`] for speed.
fn cheapest(spelling: &[i64], lattice: &Matrix<i64>, count: usize) -> (Matrix<i64>, bool) {
    if lattice.is_empty() {
        return (vec![spelling.to_vec()], true);
    }
    let mut centred = spelling.to_vec();
    centred[1] -= NOMINAL_CENTRE;
    let weights = spelling_cost_weights(spelling.len());
    let (enharmonics, complete) = cvp_l1_top_k(
        &centred,
        lattice,
        &weights,
        count.max(1),
        Some(MAX_SEARCH_NODES),
    )
    .expect("no overflow occurs for any reasonable temperament");
    let spellings = enharmonics
        .iter()
        .map(|enharmonic| subtract(spelling, enharmonic))
        .collect();
    (spellings, complete)
}

/// Whether generators with these `images` reach every tempered interval of a
/// rank `rank` temperament. Where they do not, some cannot be written at all.
pub(crate) fn spans(images: &Matrix<i64>, rank: usize) -> bool {
    let identity: Matrix<i64> = eye(rank);
    solve_diophantine(&transpose(images), &identity).is_ok()
}

/// A quadratic stand-in for [`spelling_cost`], for reducing a lattice: the
/// squares of [`spelling_cost_weights`] on the diagonal.
///
/// Register costs nothing, so the form is only semidefinite, but no enharmonic
/// is a stack of octaves and it is definite on every lattice reduced under it.
fn spelling_cost_l2(len: usize) -> Matrix<f64> {
    let weights = spelling_cost_weights(len);
    (0..len)
        .map(|row| {
            (0..len)
                .map(|col| {
                    if row == col {
                        (weights[row] * weights[row]) as f64
                    } else {
                        0.0
                    }
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Simplifier;

    fn notation(subgroup: &str) -> Notation {
        Notation::from_ji(&subgroup.parse::<Subgroup>().unwrap()).unwrap()
    }

    /// The notation of the temperament of `subgroup` that tempers out `commas`.
    fn tempered(subgroup: &str, commas: &[(u64, u64)]) -> Notation {
        let subgroup: Subgroup = subgroup.parse().unwrap();
        let commas: Vec<Vec<i64>> = commas
            .iter()
            .map(|&(num, den)| subgroup.factorize(num, den).unwrap())
            .collect();
        let temperament = Temperament::from_commas(&commas, &subgroup).unwrap();
        Notation::from_temperament(&temperament).unwrap()
    }

    /// Every notation the equal temperament of `divisions` over `subgroup` offers.
    fn et_options(divisions: i64, subgroup: &str) -> Vec<Notation> {
        let subgroup: Subgroup = subgroup.parse().unwrap();
        let temperament = Temperament::equal(divisions, &subgroup).unwrap();
        Notation::options(&temperament).unwrap()
    }

    /// Every notation the temperament of `subgroup` tempering out `commas` offers.
    fn options_of(subgroup: &str, commas: &[(u64, u64)]) -> Vec<Notation> {
        let subgroup: Subgroup = subgroup.parse().unwrap();
        let commas: Vec<Vec<i64>> = commas
            .iter()
            .map(|&(num, den)| subgroup.factorize(num, den).unwrap())
            .collect();
        let temperament = Temperament::from_commas(&commas, &subgroup).unwrap();
        Notation::options(&temperament).unwrap()
    }

    /// The ranks of a run of notations.
    fn ranks(options: &[Notation]) -> Vec<usize> {
        options.iter().map(Notation::len).collect()
    }

    /// The accidentals of a notation, as ratios.
    fn accidental_ratios(n: &Notation) -> Vec<(u64, u64)> {
        n.generators()[2..]
            .iter()
            .map(|a| n.subgroup().to_ratio(a).unwrap())
            .collect()
    }

    /// The note name of a ratio, read as an interval up from C5.
    fn note_of(n: &Notation, num: u64, den: u64) -> String {
        let interval = n.subgroup().factorize(num, den).unwrap();
        n.note(&n.spell_interval(&interval).unwrap())
    }

    #[test]
    fn pythagorean_mapping() {
        let n = notation("2.3");
        // 2/1 is an octave and 3/1 is an octave plus a fifth.
        assert_eq!(n.spell_interval(&[1, 0]).unwrap(), vec![1, 0]);
        assert_eq!(n.spell_interval(&[0, 1]).unwrap(), vec![1, 1]);
        assert_eq!(n.len(), 2);
        assert_eq!(n.dim(), 2);
        assert!(accidental_ratios(&n).is_empty());
    }

    #[test]
    fn generators_map_to_unit_vectors() {
        for subgroup in ["2.3", "2.3.5", "2.3.7", "2.3.11", "2.3.5.7.11"] {
            let n = notation(subgroup);
            for (index, generator) in n.generators().iter().enumerate() {
                let mut unit = vec![0; n.len()];
                unit[index] = 1;
                assert_eq!(n.spell_interval(generator).unwrap(), unit);
            }
        }
    }

    #[test]
    fn accidentals_are_small_and_ascending() {
        let n = notation("2.3.5.7.11.13");
        for a in &n.generators()[2..] {
            let cents = n.subgroup().to_cents(a);
            assert!(cents > 0.0 && cents < MAX_ACCIDENTAL_CENTS);
        }
    }

    #[test]
    fn five_limit_mapping() {
        let n = notation("2.3.5");
        // 5 is four fifths up, lowered by a syntonic comma.
        assert_eq!(n.spell_interval(&[0, 0, 1]).unwrap(), vec![0, 4, -1]);
        assert_eq!(
            n.generators(),
            &vec![vec![1, 0, 0], vec![-1, 1, 0], vec![-4, 4, -1]]
        );
    }

    #[test]
    fn spelling_checks_dimensions() {
        let n = notation("2.3");
        assert!(n.spell_interval(&[1, 0, 0]).is_err());
        assert!(n.spell_interval(&[1]).is_err());
    }

    #[test]
    fn notes_along_the_fifth_chain() {
        let n = notation("2.3");
        // The nominals, as fifths up and down from C5 with no octave shift.
        assert_eq!(n.note(&[0, 0]), "C5");
        assert_eq!(n.note(&[0, 1]), "G5");
        assert_eq!(n.note(&[0, 2]), "D6");
        assert_eq!(n.note(&[0, -1]), "F4");
        assert_eq!(n.note(&[0, -2]), "Bb3");
        assert_eq!(n.note(&[0, 6]), "F#8");
        assert_eq!(n.note(&[0, 13]), "F##12");
        assert_eq!(n.note(&[0, -8]), "Fb0");
    }

    #[test]
    fn notes_across_octaves() {
        let n = notation("2.3");
        assert_eq!(n.note(&[1, 0]), "C6");
        assert_eq!(n.note(&[-1, 0]), "C4");
        assert_eq!(n.note(&[3, 0]), "C8");
        assert_eq!(n.note(&[-3, 5]), "B4");
        assert_eq!(n.note(&[-2, 5]), "B5");
    }

    #[test]
    fn notes_from_ratios() {
        let n = notation("2.3");
        assert_eq!(note_of(&n, 1, 1), "C5");
        assert_eq!(note_of(&n, 3, 2), "G5");
        assert_eq!(note_of(&n, 9, 8), "D5");
        assert_eq!(note_of(&n, 4, 3), "F5");
        assert_eq!(note_of(&n, 16, 9), "Bb5");
        assert_eq!(note_of(&n, 2, 1), "C6");
        assert_eq!(note_of(&n, 1, 2), "C4");
        assert_eq!(note_of(&n, 81, 64), "E5");
        assert_eq!(note_of(&n, 2187, 2048), "C#5");
    }

    #[test]
    fn accidentals_match_fjs() {
        let n = notation("2.3.5.7.11.13.17.19.23.29.31.37.41");
        // Note we always use ascending intervals, unlike FJS which only uses otonal ones.
        // We also use sqrt(2187/2048) instead of the original 65/63, which only changes prime 31.
        let fjs_accidentals = [
            (81, 80),
            (64, 63),
            (33, 32),
            (1053, 1024),
            (4131, 4096),
            (513, 512),
            (736, 729),
            (261, 256),
            (32, 31),
            (37, 36),
            (82, 81),
        ];
        for (i, a) in n.generators()[2..].iter().enumerate() {
            let ratio = n.subgroup().to_ratio(a).unwrap();
            assert_eq!(ratio, fjs_accidentals[i])
        }
    }

    #[test]
    fn a_subgroup_past_the_symbols_is_refused() {
        // Eleven pairs of symbols, so the 41 limit is the largest that can be
        // notated: the octave, the fifth and one accidental per prime beyond 3.
        assert_eq!(Notation::from_ji(&Subgroup::p_limit(41)).unwrap().len(), 13);
        assert!(Notation::from_ji(&Subgroup::p_limit(43)).is_err());
    }

    #[test]
    fn nothing_musical_runs_out_of_budget() {
        // The budget is a safety valve for absurd input, so an ordinary note
        // must never reach it: within a few octaves every search must close.
        let mut checked = 0;
        for divisions in 5..=100 {
            for sub in ["2.3.5", "2.3.5.7", "2.3.5.7.11"] {
                let subgroup: Subgroup = sub.parse().unwrap();
                let Ok(t) = Temperament::equal(divisions, &subgroup) else {
                    continue;
                };
                let Ok(n) = Notation::from_temperament(&t) else {
                    continue;
                };
                let simplifier = Simplifier::new(&t);
                for steps in [-2 * divisions, -divisions, 0, divisions, 4 * divisions] {
                    let (_, complete) = n.spellings_within_budget(&[steps], 3).unwrap();
                    assert!(complete, "spelling {steps} of {divisions}et over {sub}");
                    let (_, complete) = simplifier
                        .simplifications_within_budget(&[steps], 3)
                        .unwrap();
                    assert!(complete, "simplifying {steps} of {divisions}et over {sub}");
                    checked += 1;
                }
            }
        }
        assert!(checked > 500, "only {checked} searches swept");
    }

    #[test]
    fn a_target_too_far_to_search_still_answers() {
        // Past the budget the answers are the best found rather than the best
        // there is, and there may be fewer of them - but they are still
        // spellings of what was asked for, and they still come back quickly.
        let subgroup: Subgroup = "2.3.5.7.11".parse().unwrap();
        let t = Temperament::equal(41, &subgroup).unwrap();
        let n = Notation::from_temperament(&t).unwrap();
        for steps in [1_000_000i64, 1_000_000_000, i64::MAX / 2] {
            let spellings = n.spellings(&[steps], 3).unwrap();
            assert!(!spellings.is_empty());
            for spelling in &spellings {
                assert_eq!(n.temper(spelling).unwrap(), vec![steps]);
            }
        }
    }

    #[test]
    fn notes_with_accidentals() {
        assert_eq!(note_of(&notation("2.3.5"), 5, 4), "vE5");
        assert_eq!(note_of(&notation("2.3.7"), 7, 4), "vBb5");
        assert_eq!(note_of(&notation("2.3.11"), 11, 8), "^F5");

        let n = notation("2.3.5.7.11");
        assert_eq!(note_of(&n, 5, 4), "vE5");
        assert_eq!(note_of(&n, 7, 4), "<Bb5");
        assert_eq!(note_of(&n, 11, 8), "tF5");
        assert_eq!(note_of(&n, 25, 16), "vvG#5");
        assert_eq!(note_of(&n, 35, 32), "v<D5");
    }

    #[test]
    fn tempering_out_an_accidental_drops_it() {
        // Meantone tempers out 81/80 and archytas 64/63, which are exactly the
        // accidentals for 5 and 7, so both are notated as pythagorean is.
        let pythagorean = notation("2.3");
        for (subgroup, comma) in [("2.3.5", (81, 80)), ("2.3.7", (64, 63))] {
            let n = tempered(subgroup, &[comma]);
            assert_eq!(n.len(), 2);
            assert!(accidental_ratios(&n).is_empty());
            // The comma is a unison, so it is written as one.
            let comma = n.subgroup().factorize(comma.0, comma.1).unwrap();
            assert_eq!(n.spell_interval(&comma).unwrap(), vec![0; n.len()]);
            // The octave and the fifth are still what they always were.
            assert_eq!(n.generators()[0][..2], pythagorean.generators()[0][..]);
            assert_eq!(n.generators()[1][..2], pythagorean.generators()[1][..]);
        }
    }

    #[test]
    fn tempered_primes_land_on_nominals() {
        // The meantone third is a plain E and the archytas seventh a plain Bb,
        // where just intonation needs an accidental on each.
        let meantone = tempered("2.3.5", &[(81, 80)]);
        assert_eq!(meantone.spell_interval(&[0, 0, 1]).unwrap(), vec![0, 4]);
        assert_eq!(note_of(&meantone, 5, 4), "E5");
        assert_eq!(note_of(&meantone, 81, 64), "E5");

        let archytas = tempered("2.3.7", &[(64, 63)]);
        assert_eq!(archytas.spell_interval(&[0, 0, 1]).unwrap(), vec![4, -2]);
        assert_eq!(note_of(&archytas, 7, 4), "Bb5");
        assert_eq!(note_of(&archytas, 16, 9), "Bb5");
    }

    #[test]
    fn only_tempered_accidentals_are_dropped() {
        // Marvel tempers out 225/224, which is neither accidental, so both stay
        // and the notation spells as the just intonation one does. The two are
        // not equal, since each carries the temperament it notates.
        let marvel = tempered("2.3.5.7", &[(225, 224)]);
        assert_eq!(marvel.len(), 4);
        assert_eq!(marvel.generators(), notation("2.3.5.7").generators());
        assert_ne!(marvel.temperament(), notation("2.3.5.7").temperament());

        // Septimal meantone tempers out 81/80 as well, dropping just that one.
        let n = tempered("2.3.5.7", &[(81, 80), (225, 224)]);
        assert_eq!(accidental_ratios(&n), vec![(64, 63)]);
        assert_eq!(note_of(&n, 5, 4), "E5");
        assert_eq!(note_of(&n, 7, 4), "vBb5");
    }

    #[test]
    fn notation_12et_is_meantone() {
        // 12et tempers out 81/80, so it has the same notation as meantone.
        let subgroup = Subgroup::p_limit(5);
        let n = Notation::from_temperament(&Temperament::equal(12, &subgroup).unwrap()).unwrap();
        let meantone = tempered("2.3.5", &[(81, 80)]);
        assert_eq!(n.generators(), meantone.generators());
        assert_eq!(n.generators(), meantone.generators());
        assert_eq!(note_of(&n, 5, 4), "E5");
    }

    #[test]
    fn the_smallest_notation_drops_an_accidental_that_is_not_tempered_out() {
        // Septimal meantone has two notations. Keeping the accidental for 7
        // writes the harmonic seventh as a lowered Bb; the smallest writes the
        // same tempered interval with sharps and flats alone, as A#.
        let options = options_of("2.3.5.7", &[(81, 80), (225, 224)]);
        assert_eq!(ranks(&options), vec![2, 3]);

        assert!(accidental_ratios(&options[0]).is_empty());
        assert_eq!(note_of(&options[0], 5, 4), "E5");
        assert_eq!(note_of(&options[1], 5, 4), "E5");
        assert_eq!(note_of(&options[0], 7, 4), "A#5");
        assert_eq!(note_of(&options[1], 7, 4), "vBb5");
    }

    #[test]
    fn the_smallest_notation_has_the_rank_of_the_temperament() {
        // Schismatic reaches 5 along the fifth chain too, eight fifths down,
        // so its just third is written as a diminished fourth.
        let schismatic = options_of("2.3.5", &[(32805, 32768)]);
        assert_eq!(schismatic[0].len(), 2);
        assert_eq!(note_of(&schismatic[0], 5, 4), "Fb5");

        // Marvel is rank 3, so its smallest notation keeps one accidental. Only
        // the one for 5 works; keeping the one for 7 leaves 5 unreachable.
        let marvel = options_of("2.3.5.7", &[(225, 224)]);
        assert_eq!(marvel[0].len(), 3);
        assert_eq!(accidental_ratios(&marvel[0]), vec![(81, 80)]);
        assert_eq!(note_of(&marvel[0], 5, 4), "vE5");
        assert_eq!(note_of(&marvel[0], 7, 4), "vvA#5");

        // Where the smallest already reaches the rank of the temperament and
        // keeps its nominals, it is what is recommended too.
        for (subgroup, comma) in [("2.3.5", (81, 80)), ("2.3.7", (64, 63))] {
            assert_eq!(
                options_of(subgroup, &[comma])[0].generators(),
                tempered(subgroup, &[comma]).generators()
            );
        }
    }

    #[test]
    fn equal_temperament_offers_the_step_when_the_chain_reaches() {
        // The fifth chain of 22et reaches every note, so the fifth chain alone
        // is a notation and the step accidental is only an offer. Its major
        // third at nine fifths up is a defining property, not a mistake.
        let options = et_options(22, "2.3.5.7");
        assert_eq!(ranks(&options), vec![2, 3]);
        assert!(accidental_ratios(&options[0]).is_empty());
        assert_eq!(note_of(&options[0], 5, 4), "D#5");
        assert_eq!(note_of(&options[0], 7, 4), "Bb5");
        // Its step is the syntonic comma, which puts 5 back on E.
        assert_eq!(accidental_ratios(&options[1]), vec![(81, 80)]);
        assert_eq!(note_of(&options[1], 5, 4), "vE5");
        assert_eq!(note_of(&options[1], 7, 4), "Bb5");

        // 31et is meantone, so 64/63 is its step instead and the two swap over.
        let options = et_options(31, "2.3.5.7");
        assert_eq!(ranks(&options), vec![2, 3]);
        assert_eq!(note_of(&options[0], 5, 4), "E5");
        assert_eq!(note_of(&options[0], 7, 4), "A#5");
        assert_eq!(accidental_ratios(&options[1]), vec![(64, 63)]);
        assert_eq!(note_of(&options[1], 7, 4), "vBb5");
    }

    #[test]
    fn equal_temperament_stacks_further_accidentals_on_the_step() {
        // 41et: the fifth chain, then one accidental worth a step, then a
        // second for 11. The 11 is worth two steps, so with only the step it is
        // written as two of them. 64/63 is worth a step as well, so it is the
        // step over again and is never offered as a second accidental - but it
        // is still what the seventh is spelled with.
        let options = et_options(41, "2.3.5.7.11");
        assert_eq!(ranks(&options), vec![2, 3, 4]);
        assert_eq!(accidental_ratios(&options[1]), vec![(81, 80)]);
        assert_eq!(note_of(&options[1], 7, 4), "vBb5");
        assert_eq!(note_of(&options[1], 11, 8), "^^F5");
        assert_eq!(accidental_ratios(&options[2]), vec![(81, 80), (33, 32)]);
        assert_eq!(note_of(&options[2], 11, 8), ">F5");

        // 72et needs its step, since its fifth chain closes early, and has two
        // further accidentals on top: 64/63 is worth two steps and 33/32 three.
        let options = et_options(72, "2.3.5.7.11");
        assert_eq!(ranks(&options), vec![3, 4, 5]);
        assert_eq!(accidental_ratios(&options[0]), vec![(81, 80)]);
        assert_eq!(note_of(&options[0], 7, 4), "vvBb5");
        assert_eq!(note_of(&options[0], 11, 8), "^^^F5");
        assert_eq!(
            accidental_ratios(&options[2]),
            vec![(81, 80), (64, 63), (33, 32)]
        );
    }

    #[test]
    fn an_accidental_worth_what_another_is_worth_is_not_offered() {
        // 41et spells 5/4 and 7/4 with the same accidental, since it tempers out
        // the difference between 81/80 and 64/63. That is a property of 41et, not
        // an excuse for a second symbol, so there is no notation with both.
        for options in [et_options(41, "2.3.5.7"), et_options(41, "2.3.5.7.11")] {
            for n in &options {
                assert!(!accidental_ratios(n).contains(&(64, 63)));
            }
        }

        // Nor at higher rank: pele tempers out 5120/5103, so its accidentals for
        // 5 and 7 are one interval and it has a single notation.
        let options = options_of("2.3.5.7", &[(5120, 5103)]);
        assert_eq!(ranks(&options), vec![3]);
        assert_eq!(accidental_ratios(&options[0]), vec![(81, 80)]);
        assert_eq!(note_of(&options[0], 7, 4), "vBb5");

        // 31et is the same story one prime up: 33/32 ~ 64/63 is.
        let options = et_options(31, "2.3.5.7.11");
        assert_eq!(ranks(&options), vec![2, 3]);
        assert_eq!(accidental_ratios(&options[1]), vec![(64, 63)]);
        assert_eq!(note_of(&options[1], 11, 8), "^F5");
    }

    #[test]
    fn equal_temperament_with_nothing_to_offer() {
        // A meantone equal temperament tempers out the accidental for 5, so
        // nothing is left to offer. Its fifth chain reaches every note, so the
        // fifth chain alone is all it gets - and all it needs, the enharmonics
        // of 12et being the point of it.
        for divisions in [5, 7, 12, 19] {
            let options = et_options(divisions, "2.3.5");
            assert_eq!(ranks(&options), vec![2]);
            assert_eq!(note_of(&options[0], 5, 4), "E5");
        }

        // 15et and 34et need their accidental and have nothing beyond it.
        for divisions in [15, 34] {
            let options = et_options(divisions, "2.3.5");
            assert_eq!(ranks(&options), vec![3]);
            assert_eq!(accidental_ratios(&options[0]), vec![(81, 80)]);
            assert_eq!(note_of(&options[0], 5, 4), "vE5");
        }
    }

    #[test]
    fn equal_temperament_whose_accidental_is_worth_more_than_a_step() {
        // The fifth chain of 25et closes after five notes, so it needs an
        // accidental, and its syntonic comma is worth two steps rather than one.
        // An accidental is one step or it is nothing - that is the rule ups and
        // downs is built on - so these have no notation.
        for divisions in [25, 51, 54] {
            let subgroup: Subgroup = "2.3.5".parse().unwrap();
            let temperament = Temperament::equal(divisions, &subgroup).unwrap();
            assert!(Notation::options(&temperament).is_err());
        }

        // The answer is a subgroup whose accidental does fit. 24et over 2.3.5 is
        // not primitive and saturates to 12et, but over 2.3.5.11 its quartertone is
        // 33/32 and worth exactly one step.
        let options = et_options(24, "2.3.5.11");
        assert_eq!(ranks(&options), vec![3]);
        assert_eq!(accidental_ratios(&options[0]), vec![(33, 32)]);
        assert_eq!(note_of(&options[0], 5, 4), "E5");
        assert_eq!(note_of(&options[0], 11, 8), "^F5");
    }

    #[test]
    fn equal_temperament_gets_single_step_accidental() {
        // A constructed example that maps 81/80 to two steps and 64/63 to one, and does not have a single circle of fifths.
        let subgroup: Subgroup = "2.3.5.7".parse().unwrap();
        let temperament =
            Temperament::from_mapping(&vec![vec![25, 40, 60, 69]], &subgroup).unwrap();
        let options = Notation::options(&temperament).unwrap();
        assert_eq!(ranks(&options), vec![3]);
        assert_eq!(accidental_ratios(&options[0]), vec![(64, 63)]);
    }

    #[test]
    fn no_rank_two_notation_when_the_fifth_chain_does_not_reach() {
        // Blackwood's fifth chain closes after five notes, and porcupine
        // reaches 5 only through its own comma. Neither has a rank 2 notation,
        // so neither has anything smaller than the one it started with.
        for comma in [(256, 243), (250, 243)] {
            let options = options_of("2.3.5", &[comma]);
            assert_eq!(ranks(&options), vec![3]);
            assert_eq!(note_of(&options[0], 5, 4), "vE5");
        }
    }

    #[test]
    fn rank_two_temperaments_offer_the_whole_run() {
        // Huygens keeps the meantone spelling of 5, so its accidental for 5 is
        // tempered out and only 7 and 11 are left to offer. All three notations
        // spell 5 and 7 the same way; the 11 is what each of them adds.
        let options = options_of("2.3.5.7.11", &[(81, 80), (126, 125), (99, 98)]);
        assert_eq!(ranks(&options), vec![2, 3, 4]);
        assert_eq!(accidental_ratios(&options[1]), vec![(64, 63)]);
        assert_eq!(accidental_ratios(&options[2]), vec![(64, 63), (33, 32)]);
        for n in &options {
            assert_eq!(note_of(n, 5, 4), "E5");
        }
        assert_eq!(note_of(&options[0], 7, 4), "A#5");
        assert_eq!(note_of(&options[1], 7, 4), "vBb5");
        assert_eq!(note_of(&options[2], 11, 8), ">F5");
    }

    #[test]
    fn a_dropped_accidental_becomes_a_stack_of_the_lower_ones() {
        // 11-limit marvel keeps an accidental for 5 and may drop the other two.
        // 64/63 needs the fifth chain, but 33/32 is worth exactly what the two
        // below it are worth together, so it becomes one of each.
        let options = options_of("2.3.5.7.11", &[(225, 224), (385, 384)]);
        assert_eq!(ranks(&options), vec![3, 4, 5]);
        assert_eq!(note_of(&options[1], 11, 8), "^>F5");
        // Dropping 64/63 too substitutes its own spelling into that one.
        assert_eq!(note_of(&options[0], 11, 8), "^^^Gbb5");
    }

    #[test]
    fn a_stack_is_the_shortest_one_that_works() {
        // 72et is worth one step for 81/80, two for 64/63 and three for 33/32,
        // so a notation keeping the first two may write the third either as
        // three of the first or as one of each. Two marks beat three.
        let options = et_options(72, "2.3.5.7.11");
        assert_eq!(ranks(&options), vec![3, 4, 5]);
        assert_eq!(note_of(&options[1], 11, 8), "^>F5");
        // With only 81/80 kept there is nothing to choose and it is three.
        assert_eq!(note_of(&options[0], 11, 8), "^^^F5");
    }

    #[test]
    fn spelling_the_spelled_settles() {
        // The best spelling of a tempered interval is already `spell`'s fixed point:
        // reading it back as a just interval and spelling again should not
        // move it - the same property `Simplifier::simplify` settles into on
        // its own answer.
        for (divisions, subgroup) in [(41, "2.3.5.7.11"), (31, "2.3.5.7"), (22, "2.3.5")] {
            let notation = et_options(divisions, subgroup).pop().unwrap();
            for ups in -20..=60 {
                let mut spelling = vec![0; notation.len()];
                spelling[2] = ups;
                let interval = notation.to_interval(&spelling).unwrap();

                let spelled = notation.spell_interval(&interval).unwrap();
                let round_trip = notation
                    .spell_interval(&notation.to_interval(&spelled).unwrap())
                    .unwrap();
                assert_eq!(
                    round_trip, spelled,
                    "{divisions}et over {subgroup}, {ups} ups"
                );
            }
        }
    }

    #[test]
    fn the_recommendation_is_the_smallest_that_keeps_the_nominals() {
        // 41et has three notations. The smallest spells every prime off its
        // nominal, and the largest only turns the two marks of the middle one
        // into one of another kind, so the middle one is what is wanted.
        let subgroup: Subgroup = "2.3.5.7.11".parse().unwrap();
        let temperament = Temperament::equal(41, &subgroup).unwrap();
        let options = Notation::options(&temperament).unwrap();
        assert_eq!(ranks(&options), vec![2, 3, 4]);
        assert_eq!(
            Notation::from_temperament(&temperament)
                .unwrap()
                .generators(),
            options[1].generators()
        );

        // Where in the run it lands is not fixed: septimal meantone, marvel and
        // schismatic want the second of two, huygens the third of three.
        for (subgroup, commas, wanted) in [
            ("2.3.5.7", &[(81, 80), (225, 224)][..], 1),
            // Huygens wants the second of three: its rank 3 notation already
            // spells every prime on its own nominal, so the fourth accidental
            // is a symbol bought for nothing.
            ("2.3.5.7.11", &[(81, 80), (126, 125), (99, 98)][..], 1),
            ("2.3.5.7", &[(225, 224)][..], 1),
            ("2.3.5", &[(32805, 32768)][..], 1),
        ] {
            assert_eq!(
                tempered(subgroup, commas).generators(),
                options_of(subgroup, commas)[wanted].generators()
            );
        }
    }

    #[test]
    fn flattone_single_notation() {
        // Flattone writes 11/8 as F#, where just intonation has tF.
        // That means # = t, so that notation is skipped.
        let flattone = &[(45, 44), (81, 80)][..];
        let options = options_of("2.3.5.11", flattone);
        assert_eq!(ranks(&options), vec![2]);
        assert_eq!(note_of(&options[0], 11, 8), "F#5");
    }

    #[test]
    fn a_prime_off_its_nominal_is_refused() {
        // The spellings the rule is there to reject, each the smallest notation
        // of its temperament: 5 on F rather than E, 7 on A rather than B, and 11
        // on G rather than F.
        for (subgroup, commas) in [
            ("2.3.5", &[(32805, 32768)][..]),
            ("2.3.5.7", &[(81, 80), (225, 224)][..]),
            ("2.3.5.7.11", &[(81, 80), (126, 125), (99, 98)][..]),
        ] {
            assert!(!options_of(subgroup, commas)[0].keeps_nominals());
        }
        // Just intonation keeps every nominal, by definition.
        for subgroup in ["2.3", "2.3.5", "2.3.5.7.11", "2.3.5.7.11.13"] {
            assert!(notation(subgroup).keeps_nominals());
        }
    }

    #[test]
    fn a_temperament_accepts_each_requested_11_limit_prefix() {
        let subgroup: Subgroup = "2.3.5.7.11".parse().unwrap();
        let temperament = Temperament::equal(41, &subgroup).unwrap();
        let accidentals = derive_accidentals(&subgroup);

        let notations: Vec<Notation> = (1..=accidentals.len())
            .map(|count| Notation::build(&temperament, &accidentals[..count]).unwrap())
            .collect();

        assert_eq!(
            notations.iter().map(accidental_ratios).collect::<Vec<_>>(),
            vec![
                vec![(81, 80)],
                vec![(81, 80), (64, 63)],
                vec![(81, 80), (64, 63), (33, 32)],
            ]
        );
        assert_eq!(note_of(&notations[0], 11, 8), "^^F5");
        assert_eq!(note_of(&notations[2], 11, 8), "tF5");
    }

    #[test]
    fn an_accidental_list_must_reach_every_pitch() {
        let subgroup: Subgroup = "2.3.5".parse().unwrap();
        let temperament = Temperament::equal(25, &subgroup).unwrap();
        assert!(Notation::build(&temperament, &[]).is_err());

        let syntonic = derive_accidentals(&subgroup)[0].clone();
        let notation = Notation::build(&temperament, &[syntonic]).unwrap();
        assert_eq!(notation.len(), 3);
        assert_eq!(note_of(&notation, 5, 4), "vE5");
    }

    /// Every octave-free interval with exponents in -2..=2 beyond the octave.
    fn small_intervals(dim: usize) -> Vec<Vec<i64>> {
        (0..5i64.pow(dim as u32 - 1))
            .map(|mut code| {
                let mut interval = vec![0];
                for _ in 1..dim {
                    interval.push(code % 5 - 2);
                    code /= 5;
                }
                interval
            })
            .collect()
    }

    #[test]
    fn register_does_not_change_the_spelling() {
        // Octaves cost nothing to write, so moving an interval by octaves moves
        // only its octave coordinate. The seed once charged for them and got
        // this wrong far from C5.
        for n in et_options(22, "2.3.5.7")
            .into_iter()
            .chain(options_of("2.3.5.7.11", &[(441, 440), (896, 891)]))
        {
            for interval in small_intervals(n.dim()) {
                let spelling = n.spell_interval(&interval).unwrap();
                for octaves in [-12, -3, 4, 9] {
                    let mut moved = interval.clone();
                    moved[0] += octaves;
                    let mut expected = spelling.clone();
                    expected[0] += octaves;
                    assert_eq!(n.spell_interval(&moved).unwrap(), expected);
                }
            }
        }
    }

    #[test]
    fn nothing_near_the_best_spellings_is_cheaper() {
        // Checked independently of the enumeration: every spelling within a
        // box of enharmonics around the answer costs no less than the third
        // best, and the three answers are the three cheapest costs in the box.
        // Pele's rank 4 notation is where a walk one enharmonic wide once
        // missed `E##` for `vdF##`; 41et's largest has three enharmonics.
        let pele = options_of("2.3.5.7.11", &[(441, 440), (896, 891)]);
        let et = et_options(41, "2.3.5.7.11");
        for (n, width) in [(&pele[1], 8i64), (&et[2], 4)] {
            let lattice = n.enharmonics();
            let side = 2 * width + 1;
            for interval in small_intervals(n.dim()) {
                let tempered = n.temperament().temper(&interval).unwrap();
                let found = n.spellings(&tempered, 3).unwrap();
                let mut costs: Vec<i64> = (0..side.pow(lattice.len() as u32))
                    .map(|mut code| {
                        let mut candidate = found[0].clone();
                        for row in lattice {
                            let times = code % side - width;
                            code /= side;
                            for (c, &r) in candidate.iter_mut().zip(row) {
                                *c += times * r;
                            }
                        }
                        assert_eq!(n.temper(&candidate).unwrap(), tempered);
                        spelling_cost(&candidate)
                    })
                    .collect();
                costs.sort_unstable();
                let found_costs: Vec<i64> = found.iter().map(|s| spelling_cost(s)).collect();
                assert_eq!(found_costs, costs[..3], "{interval:?}");
            }
        }
    }

    #[test]
    fn just_nominals_are_the_diatonic_degrees() {
        // (7, 11, 16, 20, 24, 26): third, seventh, fourth and sixth, as just
        // intonation writes them, one mark each on vE, <Bb, tF and *Ab.
        let subgroup: Subgroup = "2.3.5.7.11.13".parse().unwrap();
        let nominals = just_nominals(&subgroup);
        let degrees: Vec<i64> = nominals.iter().map(|n| n.degree).collect();
        let costs: Vec<i64> = nominals.iter().map(|n| n.cost).collect();
        assert_eq!(degrees, vec![16, 20, 24, 26]);
        assert_eq!(costs, vec![11, 15, 13, 19]);
        // Every derived accidental has degree zero under it.
        let map: Vec<i64> = [7, 11].into_iter().chain(degrees).collect();
        for a in derive_accidentals(&subgroup) {
            let dot: i64 = a.iter().zip(&map).map(|(x, d)| x * d).sum();
            assert_eq!(dot, 0);
        }
    }

    #[test]
    fn nominal_costs_look_past_the_cheapest_spelling() {
        // Schismatic's fifth chain puts 5 on F, and nowhere else.
        let options = options_of("2.3.5", &[(32805, 32768)]);
        assert_eq!(options[0].nominal_costs(), vec![None]);
        assert_eq!(options[1].nominal_costs(), vec![Some(11)]);

        // Flattone writes 11 as F#, which is on F.
        let options = options_of("2.3.5.11", &[(45, 44), (81, 80)]);
        assert_eq!(options[0].nominal_costs(), vec![Some(4), Some(8)]);

        // 41et's largest notation writes 7/4 as tA, but vBb is there too, at
        // what just intonation spends on it, so the nominals are kept.
        let options = et_options(41, "2.3.5.7.11");
        assert_eq!(note_of(&options[2], 7, 4), ">A5");
        assert_eq!(options[2].nominal_costs()[1], Some(15));
        assert!(options[2].keeps_nominals());
    }

    #[test]
    fn a_cheaper_spelling_elsewhere_does_not_hide_the_nominal() {
        // Semaphore and pele each have a spelling cheaper than the one on the
        // nominal, which the old rule, reading only `spell`, took as a miss.
        let semaphore = options_of("2.3.7", &[(49, 48)]);
        assert!(semaphore[0].keeps_nominals());

        let pele = tempered("2.3.5.7.11", &[(441, 440), (896, 891)]);
        assert_eq!(accidental_ratios(&pele), vec![(81, 80), (33, 32)]);
        assert!(pele.keeps_nominals());
    }

    #[test]
    fn a_tie_goes_to_the_lower_primes() {
        // Miracle's two accidental notations tie three ways: whichever pair is
        // kept, one of 5, 7 and 11 needs two marks. The simpler 5 and then 7
        // are preferred, so it is 11 that takes them.
        let miracle = &[(225, 224), (1029, 1024), (385, 384)][..];
        let options = options_of("2.3.5.7.11", miracle);
        assert_eq!(accidental_ratios(&options[1]), vec![(81, 80), (64, 63)]);
        assert_eq!(note_of(&options[1], 11, 8), "^>F5");
        assert_eq!(
            tempered("2.3.5.7.11", miracle).generators(),
            options[1].generators()
        );
    }

    #[test]
    fn a_notation_of_a_given_size() {
        let subgroup: Subgroup = "2.3.5.7.11".parse().unwrap();
        let commas =
            [(225, 224), (1029, 1024), (385, 384)].map(|(n, d)| subgroup.factorize(n, d).unwrap());
        let miracle = Temperament::from_commas(&commas, &subgroup).unwrap();

        // The fifth chain alone does not reach every tempered interval of miracle.
        assert!(Notation::with_count(&miracle, 0).is_err());
        let one = Notation::with_count(&miracle, 1).unwrap();
        assert_eq!(accidental_ratios(&one), vec![(81, 80)]);
        let three = Notation::with_count(&miracle, 3).unwrap();
        assert_eq!(three.len(), 5);
        assert!(Notation::with_count(&miracle, 4).is_err());

        // Only the images matter: in 41et 49/48 is a step as good as 81/80.
        let subgroup: Subgroup = "2.3.5.7".parse().unwrap();
        let t = Temperament::equal(41, &subgroup).unwrap();
        let septimal = subgroup.factorize(49, 48).unwrap();

        let n = Notation::with_count_and_accidentals(&t, &[septimal], 1).unwrap();
        let syntonic =
            Notation::with_count_and_accidentals(&t, &derive_accidentals(&subgroup)[..1], 1)
                .unwrap();
        assert_eq!(n.enharmonics(), syntonic.enharmonics());
        assert_eq!(n.nominal_costs(), syntonic.nominal_costs());
    }

    #[test]
    fn the_circle_of_fifths_never_closes() {
        // An equal temperament must temper out the pythagorean comma, and no
        // notation can: a notation is free on its octave, its fifth and its
        // accidentals. That difference is the enharmonic lattice, and in 12et
        // it is exactly what leaves C# and Db to differ.
        let subgroup = Subgroup::p_limit(5);
        let t = Temperament::equal(12, &subgroup).unwrap();
        let n = Notation::build(&t, &[]).unwrap();
        assert_eq!(n.enharmonics().len(), 1);
        // Twelve fifths less seven octaves, which is the pythagorean comma.
        assert_eq!(n.enharmonics()[0], vec![-7, 12]);
        let comma = n.to_interval(&n.enharmonics()[0]).unwrap();
        assert_eq!(subgroup.to_ratio(&comma).unwrap(), (531441, 524288));
    }

    #[test]
    fn enharmonics_are_the_freedom_over_the_temperament() {
        // 22et: the fifth chain alone leaves one enharmonic, and the notation
        // with an accidental leaves two - its octave being twenty two steps and
        // its fifth thirteen, which is the whole of ups and downs in 22et.
        let subgroup: Subgroup = "2.3.5.7".parse().unwrap();
        let t = Temperament::equal(22, &subgroup).unwrap();
        let bare = Notation::build(&t, &[]).unwrap();

        let syntonic = derive_accidentals(&subgroup)[0].clone();
        let raised = Notation::build(&t, &[syntonic]).unwrap();

        assert_eq!(bare.enharmonics().len(), 1);
        assert_eq!(raised.enharmonics().len(), 2);
        for e in raised.enharmonics() {
            assert_eq!(raised.temper(e).unwrap(), vec![0]);
        }

        // Every requested notation has one enharmonic per extra coordinate over
        // the temperament, and each is a non-unison it tempers out.
        for (divisions, subgroup) in [(12, "2.3.5"), (22, "2.3.5.7"), (41, "2.3.5.7.11")] {
            let subgroup: Subgroup = subgroup.parse().unwrap();
            let t = Temperament::equal(divisions, &subgroup).unwrap();
            let accidentals = derive_accidentals(&subgroup);
            for count in 0..=accidentals.len() {
                let Ok(n) = Notation::build(&t, &accidentals[..count]) else {
                    continue;
                };
                assert_eq!(n.enharmonics().len(), n.len() - t.rank());
                for e in n.enharmonics() {
                    // Tempers to nothing, but is not the unison on the page.
                    assert_eq!(n.temper(e).unwrap(), vec![0; t.rank()]);
                    assert!(e.iter().any(|&x| x != 0));
                }
            }
        }
    }

    #[test]
    fn a_bijection_has_no_enharmonics() {
        // Where the notation has the rank of the temperament its kernel is the
        // temperament's, so there is nothing left over to choose.
        let subgroup: Subgroup = "2.3.5.7".parse().unwrap();
        let commas = [vec![-4, 4, -1, 0], vec![1, 2, -3, 1]];
        let t = Temperament::from_commas(&commas, &subgroup).unwrap();
        let n = Notation::build(&t, &[]).unwrap();
        assert_eq!(n.len(), t.rank());
        assert!(n.enharmonics().is_empty());
    }

    #[test]
    fn spelling_cost_agrees_l2() {
        let rank = 5;
        let weights = spelling_cost_l2(5);

        // Octaves are free only for the l1 cost
        assert_eq!(spelling_cost(&[5, 2, 0, 0, 0]), 0);

        for i in 1..rank {
            // Relative to D5
            let mut interval = vec![-1, 2, 0, 0, 0];
            interval[i] += 1;

            let w_l1 = spelling_cost(&interval);
            let w_l2 = weights[i][i];

            assert_eq!((w_l1 * w_l1) as f64, w_l2);
        }
    }

    #[test]
    fn custom_accidental_25edo() {
        let subgroup: Subgroup = "2.3.5".parse().unwrap();
        let temperament = Temperament::equal(25, &subgroup).unwrap();

        // Fails on defaults because 81/80 does not reach the whole temperament
        // or is not worth a step (25edo over 2.3.5 is known to have no default notation).
        assert!(Notation::options(&temperament).is_err());

        // But we can supply 25/24 as a custom accidental.
        let custom_acc = subgroup.factorize(25, 24).unwrap();

        let options = Notation::options_with_accidentals(&temperament, &[custom_acc]).unwrap();
        assert_eq!(ranks(&options), vec![3]);
        assert_eq!(note_of(&options[0], 5, 4), "vvE5");
    }

    #[test]
    fn custom_accidental_41edo() {
        let subgroup: Subgroup = "2.3.5.7".parse().unwrap();
        let temperament = Temperament::equal(41, &subgroup).unwrap();

        // We can notate using a single accidental for 49/48 or 50/49.
        let acc_49_48 = subgroup.factorize(49, 48).unwrap();
        let acc_50_49 = subgroup.factorize(50, 49).unwrap();

        let options_49_48 =
            Notation::options_with_accidentals(&temperament, std::slice::from_ref(&acc_49_48))
                .unwrap();
        assert_eq!(options_49_48.last().unwrap().len(), 3);

        let options_50_49 =
            Notation::options_with_accidentals(&temperament, std::slice::from_ref(&acc_50_49))
                .unwrap();
        assert_eq!(options_50_49.last().unwrap().len(), 3);
    }
}
