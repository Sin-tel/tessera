//! Timings of what runs repeatedly - spelling, simplifying, reading a spelling
//! back - and of building a notation, which runs once per temperament but
//! searches every subset of the accidentals.
//!
//! Each iteration runs over every step of the equal temperament, so the times
//! are per step, averaged over intervals that make the searches do real work.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use xen_utils::{Notation, Subgroup, Temperament};

const CASES: [(i64, &str); 3] = [
    (41, "2.3.5.7.11"),
    (72, "2.3.5.7.11"),
    (41, "2.3.5.7.11.13"),
];

struct Case {
    name: String,
    temperament: Temperament,
    notation: Notation,
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
            let tempered: Vec<Vec<i64>> = (0..divisions).map(|step| vec![step]).collect();
            let spellings: Vec<Vec<i64>> =
                tempered.iter().map(|t| notation.from_tempered(t)).collect();
            let intervals = spellings.iter().map(|s| notation.to_interval(s)).collect();
            Case {
                name: format!("{divisions}et {subgroup}"),
                temperament,
                notation,
                tempered,
                spellings,
                intervals,
            }
        })
        .collect()
}

fn bench(c: &mut Criterion) {
    let cases = cases();

    let mut group = c.benchmark_group("Notation::from_tempered");
    for case in &cases {
        group.throughput(Throughput::Elements(case.tempered.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for t in &case.tempered {
                    black_box(case.notation.from_tempered(black_box(t)));
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("Notation::spellings(4)");
    for case in &cases {
        group.throughput(Throughput::Elements(case.spellings.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for s in &case.spellings {
                    black_box(case.notation.spellings(black_box(s), 4));
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("Notation::from_interval");
    for case in &cases {
        group.throughput(Throughput::Elements(case.intervals.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for i in &case.intervals {
                    black_box(case.notation.from_interval(black_box(i)));
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
                    black_box(case.notation.to_interval(black_box(s)));
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
                    black_box(case.notation.to_tempered(black_box(s)));
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("Notation::simplify");
    for case in &cases {
        group.throughput(Throughput::Elements(case.spellings.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for s in &case.spellings {
                    black_box(case.notation.simplify(black_box(s)));
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("Notation::simplifications(8)");
    for case in &cases {
        group.throughput(Throughput::Elements(case.spellings.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&case.name), case, |b, case| {
            b.iter(|| {
                for s in &case.spellings {
                    black_box(case.notation.simplifications(black_box(s), 8));
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
