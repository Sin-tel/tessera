//! Simplifying just intervals.

use diophantine::{Matrix, cvp_l1_top_k};

use crate::Error;
use crate::Temperament;
use crate::util::{MAX_SEARCH_NODES, subtract};

/// Finds the simplest just intervals that temper to a given tempered interval.
///
/// "Simplest" here means the Wilson norm [`sopfr`], with ties broken by the
/// quadratic norm under `diag(p²)` and then by the interval itself. The search
/// is exact: `sopfr` is a weighted L1 norm, and the lattice enumeration prunes
/// on the quadratic norm, which never exceeds it.
#[derive(Debug, Clone)]
pub struct Simplifier {
    temperament: Temperament,
    lattice: Matrix<i64>,
    /// The primes of the subgroup: the weights `sopfr` gives each exponent.
    primes: Vec<i64>,
}

impl Simplifier {
    /// Builds the simplifier over the temperament.
    pub fn new(temperament: &Temperament) -> Self {
        let lattice = temperament.reduced_comma_basis();
        let primes = temperament
            .subgroup()
            .basis()
            .iter()
            .map(|&p| i64::from(p))
            .collect();
        Simplifier {
            temperament: temperament.clone(),
            lattice,
            primes,
        }
    }

    /// The reduced basis of the comma lattice being searched.
    pub fn lattice(&self) -> &Matrix<i64> {
        &self.lattice
    }

    /// The simplest just interval that tempers to `tempered`.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `tempered` does not have one
    /// entry per generator of the temperament.
    pub fn simplify(&self, tempered: &[i64]) -> Result<Vec<i64>, Error> {
        Ok(self.simplifications(tempered, 1)?.swap_remove(0))
    }

    /// [`simplify`](Self::simplify) the tempered interval a just interval maps
    /// to: the simplest just interval the temperament makes equal to it.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `interval` does not have one
    /// entry per basis element of the subgroup.
    pub fn simplify_interval(&self, interval: &[i64]) -> Result<Vec<i64>, Error> {
        self.simplify(&self.temperament.temper(interval)?)
    }

    /// The `count` simplest just intervals that temper to `tempered`, simplest
    /// first.
    ///
    /// The cost grows with `count`, and the search only closes once it holds
    /// `count` intervals, so ask for the handful wanted rather than all of them.
    /// Just intonation has only the one.
    ///
    /// Exact within [`MAX_SEARCH_NODES`]. A tempered
    /// interval absurdly far from a unison exhausts that budget and is answered
    /// with the best intervals found, which may be fewer than `count`.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `tempered` does not have one
    /// entry per generator of the temperament.
    pub fn simplifications(&self, tempered: &[i64], count: usize) -> Result<Matrix<i64>, Error> {
        Ok(self.simplifications_within_budget(tempered, count)?.0)
    }

    /// [`simplifications`](Self::simplifications), and whether the search closed
    /// rather than running out of [`MAX_SEARCH_NODES`]. Only a test reads the flag.
    pub(crate) fn simplifications_within_budget(
        &self,
        tempered: &[i64],
        count: usize,
    ) -> Result<(Matrix<i64>, bool), Error> {
        let interval = self.temperament.preimage(tempered)?;
        if self.lattice.is_empty() {
            let simplifications = if count == 0 { vec![] } else { vec![interval] };
            return Ok((simplifications, true));
        }
        let (commas, complete) = cvp_l1_top_k(
            &interval,
            &self.lattice,
            &self.primes,
            count,
            Some(MAX_SEARCH_NODES),
        )?;
        let simplifications = commas
            .iter()
            .map(|comma| subtract(&interval, comma))
            .collect();
        Ok((simplifications, complete))
    }

    /// [`simplifications`](Self::simplifications) of the tempered interval a
    /// just interval maps to.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `interval` does not have one
    /// entry per basis element of the subgroup.
    pub fn simplifications_interval(
        &self,
        interval: &[i64],
        count: usize,
    ) -> Result<Matrix<i64>, Error> {
        self.simplifications(&self.temperament.temper(interval)?, count)
    }
}

