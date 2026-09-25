//! Tools for regular temperament theory.
//!
//! Just intervals are represented as integer vectors of exponents over the
//! basis of a just intonation [`Subgroup`], such as `2.3.7`. A regular
//! temperament is a linear map from such a vector space to a free abelian
//! group of lower rank; see [`Temperament`], which carries its subgroup
//! along with its mapping.

// This lint tends to reduce clarity for loops over matrices.
#![allow(clippy::needless_range_loop)]

pub mod error;
pub mod notation;
mod notation_options;
pub mod primes;
pub mod simplify;
pub mod temperament;
pub mod tuning;
mod util;

pub use diophantine::Matrix;
pub use error::Error;
pub use notation::Notation;
pub use primes::Subgroup;
pub use simplify::Simplifier;
pub use temperament::Temperament;
pub use tuning::Tuning;
pub use util::MAX_SEARCH_NODES;
