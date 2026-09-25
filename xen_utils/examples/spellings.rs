//! The best spellings of every step of an equal temperament.

use xen_utils::{Notation, Simplifier, Subgroup, Temperament};

const DIVISIONS: i64 = 41;
const SUBGROUP: &str = "2.3.5.7.11";
const SHOWN: usize = 3;

fn main() {
    for (divisions, subgroup) in [(12, "2.3.5"), (DIVISIONS, SUBGROUP)] {
        let subgroup: Subgroup = subgroup.parse().unwrap();
        let temperament = Temperament::equal(divisions, &subgroup).unwrap();
        let notation = Notation::from_temperament(&temperament).unwrap();
        let simplifier = Simplifier::new(&temperament);

        println!(
            "\n{divisions}et over {subgroup}, notation of length {}",
            notation.len()
        );
        println!(
            "enharmonics: {}",
            notation
                .enharmonics()
                .iter()
                .map(|e| format!("{e:?}"))
                .collect::<Vec<_>>()
                .join("  ")
        );
        println!("\n{:>4}  {:>9}  ways to write it", "step", "reading");

        for steps in 0..divisions + 1 {
            let tempered = [steps];
            let reading = simplifier.simplify(&tempered).unwrap();
            let (num, den) = subgroup.to_ratio(&reading).unwrap();

            let ways: Vec<String> = notation
                .spellings(&tempered, SHOWN)
                .unwrap()
                .iter()
                .map(|c| format!("{:10}", notation.note(c)))
                .collect();
            println!(
                "{steps:4}  {:>9}  {}",
                format!("{num}/{den}"),
                ways.join("")
            );
        }
    }
}
