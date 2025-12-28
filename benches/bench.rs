#![allow(unused_imports)]
#![allow(dead_code)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use tessera::dsp::delayline::*;
use tessera::dsp::*;

const SAMPLE_RATE: f32 = 44100.0;

fn criterion_benchmark(c: &mut Criterion) {
	c.bench_function("delay_cubic", |b| {
		let mut line = DelayLine::new(SAMPLE_RATE, 0.01);

		b.iter(|| {
			let _ = line.go_back_cubic(black_box(0.001));
		})
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
