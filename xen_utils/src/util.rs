//! Integer vector helpers.
//!
//! Nothing here knows anything about music. These are the operations the
//! derivation reaches for again and again, named so that the code using them
//! reads as what it is doing rather than as index arithmetic.

use diophantine::Matrix;

/// The Lovasz condition for every lattice reduction here, the usual `0.99`.
pub const LLL_DELTA: f64 = 0.99;

/// How many nodes either lattice search may enumerate before it gives up and
/// answers with the best it has found.
pub const MAX_SEARCH_NODES: u64 = 100_000;

/// The combination of `generators` given by `counts`, as a vector of `dim`
/// entries.
pub fn combination(counts: &[i64], generators: &Matrix<i64>, dim: usize) -> Vec<i64> {
    (0..dim)
        .map(|entry| {
            counts
                .iter()
                .zip(generators)
                .map(|(count, generator)| count * generator[entry])
                .sum()
        })
        .collect()
}

/// `one` less `other`, entry by entry.
pub fn subtract(one: &[i64], other: &[i64]) -> Vec<i64> {
    one.iter().zip(other).map(|(a, b)| a - b).collect()
}

/// Whether every entry of `vector` is zero.
pub fn is_zero(vector: &[i64]) -> bool {
    vector.iter().all(|&entry| entry == 0)
}

/// The rows of `matrix` at `indices`, in the order the indices are given.
pub fn select(matrix: &Matrix<i64>, indices: &[usize]) -> Matrix<i64> {
    indices.iter().map(|&index| matrix[index].clone()).collect()
}

/// `vector` as a one column matrix, the shape the diophantine solvers take a
/// right hand side in.
pub fn column(vector: &[i64]) -> Matrix<i64> {
    vector.iter().map(|&entry| vec![entry]).collect()
}

/// The first column of `matrix`, undoing [`column`].
pub fn first_column(matrix: &Matrix<i64>) -> Vec<i64> {
    matrix.iter().map(|row| row[0]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combinations_and_subtraction() {
        let generators = vec![vec![1, 0, 0], vec![-1, 1, 0]];
        // Two octaves and three fifths.
        assert_eq!(combination(&[2, 3], &generators, 3), vec![-1, 3, 0]);
        assert_eq!(combination(&[0, 0], &generators, 3), vec![0, 0, 0]);
        assert_eq!(subtract(&[1, 2, 3], &[1, 0, -3]), vec![0, 2, 6]);
    }

    #[test]
    fn selecting_and_reshaping() {
        let matrix = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        assert_eq!(select(&matrix, &[2, 0]), vec![vec![5, 6], vec![1, 2]]);
        assert_eq!(select(&matrix, &[]), Vec::<Vec<i64>>::new());

        assert_eq!(column(&[7, 8]), vec![vec![7], vec![8]]);
        assert_eq!(first_column(&matrix), vec![1, 3, 5]);
        assert_eq!(first_column(&column(&[7, 8])), vec![7, 8]);

        assert!(is_zero(&[0, 0]));
        assert!(is_zero(&[]));
        assert!(!is_zero(&[0, -1]));
    }
}
