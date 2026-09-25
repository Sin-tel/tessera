//! Simplifying intervals, in 41et.
//!
//! The accidental of 41et's recommended notation is worth one step, so stacking
//! ups on `C` walks through the whole scale and arrives back at `C` an octave
//! later. The intervals that stack reaches are absurd - a fifth is reached by
//! (81/80)^24.

use xen_utils::{Notation, Simplifier, Subgroup, Temperament};

const DIVISIONS: i64 = 41;

fn main() {
    let subgroup: Subgroup = "2.3.5.7.11".parse().unwrap();
    let temperament = Temperament::equal(DIVISIONS, &subgroup).unwrap();
    let notation = Notation::from_temperament(&temperament).unwrap();
    let simplifier = Simplifier::new(&temperament);

    println!(
        "41et over {subgroup}, notation of length {}",
        notation.len()
    );
    println!("comma lattice, reduced:");
    for comma in simplifier.lattice() {
        let ascending = subgroup.ascending(comma);
        println!("    {:?}  {}", ascending, ratio(&subgroup, &ascending));
    }
    println!("\nups  simplest  note");

    for ups in 0..=DIVISIONS {
        let mut spelling = vec![0; notation.len()];
        spelling[2] = ups;

        let stacked = notation.to_interval(&spelling).unwrap();
        let simplified = simplifier.simplify_interval(&stacked).unwrap();
        println!(
            "{ups:3}  {:8}  {:6}",
            ratio(&subgroup, &simplified),
            notation.note(&notation.spell_interval(&simplified).unwrap()),
        );
    }
}

/// `interval` as a ratio, or its exponents where that overflows.
fn ratio(subgroup: &Subgroup, interval: &[i64]) -> String {
    match subgroup.to_ratio(interval) {
        Ok((numerator, denominator)) => format!("{numerator}/{denominator}"),
        Err(_) => format!("{interval:?}"),
    }
}