/// The Wilson norm of an interval: the sum of the prime factors of its
/// numerator and denominator, with repetition. `81/80` is `3*4 + 2*4 + 5 = 25`.
pub fn sopfr(interval: &[i64], primes: &[u32]) -> i64 {
    interval
        .iter()
        .zip(primes)
        .map(|(&exponent, &prime)| i64::from(prime) * exponent.abs())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Notation;
    use crate::primes::Subgroup;
    use crate::temperament::Temperament;
    use diophantine::cvp_exact;

    /// A notation of the equal temperament of `divisions` over `subgroup`, and
    /// its simplifier.
    ///
    /// The largest of the run rather than the recommended one, since these tests
    /// stack a notation's first accidental and the recommendation is free to
    /// keep none. Simplifying does not depend on the notation either way.
    fn simplifier(divisions: i64, subgroup: &str) -> (Notation, Simplifier) {
        let subgroup: Subgroup = subgroup.parse().unwrap();
        let temperament = Temperament::equal(divisions, &subgroup).unwrap();
        let options = Notation::options(&temperament).unwrap();
        let notation = options.last().unwrap().clone();
        let simplifier = Simplifier::new(&temperament);
        (notation, simplifier)
    }

    /// The just interval of `ups` of the first accidental stacked on `C`.
    fn stack(notation: &Notation, ups: i64) -> Vec<i64> {
        let mut spelling = vec![0; notation.len()];
        spelling[2] = ups;
        notation.to_interval(&spelling).unwrap()
    }

    /// What `interval` simplifies to, as a ratio.
    fn simplified(notation: &Notation, simplifier: &Simplifier, interval: &[i64]) -> (u64, u64) {
        let simplified = simplifier.simplify_interval(interval).unwrap();
        notation.subgroup().to_ratio(&simplified).unwrap()
    }

    #[test]
    fn a_target_too_far_to_search_still_answers() {
        // Past the budget the answer is the best found rather than the simplest
        // there is, but it is still a just interval that tempers to what was
        // asked for, and it still comes back quickly.
        let subgroup: Subgroup = "2.3.5.7.11".parse().unwrap();
        let t = Temperament::equal(41, &subgroup).unwrap();
        let s = Simplifier::new(&t);
        for steps in [1_000_000i64, 100_000_000] {
            let simplifications = s.simplifications(&[steps], 3).unwrap();
            assert!(!simplifications.is_empty());
            for interval in &simplifications {
                assert_eq!(t.temper(interval).unwrap(), vec![steps]);
            }
        }
    }

    #[test]
    fn the_wilson_norm_is_the_sum_of_prime_factors() {
        let primes = [2, 3, 5, 7, 11];
        // 81/80 is 3+3+3+3 over 2+2+2+2+5, and 45/44 is 3+3+5 over 2+2+11.
        assert_eq!(sopfr(&[-4, 4, -1, 0, 0], &primes), 25);
        assert_eq!(sopfr(&[-2, 2, 1, 0, -1], &primes), 26);
        assert_eq!(sopfr(&[1, 0, 0, 0, 0], &primes), 2);
        assert_eq!(sopfr(&[0; 5], &primes), 0);
    }

    #[test]
    fn a_stack_of_ups_simplifies_to_the_octave() {
        // The accidental of 41et is worth one step, so 41 of them are an octave,
        // though as a just interval they are 81/80 forty one times over.
        let (notation, simplifier) = simplifier(41, "2.3.5.7.11");
        let stacked = stack(&notation, 41);
        assert_eq!(stacked, vec![-164, 164, -41, 0, 0]);

        let simplified = simplifier.simplify_interval(&stacked).unwrap();
        assert_eq!(simplified, vec![1, 0, 0, 0, 0]);
        assert_eq!(
            notation.note(&notation.spell_interval(&simplified).unwrap()),
            "C6"
        );
    }

    #[test]
    fn forty_one_ups_are_the_interval_table_of_41et() {
        // The stack reaches every step of 41et, and what it simplifies to is
        // what those steps are usually called.
        let (notation, simplifier) = simplifier(41, "2.3.5.7.11");
        for (ups, ratio) in [
            (0, (1, 1)),
            (8, (8, 7)),
            (9, (7, 6)),
            (11, (6, 5)),
            (13, (5, 4)),
            (17, (4, 3)),
            (19, (11, 8)),
            (24, (3, 2)),
            (28, (8, 5)),
            (30, (5, 3)),
            (33, (7, 4)),
            (37, (15, 8)),
        ] {
            assert_eq!(
                simplified(&notation, &simplifier, &stack(&notation, ups)),
                ratio,
                "{ups} ups"
            );
        }
    }

    #[test]
    fn the_l1_norm_is_what_decides() {
        // A step of 41et is 64/63 or 81/80, both of norm 25, and not 45/44,
        // which the quadratic norm alone prefers at 182 against 233.
        let (notation, simplifier) = simplifier(41, "2.3.5.7.11");
        assert_eq!(
            simplified(&notation, &simplifier, &stack(&notation, 1)),
            (64, 63)
        );

        // Under the quadratic norm the closest really is the undecimal comma,
        // so it is the L1 search that is doing this, not the reduction.
        let tempered = notation.temperament().temper(&stack(&notation, 1)).unwrap();
        let interval = notation.temperament().preimage(&tempered).unwrap();
        let weights = notation.subgroup().weights();
        let (closest, _) = cvp_exact(&interval, simplifier.lattice(), &weights, None).unwrap();
        let seed = subtract(&interval, &closest);
        assert_eq!(notation.subgroup().to_ratio(&seed).unwrap(), (45, 44));
    }

    #[test]
    fn the_hollows_a_local_walk_fell_into() {
        // The simplifier was once a walk of balls around a seed, and two cases
        // pinned where it went wrong. Fifteen octaves down, 9et reaches
        // 1/39366, of norm 29, where a walk one step wide stopped at 1/34560,
        // of norm 30.
        let (notation, simplifier) = simplifier(9, "2.3.5.7");
        let primes = notation.subgroup().basis();
        let far = notation.temperament().temper(&[-15, 0, 0, 0]).unwrap();
        let simplest = simplifier.simplify(&far).unwrap();
        assert_eq!(notation.subgroup().to_ratio(&simplest).unwrap(), (1, 39366));
        assert_eq!(sopfr(&simplest, primes), 29);
        assert_eq!(sopfr(&[-8, 3, 1, 0], primes), 30);

        // Six of augmented's accidental land where one ball's optimum,
        // [-31, 24, -3], is not the simplest: trading the fifth for two more
        // drops the 5 entirely and is one lighter.
        let subgroup: Subgroup = "2.3.5".parse().unwrap();
        let comma = subgroup.factorize(128, 125).unwrap();
        let temperament = Temperament::from_commas(&[comma], &subgroup).unwrap();
        let notation = Notation::from_temperament(&temperament).unwrap();
        let simplifier = Simplifier::new(&temperament);
        let simplest = simplifier.simplify_interval(&stack(&notation, 6)).unwrap();
        assert_eq!(
            subgroup.to_ratio(&simplest).unwrap(),
            (282_429_536_481, 274_877_906_944)
        );
        assert!(sopfr(&simplest, subgroup.basis()) < sopfr(&[-31, 24, -3], subgroup.basis()));
    }

    #[test]
    fn nothing_near_the_simplest_is_simpler() {
        // Checked independently of the enumeration: every interval within six
        // commas of the answer along each reduced comma ranks no better.
        let rank = |interval: &[i64], primes: &[u32]| {
            let squared: i64 = interval
                .iter()
                .zip(primes)
                .map(|(&e, &p)| i64::from(p * p) * e * e)
                .sum();
            (sopfr(interval, primes), squared)
        };
        for (divisions, subgroup) in [(22, "2.3.5"), (31, "2.3.5.7"), (41, "2.3.5.7")] {
            let (notation, simplifier) = simplifier(divisions, subgroup);
            let primes = notation.subgroup().basis();
            let lattice = simplifier.lattice();
            let width = 13i64;
            for step in 0..divisions {
                let simplest = simplifier.simplify(&[step]).unwrap();
                let best = rank(&simplest, primes);
                for code in 0..width.pow(lattice.len() as u32) {
                    let mut candidate = simplest.clone();
                    let mut remaining = code;
                    for row in lattice {
                        let times = remaining % width - width / 2;
                        remaining /= width;
                        for (c, &r) in candidate.iter_mut().zip(row) {
                            *c += times * r;
                        }
                    }
                    assert!(
                        rank(&candidate, primes) >= best,
                        "{divisions}et over {subgroup}, step {step}"
                    );
                }
            }
        }
    }

    #[test]
    fn simplifying_keeps_the_tempered_interval_and_settles() {
        for (divisions, subgroup) in [(41, "2.3.5.7.11"), (31, "2.3.5.7"), (22, "2.3.5")] {
            let (notation, simplifier) = simplifier(divisions, subgroup);
            assert!(
                notation.len() > 2,
                "{divisions}et over {subgroup} has no accidental"
            );
            let temperament = notation.temperament();
            for ups in -20..=60 {
                let stacked = stack(&notation, ups);
                let simplified = simplifier.simplify_interval(&stacked).unwrap();

                assert_eq!(
                    temperament.temper(&simplified).unwrap(),
                    temperament.temper(&stacked).unwrap()
                );
                assert!(
                    sopfr(&simplified, notation.subgroup().basis())
                        <= sopfr(&stacked, notation.subgroup().basis())
                );
                assert_eq!(
                    simplifier.simplify_interval(&simplified).unwrap(),
                    simplified
                );
            }
        }
    }

    #[test]
    fn simplifying_the_simplified_settles() {
        // The top candidate of a coset is already `simplify`'s fixed point:
        // asking again should return exactly what was asked.
        for (divisions, subgroup) in [(41, "2.3.5.7.11"), (31, "2.3.5.7"), (22, "2.3.5")] {
            let (notation, simplifier) = simplifier(divisions, subgroup);
            for ups in -20..=60 {
                let stacked = stack(&notation, ups);
                let simplified = simplifier.simplify_interval(&stacked).unwrap();
                assert_eq!(
                    simplifier.simplify_interval(&simplified).unwrap(),
                    simplified,
                    "{divisions}et over {subgroup}, {ups} ups"
                );
            }
        }
    }

    #[test]
    fn the_simplifications_are_ranked_readings_of_one_tempered_interval() {
        // Fifteen steps of 41et, where the simplest reading is not the most
        // convenient spelling: 9/7 costs three marks and means something, and
        // the runner up is spelled no better and means much less.
        let (notation, simplifier) = simplifier(41, "2.3.5.7.11");
        let subgroup = notation.subgroup();
        let target = stack(&notation, 15);

        let candidates = simplifier.simplifications_interval(&target, 3).unwrap();
        let ratios: Vec<(u64, u64)> = candidates
            .iter()
            .map(|candidate| subgroup.to_ratio(candidate).unwrap())
            .collect();
        assert_eq!(ratios, vec![(9, 7), (32, 25), (35, 27)]);

        // The first is what simplifying gives, the rest are ranked behind it,
        // and every one of them tempers to the same thing.
        assert_eq!(
            candidates[0],
            simplifier.simplify_interval(&target).unwrap()
        );
        let tempered = notation.temperament().temper(&target).unwrap();
        let mut norms = Vec::new();
        for candidate in simplifier.simplifications_interval(&target, 40).unwrap() {
            assert_eq!(notation.temperament().temper(&candidate).unwrap(), tempered);
            norms.push(sopfr(&candidate, subgroup.basis()));
        }
        assert!(norms.windows(2).all(|pair| pair[0] <= pair[1]));

        // Exactly as many as asked for, since the lattice never runs out.
        assert_eq!(norms.len(), 40);
        assert!(
            simplifier
                .simplifications_interval(&target, 0)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn just_intonation_has_nothing_to_simplify() {
        let subgroup: Subgroup = "2.3.5.7".parse().unwrap();
        let simplifier = Simplifier::new(&Temperament::from_ji(&subgroup).unwrap());

        assert!(simplifier.lattice().is_empty());
        let interval = subgroup.factorize(225, 224).unwrap();
        assert_eq!(simplifier.simplify_interval(&interval).unwrap(), interval);
    }

    #[test]
    fn the_coordinates_have_to_fit() {
        let (_, simplifier) = simplifier(41, "2.3.5.7.11");
        assert!(simplifier.simplify_interval(&[0, 0]).is_err());
        assert!(simplifier.simplify(&[0, 0]).is_err());
        assert_eq!(simplifier.simplify(&[41]).unwrap(), vec![1, 0, 0, 0, 0]);
    }
}
