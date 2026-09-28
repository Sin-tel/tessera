//! Tests for simplifying: [`Notation::simplify`] and [`Notation::simplifications`].

use diophantine::cvp_exact;

use crate::Notation;
use crate::primes::Subgroup;
use crate::temperament::Temperament;
use crate::util::subtract;

/// A notation of the equal temperament of `divisions` over `subgroup`.
///
/// The largest of the run rather than the recommended one, since these tests
/// stack a notation's first accidental and the recommendation is free to
/// keep none. Simplifying does not depend on the notation either way.
fn notation(divisions: i64, subgroup: &str) -> Notation {
    let subgroup: Subgroup = subgroup.parse().unwrap();
    let temperament = Temperament::equal(divisions, &subgroup).unwrap();
    Notation::options(&temperament).unwrap().pop().unwrap()
}

/// `ups` of the first accidental stacked on `C`.
fn stack(notation: &Notation, ups: i64) -> Vec<i64> {
    let mut spelling = vec![0; notation.len()];
    spelling[2] = ups;
    spelling
}

/// What `spelling` simplifies to, as a ratio.
fn simplified(notation: &Notation, spelling: &[i64]) -> (u64, u64) {
    notation
        .subgroup()
        .to_ratio(&notation.simplify(spelling))
        .unwrap()
}

#[test]
fn a_target_too_far_to_search_still_answers() {
    // Past the budget the answer is the best found rather than the simplest
    // there is, but it is still a just interval that tempers to what was
    // asked for, and it still comes back quickly.
    let subgroup: Subgroup = "2.3.5.7.11".parse().unwrap();
    let t = Temperament::equal(41, &subgroup).unwrap();
    let n = Notation::from_temperament(&t).unwrap();
    for steps in [1_000_000i64, 100_000_000] {
        let simplifications = n.simplifications(&n.from_tempered(&[steps]), 3);
        assert!(!simplifications.is_empty());
        for interval in &simplifications {
            assert_eq!(t.temper(interval), vec![steps]);
        }
    }
}

#[test]
fn the_wilson_norm_is_the_sum_of_prime_factors() {
    let subgroup = Subgroup::p_limit(11);
    // 81/80 is 3+3+3+3 over 2+2+2+2+5, and 45/44 is 3+3+5 over 2+2+11.
    assert_eq!(subgroup.sopfr(&[-4, 4, -1, 0, 0]), 25);
    assert_eq!(subgroup.sopfr(&[-2, 2, 1, 0, -1]), 26);
    assert_eq!(subgroup.sopfr(&[1, 0, 0, 0, 0]), 2);
    assert_eq!(subgroup.sopfr(&[0; 5]), 0);
}

#[test]
fn a_stack_of_ups_simplifies_to_the_octave() {
    // The accidental of 41et is worth one step, so 41 of them are an octave,
    // though as a just interval they are 81/80 forty one times over.
    let notation = notation(41, "2.3.5.7.11");
    let stacked = stack(&notation, 41);
    assert_eq!(notation.to_interval(&stacked), vec![-164, 164, -41, 0, 0]);

    assert_eq!(notation.simplify(&stacked), vec![1, 0, 0, 0, 0]);
    assert_eq!(notation.note(&notation.spell(&stacked)), "C6");
}

#[test]
fn forty_one_ups_are_the_interval_table_of_41et() {
    // The stack reaches every step of 41et, and what it simplifies to is
    // what those steps are usually called.
    let notation = notation(41, "2.3.5.7.11");
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
            simplified(&notation, &stack(&notation, ups)),
            ratio,
            "{ups} ups"
        );
    }
}

#[test]
fn the_l1_norm_is_what_decides() {
    // A step of 41et is 64/63 or 81/80, both of norm 25, and not 45/44,
    // which the quadratic norm alone prefers at 182 against 233.
    let notation = notation(41, "2.3.5.7.11");
    assert_eq!(simplified(&notation, &stack(&notation, 1)), (64, 63));

    // Under the quadratic norm the closest really is the undecimal comma,
    // so it is the L1 search that is doing this, not the reduction.
    let interval = notation.to_interval(&stack(&notation, 1));
    let weights = notation.subgroup().weights();
    let (closest, _) = cvp_exact(&interval, notation.commas(), &weights, None).unwrap();
    let seed = subtract(&interval, &closest);
    assert_eq!(notation.subgroup().to_ratio(&seed).unwrap(), (45, 44));
}

