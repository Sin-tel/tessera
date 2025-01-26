#![allow(unused_imports)]
#![allow(dead_code)]

#[macro_use]
extern crate bencher;

use bencher::Bencher;
use fastrand::Rng;
use std::f32::consts::PI;
use tessera::dsp::delayline::*;
use tessera::dsp::resample::*;
use tessera::dsp::*;

const ITERATIONS: u32 = 1000;
const SAMPLE_RATE: f32 = 44100.0;

fn tanhdx(x: f32) -> f32 {
	let a = x * x;
	((a + 105.0) * a + 945.0) / ((15.0 * a + 420.0) * a + 945.0)
}

fn prewarp_tan(x: f32) -> f32 {
	(x.min(0.49) * PI).tan()
}

fn distdx(x: f32) -> f32 {
	let a = 0.135;
	a + (1.0 - a) / (1.0 + 10.0 * x * x).sqrt()
}

fn dist2dx(x: f32) -> f32 {
	1.0 / (1.0 + x.abs() + 0.2 * x)
}

fn run<F: FnMut(f32) -> f32>(bench: &mut Bencher, mut cb: F) {
	bench.iter(|| (0..ITERATIONS).fold(0.1, |a, b| a + cb((b as f32) / (ITERATIONS as f32))))
}

fn tanh_bench(bench: &mut Bencher) {
	run(bench, |b| b.tanh())
}

fn tanhdx_bench(bench: &mut Bencher) {
	run(bench, |b| tanhdx(b))
}

fn distdx_bench(bench: &mut Bencher) {
	run(bench, |b| distdx(b))
}

fn dist2dx_bench(bench: &mut Bencher) {
	run(bench, |b| dist2dx(b))
}

fn softclip_bench(bench: &mut Bencher) {
	run(bench, softclip)
}

fn softclip_cubic_bench(bench: &mut Bencher) {
	run(bench, softclip_cubic)
}

fn sin_bench(bench: &mut Bencher) {
	run(bench, |b| (TWO_PI * b).sin())
}

fn sin_cheap_bench(bench: &mut Bencher) {
	run(bench, |b| sin_cheap(b))
}

fn prewarp_bench(bench: &mut Bencher) {
	run(bench, prewarp_tan)
}

fn prewarp_cheap_bench(bench: &mut Bencher) {
	run(bench, prewarp)
}

fn pitch_to_hz_bench(bench: &mut Bencher) {
	run(bench, |p| pitch_to_hz(p))
}

fn delay_go_back_int_bench(bench: &mut Bencher) {
	let line = DelayLine::new(SAMPLE_RATE, 512.0);
	run(bench, |p| line.go_back_int(p))
}

fn delay_go_back_linear_bench(bench: &mut Bencher) {
	let line = DelayLine::new(SAMPLE_RATE, 512.0);
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
	run(bench, |b| pow2_cheap(b))
}

fn log2_std_bench(bench: &mut Bencher) {
	run(bench, |b| b.log2())
}

fn log2_fast_bench(bench: &mut Bencher) {
	run(bench, |b| log2_cheap(b))
}

fn floor_bench(bench: &mut Bencher) {
	run(bench, |b| b.floor())
}

fn round_bench(bench: &mut Bencher) {
	run(bench, |b| b.round())
}

fn rand_bench(bench: &mut Bencher) {
	let mut rng = fastrand::Rng::new();
	run(bench, |_| rng.f32())
}

fn upsample_bench(bench: &mut Bencher) {
	let mut upsampler = Upsampler31::new();
	run(bench, |b| {
		let (x1, x2) = upsampler.process(b);
		x1 * x2
	})
}

fn downsample_bench(bench: &mut Bencher) {
	let mut downsampler = Downsampler51::new();
	run(bench, |b| downsampler.process(b, b + 0.5))
}

fn svf_bench(bench: &mut Bencher) {
	let mut filter = tessera::dsp::simper::Filter::new(44100.0);
	filter.set_lowpass(500.0, 0.7);
	let mut i = 0;
	run(bench, |x| {
		i += 1;
		if i >= 64 {
			i = 0;
			filter.set_lowpass(x, 0.7);
		}
		filter.process(x)
	})
}

fn svf_phase_bench(bench: &mut Bencher) {
	let mut filter = tessera::dsp::simper::Filter::new(44100.0);
	filter.set_lowpass(500.0, 0.7);
	run(bench, |x| filter.phase_delay(x))
}

fn onepole_bench(bench: &mut Bencher) {
	let mut filter = tessera::dsp::onepole::OnePole::new(44100.0);
	filter.set_lowpass(500.0);
	let mut i = 0;
	run(bench, |x| {
		i += 1;
		if i >= 64 {
			i = 0;
			filter.set_lowpass(x);
		}
		filter.process(x)
	})
}

fn dckiller_bench(bench: &mut Bencher) {
	let mut filter = tessera::dsp::DcKiller::new(44100.0);
	run(bench, |x| filter.process(x))
}

fn skf_bench(bench: &mut Bencher) {
	let mut filter = tessera::dsp::skf::Skf::new(44100.0);
	filter.set(500.0, 0.7);
	let mut i = 0;
	run(bench, |x| {
		i += 1;
		if i >= 64 {
			i = 0;
			filter.set(x, 0.7);
		}
		filter.process_lowpass(x)
	})
}

benchmark_group!(
	benches,
	// tanh_bench,
	// tanhdx_bench,
	// softclip_bench,
	// dist2dx_bench,
	// distdx_bench,
	// prewarp_bench,
	// prewarp_cheap_bench,
	// softclip_cubic_bench,
	// sin_bench,
	// sin_cheap_bench,
	// pitch_to_hz_bench,
	// delay_go_back_int_bench,
	// delay_go_back_linear_bench,
	// delay_go_back_cubic_bench,
	// pow2_std_bench,
	// pow2_fast_bench,
	// log2_std_bench,
	// log2_fast_bench,
	// rand_bench,
	// floor_bench,
	// round_bench,
	svf_bench,
	// onepole_bench,
	// dckiller_bench,
	svf_phase_bench,
	// svf_phase2_bench,
	// skf_bench,
	// upsample_bench,
	// downsample_bench,
);
benchmark_main!(benches);
