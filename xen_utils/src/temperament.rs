//! Regular temperaments as integer linear maps.

use diophantine::{
    Matrix, eye, hnf, kernel_left, kernel_right, lll, saturation, solve_diophantine, transpose,
};

use crate::Error;
use crate::primes::Subgroup;
use crate::util::{column, first_column};

/// A regular temperament: a linear map from the interval vectors of a just
/// intonation subgroup to a free abelian group of lower rank.
///
/// Internally this is stored as a mapping matrix (rows are generators,
/// columns are basis elements of the subgroup), always reduced to Hermite
/// normal form. Two mapping matrices describe the same temperament exactly
/// when they have the same HNF.
///
/// A mapping whose row lattice is not primitive is refused, since it does not
/// describe a valid temperament: the rows must span a primitive sublattice of
/// the dual space, one equal to its own saturation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Temperament {
    mapping: Matrix<i64>,
    subgroup: Subgroup,
}

impl Temperament {
    /// Builds a temperament from a mapping matrix over `subgroup` (rows =
    /// generators, columns = basis elements).
    ///
    /// The mapping is reduced to Hermite normal form, so the result does not
    /// depend on which basis of generators the input used.
    ///
    /// # Errors
    /// Returns [`Error::Unsupported`] if the mapping's row lattice is not
    /// primitive, i.e. not equal to its saturation. Saturating it would silently
    /// answer for a different temperament than the one asked for; see the
    /// type documentation.
    pub fn from_mapping(mapping: &Matrix<i64>, subgroup: &Subgroup) -> Result<Self, Error> {
        if mapping.is_empty() {
            return Err(Error::InvalidDimensions("mapping must be non-empty".into()));
        }
        if mapping.iter().any(|row| row.len() != subgroup.dim()) {
            return Err(Error::InvalidDimensions(format!(
                "mapping rows must have {} entries, one per basis element of {subgroup}",
                subgroup.dim()
            )));
        }

        // The lattice is primitive when saturating it changes nothing.
        let canonical = hnf(mapping)?;
        if saturation(mapping)? != canonical {
            return Err(Error::Unsupported(format!(
                "mapping over {subgroup} is not primitive, and would saturate to a different temperament"
            )));
        }

        Ok(Temperament {
            mapping: canonical,
            subgroup: subgroup.clone(),
        })
    }

    /// Builds just intonation over `subgroup`: the temperament that tempers
    /// nothing out, whose mapping is the identity and whose rank is the rank of
    /// the subgroup.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if the identity is somehow not a
    /// valid mapping over `subgroup`.
    pub fn from_ji(subgroup: &Subgroup) -> Result<Self, Error> {
        Temperament::from_mapping(&eye(subgroup.dim()), subgroup)
    }

    /// Builds the temperament over `subgroup` obtained by tempering out the
    /// given commas.
    pub fn from_commas(commas: &[Vec<i64>], subgroup: &Subgroup) -> Result<Self, Error> {
        if commas.is_empty() {
            return Err(Error::InvalidDimensions("need at least one comma".into()));
        }
        if commas.iter().any(|c| c.len() != subgroup.dim()) {
            return Err(Error::InvalidDimensions(format!(
                "commas must have {} entries, one per basis element of {subgroup}",
                subgroup.dim()
            )));
        }

        // kernel_left wants the commas as columns.
        let comma_matrix = transpose(&commas.to_vec());
        let mapping = kernel_left(&comma_matrix)?;
        if mapping.is_empty() {
            return Err(Error::InvalidDimensions(
                "commas span the whole space; resulting temperament has rank 0".into(),
            ));
        }
        Temperament::from_mapping(&mapping, subgroup)
    }

    /// Builds the equal temperament of `steps` divisions of the octave over
    /// `subgroup`: each basis element `p` is mapped to the number of steps of
    /// `1/steps` octaves that best approximates it, found by scaling `log2(p)` by
    /// `steps` and rounding.
    pub fn equal(steps: i64, subgroup: &Subgroup) -> Result<Self, Error> {
        let map: Vec<i64> = subgroup
            .basis()
            .iter()
            .map(|&p| (steps as f64 * f64::from(p).log2()).round_ties_even() as i64)
            .collect();
        Temperament::from_mapping(&vec![map], subgroup)
    }

