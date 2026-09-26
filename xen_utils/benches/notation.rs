//! Timings of what runs repeatedly - spelling, simplifying, reading a spelling
//! back - and of building a notation, which runs once per temperament but
//! searches every subset of the accidentals.
//!
//! Each iteration runs over every step of the equal temperament, so the times
//! are per step, averaged over intervals that make the searches do real work.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use xen_utils::{Notation, Simplifier, Subgroup, Temperament};

const CASES: [(i64, &str); 3] = [
    (41, "2.3.5.7.11"),
    (72, "2.3.5.7.11"),
    (41, "2.3.5.7.11.13"),
];

struct Case {
    name: String,
    temperament: Temperament,
    notation: Notation,
    simplifier: Simplifier,
    /// Every step of the temperament.
    tempered: Vec<Vec<i64>>,
    /// The best spelling of each step.
    spellings: Vec<Vec<i64>>,
    /// The literal reading of each of those spellings.
    intervals: Vec<Vec<i64>>,
}

fn cases() -> Vec<Case> {
    CASES
        .iter()
        .map(|&(divisions, subgroup)| {
            let subgroup: Subgroup = subgroup.parse().unwrap();
            let temperament = Temperament::equal(divisions, &subgroup).unwrap();
            let notation = Notation::from_temperament(&temperament).unwrap();
            let simplifier = Simplifier::new(&temperament);
            let tempered: Vec<Vec<i64>> = (0..divisions).map(|step| vec![step]).collect();
            let spellings: Vec<Vec<i64>> = tempered
                .iter()
                .map(|t| notation.spell(t).unwrap())
                .collect();
            let intervals = spellings
                .iter()
                .map(|s| notation.to_interval(s).unwrap())
                .collect();
            Case {
                name: format!("{divisions}et {subgroup}"),
                temperament,
                notation,
                simplifier,
                tempered,
                spellings,
                intervals,
            }
        })
        .collect()
}

fn bench(c: &mut Criterion) {
    let cases = cases();

    let mut group = c.benchmark_group("Notation::spell");
    for case in &cases {
        group.throughput(Throughput::Elements(case.tempered.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for t in &case.tempered {
                    black_box(case.notation.spell(black_box(t)).unwrap());
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("Notation::spellings(4)");
    for case in &cases {
        group.throughput(Throughput::Elements(case.tempered.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for t in &case.tempered {
                    black_box(case.notation.spellings(black_box(t), 4).unwrap());
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("Notation::spell_interval");
    for case in &cases {
        group.throughput(Throughput::Elements(case.intervals.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for i in &case.intervals {
                    black_box(case.notation.spell_interval(black_box(i)).unwrap());
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("Notation::to_interval");
    for case in &cases {
        group.throughput(Throughput::Elements(case.spellings.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for s in &case.spellings {
                    black_box(case.notation.to_interval(black_box(s)).unwrap());
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("Notation::temper");
    for case in &cases {
        group.throughput(Throughput::Elements(case.spellings.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for s in &case.spellings {
                    black_box(case.notation.temper(black_box(s)).unwrap());
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("Simplifier::simplify");
    for case in &cases {
        group.throughput(Throughput::Elements(case.tempered.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for t in &case.tempered {
                    black_box(case.simplifier.simplify(black_box(t)).unwrap());
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("Simplifier::simplifications(8)");
    for case in &cases {
        group.throughput(Throughput::Elements(case.tempered.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for t in &case.tempered {
                    black_box(case.simplifier.simplifications(black_box(t), 8).unwrap());
                }
            });
        });
    }
    group.finish();

    // Built once per temperament, but the subset search makes it the one
    // construction worth watching.
    let mut group = c.benchmark_group("Notation::from_temperament");
    for case in &cases {
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| black_box(Notation::from_temperament(black_box(&case.temperament)).unwrap()));
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
