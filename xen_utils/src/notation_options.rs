use diophantine::Matrix;

use crate::Error;
use crate::notation::{Notation, fifth_chain, spans};
use crate::temperament::Temperament;
use crate::util::{is_zero, select};

pub(crate) struct NotationOptions<'a> {
    temperament: &'a Temperament,
    accidentals: Matrix<i64>,
    /// What each accidental maps to in the temperament.
    images: Matrix<i64>,
}

impl<'a> NotationOptions<'a> {
    /// Keeps the accidentals that can be of any use: not tempered out, and not
    /// worth what an earlier one is worth, up to direction.
    pub(crate) fn new(
        temperament: &'a Temperament,
        accidentals: &[Vec<i64>],
    ) -> Result<Self, Error> {
        let vectors: Matrix<i64> = accidentals.to_vec();
        let images = temperament.temper_all(&vectors)?;
        let mut useful: Vec<usize> = Vec::new();

        // sharp
        let mut apotome = vec![0; temperament.dim()];
        apotome[0] = -11;
        apotome[1] = 7;
        apotome = temperament.temper(&apotome)?;

        for index in 0..images.len() {
            // skip if tempered
            if is_zero(&images[index]) {
                continue;
            }
            // skip if it is the same as one we already have
            if useful
                .iter()
                .any(|&kept| equal_up_to_sign(&images[kept], &images[index]))
            {
                continue;
            }
            // skip if it maps to a sharp or flat
            if equal_up_to_sign(&apotome, &images[index]) {
                continue;
            }
            useful.push(index);
        }

        Ok(NotationOptions {
            temperament,
            accidentals: useful.iter().map(|&i| accidentals[i].clone()).collect(),
            images: select(&images, &useful),
        })
    }

    /// The best notation of each size, smallest first.
    pub(crate) fn search(&self) -> Result<Vec<Notation>, Error> {
        let mut found = Vec::new();
        for size in 0..=self.accidentals.len() {
            if let Some(notation) = self.best_of_size(size)? {
                found.push(notation);
            }
        }
        if found.is_empty() {
            return Err(self.nothing_spans());
        }
        Ok(found)
    }

    /// The best notation keeping exactly `size` of the accidentals.
    pub(crate) fn with_count(&self, size: usize) -> Result<Notation, Error> {
        self.best_of_size(size)?.ok_or_else(|| {
            Error::Unsupported(format!(
                "no notation of this temperament keeps exactly {size} of these accidentals"
            ))
        })
    }

    /// The best notation keeping exactly `size` of the accidentals, if any.
    ///
    /// Ranked by how many primes are off their nominals, then by the total cost
    /// of writing the primes on their nominals, then by those costs one prime at
    /// a time, lowest prime first. Where all of that ties, the subset that comes
    /// first in the order the accidentals were given wins.
    fn best_of_size(&self, size: usize) -> Result<Option<Notation>, Error> {
        let mut best: Option<(Score, Notation)> = None;
        for subset in subsets(&(0..self.accidentals.len()).collect::<Vec<_>>(), size) {
            if !self.valid(&subset)? {
                continue;
            }
            let kept: Vec<Vec<i64>> = subset
                .iter()
                .map(|&i| self.accidentals[i].clone())
                .collect();
            let notation = Notation::build(self.temperament, &kept)?;
            let score = Score::of(&notation);
            if best.as_ref().is_none_or(|(held, _)| score < *held) {
                best = Some((score, notation));
            }
        }
        Ok(best.map(|(_, notation)| notation))
    }

    /// Whether the accidentals at `keep` make a notation at all.
    ///
    /// They must reach every tempered interval together with the octave and the
    /// fifth. If an equal temperment keeps any accidental, one of them must
    /// map to a single step.
    fn valid(&self, keep: &[usize]) -> Result<bool, Error> {
        if self.temperament.rank() == 1
            && !keep.is_empty()
            && !keep.iter().any(|&i| self.images[i][0].abs() == 1)
        {
            return Ok(false);
        }
        let mut generators = fifth_chain(self.temperament.dim());
        generators.extend(keep.iter().map(|&i| self.accidentals[i].clone()));
        let images = self.temperament.temper_all(&generators)?;
        Ok(spans(&images, self.temperament.rank()))
    }

    /// Why no subset of the accidentals reaches every tempered interval.
    fn nothing_spans(&self) -> Error {
        let subgroup = self.temperament.subgroup();
        if self.temperament.rank() == 1 {
            return Error::Unsupported(format!(
                "the fifth chain of this equal temperament does not reach every note, and no accidental of {subgroup} maps to a single step of it"
            ));
        }
        Error::Unsupported(format!(
            "the octave, the fifth and the accidentals of {subgroup} do not reach every tempered interval of this rank {} temperament",
            self.temperament.rank()
        ))
    }
}

/// How a notation ranks against others of its size, lower being better.
/// Compared field by field, in order.
#[derive(PartialEq, Eq)]
struct Score {
    /// How many primes are off their nominals.
    failures: usize,
    /// The total cost of writing every prime on its nominal, or `None` where
    /// some prime cannot be, which is worse than any cost.
    total: Option<i64>,
    /// The cost of each prime on its nominal, where one that cannot be is last.
    costs: Vec<i64>,
}

impl Score {
    fn of(notation: &Notation) -> Self {
        let verdict = notation.nominal_verdict();
        Score {
            failures: verdict.failures,
            total: verdict.costs.iter().copied().sum::<Option<i64>>(),
            costs: verdict
                .costs
                .iter()
                .map(|c| c.unwrap_or(i64::MAX))
                .collect(),
        }
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // `None` sorts first for `Option`, and here it has to sort last.
        let total = |s: &Score| s.total.unwrap_or(i64::MAX);
        self.failures
            .cmp(&other.failures)
            .then(total(self).cmp(&total(other)))
            .then_with(|| self.costs.cmp(&other.costs))
    }
}

fn equal_up_to_sign(one: &[i64], other: &[i64]) -> bool {
    assert_eq!(one.len(), other.len());
    one == other || one.iter().zip(other).all(|(a, b)| *a == -b)
}

/// Subsets of `items` of size `size`, in lexicographic order.
fn subsets(items: &[usize], size: usize) -> Vec<Vec<usize>> {
    if size == 0 {
        return vec![Vec::new()];
    }
    let mut result = Vec::new();
    for (position, &item) in items.iter().enumerate() {
        for mut rest in subsets(&items[position + 1..], size - 1) {
            let mut subset = vec![item];
            subset.append(&mut rest);
            result.push(subset);
        }
    }
    result
}