    /// The canonical mapping matrix (rows = generators, columns = basis
    /// elements of [`Self::subgroup`]).
    pub fn mapping(&self) -> &Matrix<i64> {
        &self.mapping
    }

    /// The just intonation subgroup being tempered.
    pub fn subgroup(&self) -> &Subgroup {
        &self.subgroup
    }

    /// The rank of the temperament (number of independent generators).
    pub fn rank(&self) -> usize {
        self.mapping.len()
    }

    /// The rank of the subgroup being tempered, i.e. the length of the
    /// interval vectors this temperament maps.
    pub fn dim(&self) -> usize {
        self.subgroup.dim()
    }

    /// A basis for the lattice of commas tempered out by this temperament.
    ///
    /// This is whatever basis falls out of the kernel computation and is
    /// usually not musically sensible on its own; see [`Self::reduced_comma_basis`].
    fn comma_basis(&self) -> Vec<Vec<i64>> {
        let columns = kernel_right(&self.mapping).expect("the mapping has a kernel");
        transpose(&columns)
    }

    /// A basis for the comma lattice, reduced via LLL to small, musically sensible commas.
    /// Each returned comma is normalized to be greater than unison.
    pub fn reduced_comma_basis(&self) -> Vec<Vec<i64>> {
        let commas = self.comma_basis();
        if commas.is_empty() {
            return commas;
        }
        let weights = self.subgroup.weights();
        let reduced = lll(&commas, 0.99, &weights).unwrap_or(commas);
        reduced
            .iter()
            .map(|comma| self.subgroup.ascending(comma))
            .collect()
    }

    /// Applies the mapping to each of `intervals`, one row per interval.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if any of them does not have one
    /// entry per basis element of the subgroup.
    pub fn temper_all(&self, intervals: &Matrix<i64>) -> Result<Matrix<i64>, Error> {
        intervals
            .iter()
            .map(|interval| self.temper(interval))
            .collect()
    }

    /// The tempered interval a just interval maps to: how many of each
    /// generator it is.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `interval` does not have one
    /// entry per basis element of the subgroup.
    pub fn temper(&self, interval: &[i64]) -> Result<Vec<i64>, Error> {
        if interval.len() != self.dim() {
            return Err(Error::InvalidDimensions(format!(
                "interval has {} entries, expected {}",
                interval.len(),
                self.dim()
            )));
        }
        Ok(self
            .mapping
            .iter()
            .map(|row| row.iter().zip(interval).map(|(a, b)| a * b).sum())
            .collect())
    }

