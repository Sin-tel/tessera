//! Error types.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Error type for xen_utils operations.
pub enum Error {
    /// A vector or matrix did not have the expected dimensions.
    InvalidDimensions(String),
    /// A basis of primes did not describe a valid subgroup.
    InvalidSubgroup(String),
    /// A string was not a ratio.
    InvalidRatio(String),
    /// A rational number could not be expressed in the given subgroup basis.
    NotInSubgroup(String),
    /// A computed value did not fit in its integer type.
    Overflow(String),
    /// A valid request that this library cannot handle yet.
    Unsupported(String),
    /// An integer linear algebra operation failed.
    Diophantine(diophantine::DiophantineError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidDimensions(msg) => write!(f, "invalid dimensions: {msg}"),
            Error::InvalidSubgroup(msg) => write!(f, "invalid subgroup: {msg}"),
            Error::InvalidRatio(msg) => write!(f, "invalid ratio: {msg}"),
            Error::NotInSubgroup(msg) => write!(f, "not in subgroup: {msg}"),
            Error::Overflow(msg) => write!(f, "overflow: {msg}"),
            Error::Unsupported(msg) => write!(f, "unsupported: {msg}"),
            Error::Diophantine(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<diophantine::DiophantineError> for Error {
    fn from(e: diophantine::DiophantineError) -> Self {
        Error::Diophantine(e)
    }
}
