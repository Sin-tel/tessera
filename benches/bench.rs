#[macro_use]
extern crate bencher;

use bencher::Bencher;

use audiolib::dsp::delayline::*;
use audiolib::dsp::*;

const ITERATIONS: u32 = 1000;
const SAMPLE_RATE: f32 = 44100.0;

fn run<F: FnMut(f32) -> f32>(bench: &mut Bencher, mut cb: F) {
	bench.iter(|| (0..ITERATIONS).fold(0.1, |a, b| a + cb((b as f32) / (ITERATIONS as f32))))
}

fn tanh_bench(bench: &mut Bencher) {
	run(bench, |b| b.tanh())
}

fn softclip_bench(bench: &mut Bencher) {
	run(bench, softclip)
}

fn softclip_cubic_bench(bench: &mut Bencher) {
	run(bench, softclip_cubic)
}

fn pitch_to_f_bench(bench: &mut Bencher) {
	run(bench, |p| pitch_to_f(p, 44100.0))
}

fn delay_go_back_int_bench(bench: &mut Bencher) {
	let mut line = DelayLine::new(SAMPLE_RATE, 512.0);
	run(bench, |p| line.go_back_int(p))
}

fn delay_go_back_linear_bench(bench: &mut Bencher) {
	let mut line = DelayLine::new(SAMPLE_RATE, 512.0);
	run(bench, |p| line.go_back_linear(p))
}

fn delay_go_back_cubic_bench(bench: &mut Bencher) {
	let mut line = DelayLine::new(SAMPLE_RATE, 512.0);
	run(bench, |p| line.go_back_cubic(p))
}

fn pow2_std_bench(bench: &mut Bencher) {
	run(bench, |b| 2.0_f32.powf(b))
}

fn pow2_fast_bench(bench: &mut Bencher) {
	run(bench, |b| pow2_fast(b))
}

benchmark_group!(
	benches,
	tanh_bench,
	softclip_bench,
	softclip_cubic_bench,
	pitch_to_f_bench,
	delay_go_back_int_bench,
	delay_go_back_linear_bench,
	delay_go_back_cubic_bench,
	pow2_std_bench,
	pow2_fast_bench,
);
benchmark_main!(benches);