#[test]
fn the_hollows_a_local_walk_fell_into() {
    // The simplifier was once a walk of balls around a seed, and two cases
    // pinned where it went wrong. Fifteen octaves down, 9et reaches
    // 1/39366, of norm 29, where a walk one step wide stopped at 1/34560,
    // of norm 30.
    let notation = notation(9, "2.3.5.7");
    let subgroup = notation.subgroup();
    let mut far = vec![0; notation.len()];
    far[0] = -15;
    let simplest = notation.simplify(&far);
    assert_eq!(subgroup.to_ratio(&simplest).unwrap(), (1, 39366));
    assert_eq!(subgroup.sopfr(&simplest), 29);
    assert_eq!(subgroup.sopfr(&[-8, 3, 1, 0]), 30);

    // Six of augmented's accidental land where one ball's optimum,
    // [-31, 24, -3], is not the simplest: trading the fifth for two more
    // drops the 5 entirely and is one lighter.
    let subgroup: Subgroup = "2.3.5".parse().unwrap();
    let comma = subgroup.factorize(128, 125).unwrap();
    let temperament = Temperament::from_commas(&[comma], &subgroup).unwrap();
    let notation = Notation::from_temperament(&temperament).unwrap();
    let simplest = notation.simplify(&stack(&notation, 6));
    assert_eq!(
        subgroup.to_ratio(&simplest).unwrap(),
        (282_429_536_481, 274_877_906_944)
    );
    assert!(subgroup.sopfr(&simplest) < subgroup.sopfr(&[-31, 24, -3]));
}

#[test]
fn nothing_near_the_simplest_is_simpler() {
    // Checked independently of the enumeration: every interval within six
    // commas of the answer along each reduced comma ranks no better.
    for (divisions, subgroup) in [(22, "2.3.5"), (31, "2.3.5.7"), (41, "2.3.5.7")] {
        let notation = notation(divisions, subgroup);
        let primes = notation.subgroup().basis();
        let rank = |interval: &[i64]| {
            let squared: i64 = interval
                .iter()
                .zip(primes)
                .map(|(&e, &p)| i64::from(p * p) * e * e)
                .sum();
            (notation.subgroup().sopfr(interval), squared)
        };
        let lattice = notation.commas();
        let width = 13i64;
        for step in 0..divisions {
            let simplest = notation.simplify(&notation.from_tempered(&[step]));
            let best = rank(&simplest);
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
                    rank(&candidate) >= best,
                    "{divisions}et over {subgroup}, step {step}"
                );
            }
        }
    }
}

#[test]
fn simplifying_keeps_the_tempered_interval() {
    for (divisions, subgroup) in [(41, "2.3.5.7.11"), (31, "2.3.5.7"), (22, "2.3.5")] {
        let notation = notation(divisions, subgroup);
        assert!(
            notation.len() > 2,
            "{divisions}et over {subgroup} has no accidental"
        );
        let subgroup = notation.subgroup();
        for ups in -20..=60 {
            let stacked = stack(&notation, ups);
            let simplified = notation.simplify(&stacked);

            assert_eq!(
                notation.temperament().temper(&simplified),
                notation.to_tempered(&stacked)
            );
            let literal = notation.to_interval(&stacked);
            assert!(subgroup.sopfr(&simplified) <= subgroup.sopfr(&literal));
        }
    }
}

#[test]
fn simplifying_the_simplified_settles() {
    // The top candidate of a coset is already `simplify`'s fixed point:
    // spelling it and asking again should return exactly what was asked,
    // though the search starts from another reading.
    for (divisions, subgroup) in [(41, "2.3.5.7.11"), (31, "2.3.5.7"), (22, "2.3.5")] {
        let notation = notation(divisions, subgroup);
        for ups in -20..=60 {
            let simplified = notation.simplify(&stack(&notation, ups));
            let respelled = notation.from_interval(&simplified);
            assert_eq!(
                notation.simplify(&respelled),
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
    let notation = notation(41, "2.3.5.7.11");
    let subgroup = notation.subgroup();
    let target = stack(&notation, 15);

    let candidates = notation.simplifications(&target, 3);
    let ratios: Vec<(u64, u64)> = candidates
        .iter()
        .map(|candidate| subgroup.to_ratio(candidate).unwrap())
        .collect();
    assert_eq!(ratios, vec![(9, 7), (32, 25), (35, 27)]);

    // The first is what simplifying gives, the rest are ranked behind it,
    // and every one of them tempers to the same thing.
    assert_eq!(candidates[0], notation.simplify(&target));
    let tempered = notation.to_tempered(&target);
    let mut norms = Vec::new();
    for candidate in notation.simplifications(&target, 40) {
        assert_eq!(notation.temperament().temper(&candidate), tempered);
        norms.push(subgroup.sopfr(&candidate));
    }
    assert!(norms.windows(2).all(|pair| pair[0] <= pair[1]));

    // Exactly as many as asked for, since the lattice never runs out.
    assert_eq!(norms.len(), 40);
    assert!(notation.simplifications(&target, 0).is_empty());
}

#[test]
fn just_intonation_has_nothing_to_simplify() {
    let subgroup: Subgroup = "2.3.5.7".parse().unwrap();
    let notation = Notation::from_ji(&subgroup).unwrap();

    assert!(notation.commas().is_empty());
    let interval = subgroup.factorize(225, 224).unwrap();
    assert_eq!(
        notation.simplify(&notation.from_interval(&interval)),
        interval
    );
}

#[test]
#[should_panic(expected = "one entry per notation coordinate")]
fn the_coordinates_have_to_fit() {
    notation(41, "2.3.5.7.11").simplify(&[0, 0]);
}
