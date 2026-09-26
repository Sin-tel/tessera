//! Twelve miracle generators, notated three ways.
//!
//! Miracle's generator is `15/14`, and the temperament says six of them are a
//! fifth, so twelve are `9/4`.

use xen_utils::{Notation, Simplifier, Subgroup, Temperament};

const STEPS: i64 = 12;

fn main() {
    let subgroup: Subgroup = "2.3.5.7.11".parse().unwrap();
    let commas = [(225, 224), (1029, 1024), (385, 384)]
        .map(|(num, den)| subgroup.factorize(num, den).unwrap());
    let temperament = Temperament::from_commas(&commas, &subgroup).unwrap();
    let options = Notation::options(&temperament).unwrap();
    let recommended = Notation::from_temperament(&temperament).unwrap();
    let simplifier = Simplifier::new(&temperament);

    println!("miracle over {subgroup}, {} notations", options.len());
    println!("comma lattice, reduced:");
    for comma in simplifier.lattice() {
        let ascending = subgroup.ascending(comma);
        println!("    {}", ratio(&subgroup, &ascending));
    }

    let generator = subgroup.factorize(15, 14).unwrap();
    println!(
        "\nsteps  {:>9}  {}",
        "simplest",
        options
            .iter()
            .map(|n| {
                let mark = if n.generators() == recommended.generators() {
                    "*"
                } else {
                    " "
                };
                format!("{:6}", format!("[{}]{mark}", n.len()))
            })
            .collect::<Vec<_>>()
            .join("  ")
    );

    for steps in 0..=STEPS {
        let stacked: Vec<i64> = generator.iter().map(|&e| e * steps).collect();
        let simplified = simplifier.simplify_interval(&stacked).unwrap();
        let notes: Vec<String> = options
            .iter()
            .map(|n| n.note(&n.spell_interval(&simplified).unwrap()))
            .collect();

        println!(
            "{steps:5}  {:>9}  {}",
            ratio(&subgroup, &simplified),
            notes
                .iter()
                .map(|note| format!("{note:6}"))
                .collect::<Vec<_>>()
                .join("  ")
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
