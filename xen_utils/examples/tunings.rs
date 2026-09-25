//! The Weil-Euclidean tuning of every temperament in `data/temperaments.txt`.

use std::error::Error;

use xen_utils::{Subgroup, Temperament, Tuning};

const TEMPERAMENTS: &str = include_str!("../data/temperaments.txt");

fn main() {
    for line in TEMPERAMENTS.lines() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('|').map(str::trim).collect();
        let subgroup: Subgroup = fields[1].parse().unwrap();
        let temperament = match parse_temperament(fields[2], &subgroup) {
            Ok(t) => t,
            Err(e) => {
                println!("{:20} {e}", fields[0]);
                continue;
            }
        };
        let tuning = Tuning::weil_euclidean(&temperament);

        let generators: Vec<String> = tuning
            .generators()
            .iter()
            .map(|g| format!("{g:.3}"))
            .collect();
        let errors: Vec<String> = (0..subgroup.dim())
            .map(|i| {
                let mut prime = vec![0; subgroup.dim()];
                prime[i] = 1;
                let error = tuning.pitch_interval(&prime).unwrap() - subgroup.to_cents(&prime);
                format!("{}:{error:+.3}", subgroup.basis()[i])
            })
            .collect();
        let octave = tuning.pitch_interval(&unit(&subgroup, 0)).unwrap();
        let mut fifth = unit(&subgroup, 1);
        fifth[0] = -1;
        let fifth = tuning.pitch_interval(&fifth).unwrap();

        println!(
            "{:20} {:12} octave {octave:9.3}  fifth {fifth:8.3}  gens [{}]",
            fields[0],
            subgroup.to_string(),
            generators.join(", ")
        );
        println!("{:33} errors {}", "", errors.join("  "));
    }
}

fn unit(subgroup: &Subgroup, index: usize) -> Vec<i64> {
    let mut v = vec![0; subgroup.dim()];
    v[index] = 1;
    v
}

fn parse_temperament(definition: &str, subgroup: &Subgroup) -> Result<Temperament, Box<dyn Error>> {
    if let Some(divisions) = definition.strip_prefix("et ") {
        return Ok(Temperament::equal(divisions.trim().parse()?, subgroup)?);
    }
    let commas = definition
        .split([',', ' '])
        .filter(|field| !field.is_empty())
        .map(|ratio| subgroup.parse_ratio(ratio))
        .collect::<Result<Vec<Vec<i64>>, _>>()?;
    Ok(Temperament::from_commas(&commas, subgroup)?)
}
