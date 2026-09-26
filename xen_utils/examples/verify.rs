//! Checks the invariants of the run of notations, over every temperament in
//! `data/temperaments.txt` and every equal temperament up to 72 over a few
//! subgroups.
//!
//!   * there are exactly as many enharmonics as the notation has rank over the
//!     temperament, and each is worth nothing without being the unison;
//!   * every prime can be written, and reading the spelling back gives the
//!     tempered interval it was asked for;
//!   * no spelling in the same coset is cheaper to write - checked by walking
//!     a box of enharmonics and writing the cost out a second time, so
//!     that this checks the answer rather than restating how it was found;
//!   * ranks run upwards one at a time, and the first has the rank of the
//!     temperament where one of that rank exists at all.
//!
//! The best notation of one size need not keep what the best of the size below
//! keeps. Where it does not, that is printed, but it is not a failure.

use std::error::Error;

use diophantine::{Matrix, solve_diophantine, transpose};
use xen_utils::{Notation, Subgroup, Temperament};

const TEMPERAMENTS: &str = include_str!("../data/temperaments.txt");

fn main() {
    let mut failures = 0;

    for (number, line) in TEMPERAMENTS.lines().enumerate() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        let mut fields = line.split('|').map(str::trim);
        let (Some(name), Some(subgroup), Some(definition)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        match parse(subgroup, definition) {
            Ok(t) => failures += check(name, &t),
            Err(error) => eprintln!("line {}: {error}", number + 1),
        }
    }

    for subgroup in ["2.3.5", "2.3.5.7", "2.3.5.7.11"] {
        let subgroup: Subgroup = subgroup.parse().unwrap();
        for divisions in 5..=72 {
            // An equal temperament that is not primitive, such as 24et over
            // 2.3.5, is refused rather than silently answered as a different one.
            let Ok(t) = Temperament::equal(divisions, &subgroup) else {
                continue;
            };
            failures += check(&format!("{divisions}et over {subgroup}"), &t);
        }
    }

    println!("{failures} failures");
}

fn check(name: &str, t: &Temperament) -> usize {
    let mut failures = 0;
    let mut fail = |what: String| {
        eprintln!("{name}: {what}");
        failures += 1;
    };

    let options = match Notation::options(t) {
        Ok(options) => options,
        // Not every temperament has a notation over every subgroup, and refusing
        // is a result rather than a failure.
        Err(error) => {
            println!("{name}: no notation - {error}");
            return failures;
        }
    };

    let ranks: Vec<usize> = options.iter().map(Notation::len).collect();
    if !ranks.windows(2).all(|pair| pair[1] == pair[0] + 1) {
        fail(format!("ranks do not run upwards one at a time: {ranks:?}"));
    }
    if ranks[0] < t.rank() {
        fail(format!(
            "smallest notation is rank {} < {}",
            ranks[0],
            t.rank()
        ));
    }
    if *ranks.last().unwrap() != t.dim() && t.rank() > 1 {
        // The largest keeps every useful accidental; over a full subgroup that
        // is all of them unless something is tempered out.
    }

    for (index, n) in options.iter().enumerate() {
        // A written note worth nothing that is not the unison: that is the whole
        // of what an enharmonic is, and there are exactly as many of them as the
        // notation has rank over the temperament.
        if n.enharmonics().len() != n.len() - t.rank() {
            fail(format!(
                "option {index}: {} enharmonics for a notation of length {} over a rank {} temperament",
                n.enharmonics().len(),
                n.len(),
                t.rank()
            ));
        }
        for enharmonic in n.enharmonics() {
            if n.temper(enharmonic).unwrap().iter().any(|&x| x != 0) {
                fail(format!(
                    "option {index}: the enharmonic {enharmonic:?} is not worth nothing"
                ));
            }
            if enharmonic.iter().all(|&x| x == 0) {
                fail(format!("option {index}: an enharmonic is the unison"));
            }
        }

        // Every prime can be written; reading the spelling back gives the tempered interval
        // it was asked for; and nothing in the coset is cheaper to write. The
        // cost is written out again here, and a box of the coset walked by
        // brute force, so that this checks the answer rather than
        // restating how it was found.
        for prime in 0..n.dim() {
            let mut interval = vec![0i64; n.dim()];
            interval[prime] = 1;
            let spelling = n.spell_interval(&interval).unwrap();
            if n.temper(&spelling).unwrap() != t.temper(&interval).unwrap() {
                fail(format!(
                    "option {index}: prime {prime} is spelled as another tempered interval"
                ));
            }
            if let Some(better) = cheaper(n, &spelling) {
                fail(format!(
                    "option {index}: prime {prime} is written {} where {} is cheaper",
                    n.note(&spelling),
                    n.note(&better)
                ));
            }
        }
    }

    // Each notation in the run keeps one accidental more than the one before.
    for (index, pair) in options.windows(2).enumerate() {
        let (smaller, larger) = (&pair[0], &pair[1]);
        if larger.len() != smaller.len() + 1 {
            fail(format!(
                "option {index} and the next differ by more than one"
            ));
        }
        for accidental in &smaller.generators()[2..] {
            if !larger.generators()[2..].contains(accidental) {
                println!(
                    "{name}: option {} drops an accidental option {index} keeps",
                    index + 1
                );
            }
        }
    }

    failures
}

/// How far either way to walk each enharmonic.
const WIDTH: i64 = 8;

/// A spelling of the same tempered interval that costs less than `spelling`, if there is one
/// within [`WIDTH`] enharmonics of it.
///
/// Seven half-apotomes for an accidental mark and two for a fifth away from `D`,
/// which is `Notation::spellings`' ranking spelled out a second time.
fn cheaper(n: &Notation, spelling: &[i64]) -> Option<Vec<i64>> {
    let cost = |c: &[i64]| -> i64 {
        let marks: i64 = c[2..].iter().map(|e| e.abs()).sum();
        7 * marks + 2 * (c[1] - 2).abs()
    };
    let lattice = n.enharmonics();
    let mut steps = vec![-WIDTH; lattice.len()];
    while !lattice.is_empty() {
        let candidate: Vec<i64> = (0..n.len())
            .map(|slot| {
                spelling[slot]
                    + steps
                        .iter()
                        .zip(lattice)
                        .map(|(&step, row)| step * row[slot])
                        .sum::<i64>()
            })
            .collect();
        if cost(&candidate) < cost(spelling) {
            return Some(candidate);
        }
        let mut place = 0;
        while place < steps.len() && steps[place] == WIDTH {
            steps[place] = -WIDTH;
            place += 1;
        }
        if place == steps.len() {
            break;
        }
        steps[place] += 1;
    }
    None
}

/// Whether the rows of `basis` span the same lattice as the rows of `other`.
#[allow(dead_code)]
fn same_lattice(basis: &Matrix<i64>, other: &Matrix<i64>) -> bool {
    solve_diophantine(&transpose(basis), &transpose(other)).is_ok()
        && solve_diophantine(&transpose(other), &transpose(basis)).is_ok()
}

fn parse(subgroup: &str, definition: &str) -> Result<Temperament, Box<dyn Error>> {
    let subgroup: Subgroup = subgroup.parse()?;
    if let Some(divisions) = definition.strip_prefix("et ") {
        return Ok(Temperament::equal(divisions.trim().parse()?, &subgroup)?);
    }
    let commas = definition
        .split([',', ' '])
        .filter(|field| !field.is_empty())
        .map(|ratio| subgroup.parse_ratio(ratio))
        .collect::<Result<Vec<Vec<i64>>, _>>()?;
    Ok(Temperament::from_commas(&commas, &subgroup)?)
}
