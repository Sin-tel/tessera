//! The simplest just intervals of a few tempered intervals of an equal
//! temperament.

use xen_utils::{Notation, Simplifier, Subgroup, Temperament, simplify::sopfr};

const DIVISIONS: i64 = 41;
const STEPS: [i64; 4] = [14, 15, 20, 27];
const SHOWN: usize = 5;

fn main() {
    let subgroup: Subgroup = "2.3.5.7.11".parse().unwrap();
    let temperament = Temperament::equal(DIVISIONS, &subgroup).unwrap();
    let notation = Notation::from_temperament(&temperament).unwrap();
    let simplifier = Simplifier::new(&temperament);

    for steps in STEPS {
        let tempered = [steps];
        let cents = 1200.0 * steps as f64 / DIVISIONS as f64;
        let written = notation.note(&notation.spell(&tempered).unwrap());
        println!("{steps} steps of {DIVISIONS}et, {cents:.1}c, written {written}");

        for candidate in simplifier.simplifications(&tempered, SHOWN).unwrap() {
            let (numerator, denominator) = subgroup.to_ratio(&candidate).unwrap();
            println!(
                "    {:>7}  norm {:3}  {:+6.1}c",
                format!("{numerator}/{denominator}"),
                sopfr(&candidate, subgroup.basis()),
                subgroup.to_cents(&candidate) - cents,
            );
        }
        println!();
    }
}
