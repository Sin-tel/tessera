#[cfg(test)]
mod tests {

	use tessera::dsp::log2_cheap;
	use tessera::dsp::pow2_cheap;

	#[test]
	fn test_pow2_cheap_accuracy() {
		// Test on [-2.0, .., 2.0]
		let test_cases: Vec<f32> = (-20..=20).map(|i| i as f32 / 10.0).collect();
		let mut max_rel_error = 0.0f32;

		for &x in &test_cases {
			let approx = pow2_cheap(x);
			let exact = 2.0f32.powf(x);
			let rel_error = (approx - exact).abs() / exact;

			max_rel_error = max_rel_error.max(rel_error);

			println!(
				"x = {x:+.2}: approx = {approx:.6}, exact = {exact:.6}, rel_error = {rel_error:.6}",
			);
		}

		println!("Maximum relative error: {:.6}", max_rel_error);

		assert!(max_rel_error < 5e-5, "Relative error too large: {}", max_rel_error);

		// These should be exact
		assert_eq!(pow2_cheap(-1.0), 0.5);
		assert_eq!(pow2_cheap(0.0), 1.0);
		assert_eq!(pow2_cheap(1.0), 2.0);
		assert_eq!(pow2_cheap(2.0), 4.0);
		assert_eq!(pow2_cheap(3.0), 8.0);
		assert_eq!(pow2_cheap(4.0), 16.0);
	}

	#[test]
	fn test_log2_cheap_accuracy() {
		// Test on [0.1, .., 2.0]
		let test_cases: Vec<f32> = (1..=20).map(|i| i as f32 / 10.0).collect();

		let mut max_abs_error = 0.0f32;

		for &x in &test_cases {
			let approx = log2_cheap(x);
			let exact = x.log2();
			let abs_error = (approx - exact).abs();

			max_abs_error = max_abs_error.max(abs_error);

			println!(
				"x = {x:.2}: approx = {approx:+.6}, exact = {exact:+.6}, abs_error = {abs_error:0.6}",
			);
		}

		println!("Maximum absolute error: {:.6}", max_abs_error);

		assert!(max_abs_error < 2e-4, "Error too large: {}", max_abs_error);

		// These should be exact
		assert_eq!(log2_cheap(0.5), -1.0);
		assert_eq!(log2_cheap(1.0), 0.0);
		assert_eq!(log2_cheap(2.0), 1.0);
		assert_eq!(log2_cheap(4.0), 2.0);
		assert_eq!(log2_cheap(8.0), 3.0);
		assert_eq!(log2_cheap(16.0), 4.0);
	}
}
