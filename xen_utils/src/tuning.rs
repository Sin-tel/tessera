//! Tunings: sizes in cents for the generators of a temperament.

use diophantine::Matrix;

use crate::Error;
use crate::Temperament;
use crate::primes::Subgroup;

#[derive(Debug, Clone)]
pub struct Tuning {
    temperament: Temperament,
    generators: Vec<f64>,
}

impl Tuning {
    pub fn new(temperament: &Temperament, generators: Vec<f64>) -> Result<Self, Error> {
        if generators.len() != temperament.rank() {
            return Err(Error::InvalidDimensions(format!(
                "tuning has {} generators, expected {}",
                generators.len(),
                temperament.rank()
            )));
        }
        Ok(Tuning {
            temperament: temperament.clone(),
            generators,
        })
    }

    /// Least squares on the tuning map error `gM - J` under the Weil-Euclidean
    /// metric: `(M G M^T) g = M G J`.
    pub fn weil_euclidean(temperament: &Temperament) -> Self {
        let subgroup = temperament.subgroup();
        let metric = weil_euclidean(subgroup);
        let just: Vec<f64> = subgroup.log_primes().iter().map(|l| 1200.0 * l).collect();
        let mapping = temperament.mapping();

        // M G, rank x dim.
        let mg: Matrix<f64> = mapping
            .iter()
            .map(|row| {
                (0..subgroup.dim())
                    .map(|j| row.iter().zip(&metric).map(|(&m, g)| m as f64 * g[j]).sum())
                    .collect()
            })
            .collect();
        let normal: Matrix<f64> = mg
            .iter()
            .map(|a| {
                mapping
                    .iter()
                    .map(|row| a.iter().zip(row).map(|(x, &m)| x * m as f64).sum())
                    .collect()
            })
            .collect();
        let rhs: Vec<f64> = mg.iter().map(|a| dot(a, &just)).collect();

        // The mapping has full row rank and the metric is positive definite,
        // so the normal equations are too.
        let generators = solve(normal, rhs).expect("the normal equations are positive definite");
        Tuning::new(temperament, generators).expect("one generator per rank")
    }

    pub fn temperament(&self) -> &Temperament {
        &self.temperament
    }

    /// Cents per generator of the temperament.
    pub fn generators(&self) -> &[f64] {
        &self.generators
    }

    pub fn pitch(&self, tempered: &[i64]) -> Result<f64, Error> {
        if tempered.len() != self.generators.len() {
            return Err(Error::InvalidDimensions(format!(
                "tempered interval has {} entries, expected {}",
                tempered.len(),
                self.generators.len()
            )));
        }
        Ok(tempered
            .iter()
            .zip(&self.generators)
            .map(|(&t, g)| t as f64 * g)
            .sum())
    }

    pub fn pitch_interval(&self, interval: &[i64]) -> Result<f64, Error> {
        self.pitch(&self.temperament.temper(interval)?)
    }
}

/// The Weil-Euclidean metric on tuning maps: the inverse of `B^T B`, `B` the
/// log primes stacked on their sum. By Sherman-Morrison that is the Tenney
/// metric less a rank one update, `diag(l^2) - l l^T / (n + 1)` with `l` the
/// inverse log primes.
pub fn weil_euclidean(subgroup: &Subgroup) -> Matrix<f64> {
    let n = subgroup.dim();
    let inverse: Vec<f64> = subgroup.log_primes().iter().map(|l| 1.0 / l).collect();
    (0..n)
        .map(|i| {
            (0..n)
                .map(|j| {
                    let tenney = if i == j { inverse[i] * inverse[i] } else { 0.0 };
                    tenney - inverse[i] * inverse[j] / (n as f64 + 1.0)
                })
                .collect()
        })
        .collect()
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Gaussian elimination with partial pivoting. `None` if `a` is singular.
fn solve(mut a: Matrix<f64>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let pivot = (col..n).max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))?;
        if a[pivot][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in col + 1..n {
            let factor = a[row][col] / a[col][col];
            for k in col..n {
                a[row][k] -= factor * a[col][k];
            }
            b[row] -= factor * b[col];
        }
    }
    let mut x = vec![0.0; n];
    for row in (0..n).rev() {
        let rest: f64 = (row + 1..n).map(|k| a[row][k] * x[k]).sum();
        x[row] = (b[row] - rest) / a[row][row];
    }
    Some(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn closed_form_is_the_inverse() {
        // The definition: invert (B diag(j))^T (B diag(j)), B = [I; 1].
        let s = Subgroup::p_limit(13);
        let n = s.dim();
        let j = s.log_primes();
        let gd: Matrix<f64> = (0..n)
            .map(|r| {
                (0..n)
                    .map(|c| j[r] * j[c] + if r == c { j[r] * j[r] } else { 0.0 })
                    .collect()
            })
            .collect();
        let g = weil_euclidean(&s);
        for c in 0..n {
            let unit: Vec<f64> = (0..n).map(|r| if r == c { 1.0 } else { 0.0 }).collect();
            let column = solve(gd.clone(), unit).unwrap();
            for r in 0..n {
                assert!(close(column[r], g[r][c]), "{r} {c}");
            }
        }
    }

    #[test]
    fn just_intonation_is_just() {
        let s = Subgroup::p_limit(11);
        let t = Tuning::weil_euclidean(&Temperament::from_ji(&s).unwrap());
        for (g, l) in t.generators().iter().zip(s.log_primes()) {
            assert!(close(*g, 1200.0 * l));
        }
    }

    #[test]
    fn meantone_tempers_out_the_syntonic_comma() {
        let s = Subgroup::p_limit(5);
        let t = Temperament::from_commas(&[vec![-4, 4, -1]], &s).unwrap();
        let tuning = Tuning::weil_euclidean(&t);
        assert!(close(tuning.pitch_interval(&[-4, 4, -1]).unwrap(), 0.0));
        let fifth = tuning.pitch_interval(&[-1, 1, 0]).unwrap();
        assert!(fifth > 695.0 && fifth < 698.0, "{fifth}");
    }
}
