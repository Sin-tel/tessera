//! Lists every notation each temperament in `data/temperaments.txt` offers,
//! and how each one writes the primes it tempers.
//!
//! Each prime is written on its just nominal, `-` where it cannot be, followed
//! in parentheses by what `spell` chooses where that is something else. The
//! recommended notation is marked `->`.

use std::error::Error;

use xen_utils::{Notation, Subgroup, Temperament};

const TEMPERAMENTS: &str = include_str!("../data/temperaments.txt");

const HEADER: [&str; 6] = [
    "temperament",
    "subgroup",
    "rank",
    "",
    "accidentals",
    "on the nominals",
];

fn main() {
    let mut rows = vec![HEADER.map(String::from)];

    for (number, line) in TEMPERAMENTS.lines().enumerate() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        match rows_for(line) {
            Ok(entry) => rows.extend(entry),
            Err(error) => eprintln!("line {}: {error}", number + 1),
        }
    }

    print_table(&rows);
}

/// Parses one line of the list and lays out every notation it offers, one row
/// each, naming the temperament only on the first.
fn rows_for(line: &str) -> Result<Vec<[String; 6]>, Box<dyn Error>> {
    let mut fields = line.split('|').map(str::trim);
    let (Some(name), Some(subgroup), Some(definition), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return Err(format!("expected `name | subgroup | commas`, got {line:?}").into());
    };

    let subgroup: Subgroup = subgroup.parse()?;
    let temperament = parse_temperament(definition, &subgroup)?;

    let options = match Notation::options(&temperament) {
        Ok(options) => options,
        // A temperament with no notation still earns a row saying so.
        Err(error) => {
            return Ok(vec![[
                name.to_string(),
                subgroup.to_string(),
                temperament.rank().to_string(),
                String::new(),
                "-".to_string(),
                error.to_string(),
            ]]);
        }
    };

    let recommended = Notation::from_temperament(&temperament)?;

    Ok(options
        .iter()
        .enumerate()
        .map(|(index, notation)| {
            let first = index == 0;
            [
                if first {
                    name.to_string()
                } else {
                    String::new()
                },
                if first {
                    subgroup.to_string()
                } else {
                    String::new()
                },
                if first {
                    temperament.rank().to_string()
                } else {
                    String::new()
                },
                if notation.generators() == recommended.generators() {
                    "->".to_string()
                } else {
                    String::new()
                },
                accidentals(notation),
                spelling(notation),
            ]
        })
        .collect())
}

/// The accidentals of a notation, as ratios.
fn accidentals(notation: &Notation) -> String {
    let subgroup = notation.subgroup();
    let list: Vec<String> = notation.generators()[2..]
        .iter()
        .map(|a| {
            let (num, den) = subgroup.to_ratio(a).expect("an accidental is small");
            format!("{num}/{den}")
        })
        .collect();
    if list.is_empty() {
        "-".to_string()
    } else {
        list.join(" ")
    }
}

/// Parses the third field: `et N`, or a list of commas.
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

/// The rank of a notation and how it writes each prime beyond 3 on its
/// nominal, with what `spell` chooses where that differs.
fn spelling(notation: &Notation) -> String {
    let subgroup = notation.subgroup();
    let nominal = notation.nominal_spellings();
    let mut cell = format!("[{}]", notation.len());
    for (index, on_nominal) in (2..subgroup.dim()).zip(nominal) {
        let harmonic = octave_reduce(subgroup, index);
        let (num, den) = subgroup.to_ratio(&harmonic).expect("a single prime fits");
        let spelled = notation
            .spell_interval(&harmonic)
            .expect("built over subgroup");
        // The nominal spelling is of the prime itself, so bring it down by the
        // same octaves the harmonic was.
        let on_nominal = on_nominal.map(|mut coordinates| {
            coordinates[0] += harmonic[0];
            coordinates
        });
        let written = match &on_nominal {
            Some(coordinates) => notation.note(coordinates),
            None => "-".to_string(),
        };
        cell.push_str(&format!(" {num}/{den} {written}"));
        if on_nominal.as_ref() != Some(&spelled) {
            cell.push_str(&format!(" ({})", notation.note(&spelled)));
        }
    }
    cell
}

/// The prime at `index`, brought into the octave above the unison.
fn octave_reduce(subgroup: &Subgroup, index: usize) -> Vec<i64> {
    let mut interval = vec![0i64; subgroup.dim()];
    interval[index] = 1;
    interval[0] = -(subgroup.to_cents(&interval) / 1200.0).floor() as i64;
    interval
}

/// Prints rows padded to a common width, with a rule under the header.
fn print_table(rows: &[[String; 6]]) {
    let mut widths = [0; 6];
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }

    for (number, row) in rows.iter().enumerate() {
        let line: Vec<String> = row
            .iter()
            .zip(widths)
            .map(|(cell, width)| format!("{cell:width$}"))
            .collect();
        println!("{}", line.join("  ").trim_end());

        if number == 0 {
            let rule: Vec<String> = widths.iter().map(|&width| "-".repeat(width)).collect();
            println!("{}", rule.join("  "));
        }
    }
}