    /// Some just interval that tempers to `tempered`: an arbitrary one of the
    /// many, not simplified. [`Simplifier`](crate::Simplifier) finds the
    /// simplest.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`] if `tempered` does not have one
    /// entry per generator.
    pub fn preimage(&self, tempered: &[i64]) -> Result<Vec<i64>, Error> {
        if tempered.len() != self.rank() {
            return Err(Error::InvalidDimensions(format!(
                "tempered interval has {} entries, expected {}",
                tempered.len(),
                self.rank()
            )));
        }
        let solution = solve_diophantine(&self.mapping, &column(tempered))?;
        Ok(first_column(&solution))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reduced comma basis as ratios, in the order LLL returns them: the
    /// Lovasz condition fixes that order, so it is part of what is being tested.
    fn comma_ratios(t: &Temperament) -> Vec<(u64, u64)> {
        t.reduced_comma_basis()
            .iter()
            .map(|comma| t.subgroup().to_ratio(comma).unwrap())
            .collect()
    }

    #[test]
    fn et_12et_5limit() {
        let s = Subgroup::p_limit(5);
        // 12et: 2 -> 12 steps, 3 -> 19 steps, 5 -> 28 steps.
        let t = Temperament::equal(12, &s).unwrap();
        assert_eq!(t.mapping(), &vec![vec![12, 19, 28]]);
    }

    #[test]
    fn et_over_subgroup() {
        // 2.3.7 is not a prime limit: 7 is mapped, 5 is not part of the group.
        let s: Subgroup = "2.3.7".parse().unwrap();
        let t = Temperament::equal(12, &s).unwrap();
        assert_eq!(t.mapping(), &vec![vec![12, 19, 34]]);
        assert_eq!(t.subgroup(), &s);
        assert_eq!(t.dim(), 3);
        assert_eq!(t.rank(), 1);
    }

    #[test]
    fn mapping_must_match_subgroup() {
        let s = Subgroup::p_limit(5);
        assert!(Temperament::from_mapping(&vec![vec![12, 19]], &s).is_err());
        assert!(Temperament::from_commas(&[vec![-4, 4]], &s).is_err());
    }

    #[test]
    fn reduced_commas_of_12et() {
        let s = Subgroup::p_limit(5);
        let t = Temperament::equal(12, &s).unwrap();
        let commas = t.reduced_comma_basis();
        // Rank 1 over a rank 3 group, so the comma lattice has rank 2.
        assert_eq!(commas.len(), 2);
        for comma in &commas {
            // Every returned comma is tempered out, and is an ascending interval.
            assert_eq!(t.temper(comma).unwrap(), vec![0]);
            assert!(s.to_cents(comma) > 0.0);
            // Reduction should find commas far smaller than an octave.
            assert!(s.to_cents(comma) < 100.0);
        }
    }

    #[test]
    fn reduced_commas_of_miracle() {
        let s = Subgroup::p_limit(11);
        let m31 = Temperament::equal(31, &s).unwrap();
        let m41 = Temperament::equal(41, &s).unwrap();
        assert_eq!(m31.mapping(), &vec![vec![31, 49, 72, 87, 107]]);
        assert_eq!(m41.mapping(), &vec![vec![41, 65, 95, 115, 142]]);

        // The join of 31et and 41et is 11-limit miracle: stacking the two maps
        // gives a rank 2 temperament tempering out everything both temper out.
        let mapping = vec![m31.mapping()[0].clone(), m41.mapping()[0].clone()];
        let miracle = Temperament::from_mapping(&mapping, &s).unwrap();
        assert_eq!(miracle.rank(), 2);

        // Rank 2 over a rank 5 group, so the comma lattice has rank 3.
        assert_eq!(
            comma_ratios(&miracle),
            vec![(225, 224), (385, 384), (441, 440)]
        );
    }

    #[test]
    fn meantone_from_comma() {
        let s = Subgroup::p_limit(5);
        // Tempering out 81/80 leaves rank 2 meantone.
        let t = Temperament::from_commas(&[vec![-4, 4, -1]], &s).unwrap();
        assert_eq!(t.rank(), 2);
        assert_eq!(t.temper(&[-4, 4, -1]).unwrap(), vec![0, 0]);
    }

    #[test]
    fn tempering_out_a_squared_comma_is_not_garbage() {
        // (81/80)^2 = 6561/6400 spans the same line as 81/80 itself, so
        // tempering it out should be ordinary meantone - not some artifact of
        // handing the kernel computation a non-primitive vector.
        let s = Subgroup::p_limit(5);
        let squared = Temperament::from_commas(&[vec![-8, 8, -2]], &s).unwrap();
        let meantone = Temperament::from_commas(&[vec![-4, 4, -1]], &s).unwrap();
        assert_eq!(squared, meantone);
    }

    #[test]
    fn a_mapping_that_is_not_primitive_is_refused() {
        // Every row is even, so the map only ever reaches even numbers: it is
        // not surjective onto its own image and describes no single
        // temperament. from_commas never produces this, since a kernel is
        // always primitive - only from_mapping and et need the check.
        let s = Subgroup::p_limit(5);
        assert!(Temperament::from_mapping(&vec![vec![2, 0, 2], vec![0, 2, 2]], &s).is_err());

        // 24et over 2.3.5 is the musical case: both primes land on twice an
        // odd number, so it saturates to 12et rather than describing itself.
        // 24et over 2.3.5.11 is what to ask for instead - its quartertone
        // makes the mapping primitive.
        assert!(Temperament::equal(24, &s).is_err());
        assert!(Temperament::equal(24, &"2.3.5.11".parse().unwrap()).is_ok());
    }
}
