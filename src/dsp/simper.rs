// after Andrew Simper, Cytomic, 2013, andy@cytomic.com
// see: https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf

#[derive(Debug)]
pub enum FilterSettings {
	Low(f32, f32),
	Band(f32, f32),
	High(f32, f32),
	Notch(f32, f32),
	All(f32, f32),
	Bell(f32, f32, f32),
	LowShelf(f32, f32, f32),
	HighShelf(f32, f32, f32),
}

#[derive(Debug, Default)]
pub struct Filter {
	ic1eq: f32,
	ic2eq: f32,
	a1: f32,
	a2: f32,
	a3: f32,
	m0: f32,
	m1: f32,
	m2: f32,
	sample_rate: f32,
}

impl Filter {
	pub fn new(sample_rate: f32) -> Self {
		Self {
			sample_rate,
			ic1eq: 0.0,
			ic2eq: 0.0,
			a1: 0.0,
			a2: 0.0,
			a3: 0.0,
			m0: 0.0,
			m1: 0.0,
			m2: 0.0,
		}
	}
	pub fn set(&mut self, t: FilterSettings) {
		match t {
			FilterSettings::Low(cutoff, q) => {
				let g = (std::f32::consts::PI * cutoff / self.sample_rate).tan();
				let k = 1.0 / q;
				self.a1 = 1.0 / (1.0 + g * (g + k));
				self.a2 = g * self.a1;
				self.a3 = g * self.a2;
				self.m0 = 0.0;
				self.m1 = 0.0;
				self.m2 = 1.0;
			}
			FilterSettings::Band(cutoff, q) => {
				let g = (std::f32::consts::PI * cutoff / self.sample_rate).tan();
				let k = 1.0 / q;
				self.a1 = 1.0 / (1.0 + g * (g + k));
				self.a2 = g * self.a1;
				self.a3 = g * self.a2;
				self.m0 = 0.0;
				self.m1 = 1.0;
				self.m2 = 0.0;
			}
			FilterSettings::High(cutoff, q) => {
				let g = (std::f32::consts::PI * cutoff / self.sample_rate).tan();
				let k = 1.0 / q;
				self.a1 = 1.0 / (1.0 + g * (g + k));
				self.a2 = g * self.a1;
				self.a3 = g * self.a2;
				self.m0 = 1.0;
				self.m1 = -k;
				self.m2 = -1.0;
			}
			FilterSettings::Notch(cutoff, q) => {
				let g = (std::f32::consts::PI * cutoff / self.sample_rate).tan();
				let k = 1.0 / q;
				self.a1 = 1.0 / (1.0 + g * (g + k));
				self.a2 = g * self.a1;
				self.a3 = g * self.a2;
				self.m0 = 1.0;
				self.m1 = -k;
				self.m2 = 0.0;
			}
			FilterSettings::All(cutoff, q) => {
				let g = (std::f32::consts::PI * cutoff / self.sample_rate).tan();
				let k = 1.0 / q;
				self.a1 = 1.0 / (1.0 + g * (g + k));
				self.a2 = g * self.a1;
				self.a3 = g * self.a2;
				self.m0 = 1.0;
				self.m1 = -2.0 * k;
				self.m2 = 0.0;
			}
			FilterSettings::Bell(cutoff, q, gain) => {
				let a = (10.0f32).powf(gain / 40.0);
				let g = (std::f32::consts::PI * cutoff / self.sample_rate).tan();
				let k = 1.0 / (q * a);
				self.a1 = 1.0 / (1.0 + g * (g + k));
				self.a2 = g * self.a1;
				self.a3 = g * self.a2;
				self.m0 = 1.0;
				self.m1 = k * (a * a - 1.0);
				self.m2 = 0.0;
			}
			FilterSettings::LowShelf(cutoff, q, gain) => {
				let a = (10.0f32).powf(gain / 40.0);
				let g = (std::f32::consts::PI * cutoff / self.sample_rate).tan() / a.sqrt();
				let k = 1.0 / (q);
				self.a1 = 1.0 / (1.0 + g * (g + k));
				self.a2 = g * self.a1;
				self.a3 = g * self.a2;
				self.m0 = 1.0;
				self.m1 = k * (a - 1.0);
				self.m2 = a * a - 1.0;
			}
			FilterSettings::HighShelf(cutoff, q, gain) => {
				let a = (10.0f32).powf(gain / 40.0);
				let g = (std::f32::consts::PI * cutoff / self.sample_rate).tan() / a.sqrt();
				let k = 1.0 / (q);
				self.a1 = 1.0 / (1.0 + g * (g + k));
				self.a2 = g * self.a1;
				self.a3 = g * self.a2;
				self.m0 = a * a;
				self.m1 = k * (1.0 - a) * a;
				self.m2 = 1.0 - a * a;
			}
		}
	}
	pub fn process(&mut self, v0: f32) -> f32 {
		let v3 = v0 - self.ic2eq;
		let v1 = self.a1 * self.ic1eq + self.a2 * v3;
		let v2 = self.ic2eq + self.a2 * self.ic1eq + self.a3 * v3;
		self.ic1eq = 2.0f32 * v1 - self.ic1eq;
		self.ic2eq = 2.0f32 * v2 - self.ic2eq;

		self.m0 * v0 + self.m1 * v1 + self.m2 * v2
	}
}
