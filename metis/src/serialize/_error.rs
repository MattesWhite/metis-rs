//! Serialization errors.

use std::io;

/// Type alias for `Result` with default `Error`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A general serialization error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The given IRI is invalid.
    #[error("The text {0} is not a valid IRI")]
    InvalidIri(String),
    /// The given prefix is invalid.
    #[error("The text {0} is not a valid Prefix")]
    InvalidPrefix(String),
    /// The requested indentation is to wide.
    #[error(
        "Requested to much spaces ({0}) to indent (max is {})",
        super::config::MAX_SPACES
    )]
    ToMuchSpaces(u8),
    /// The defined spacing is to small.
    #[error("Spacing must be at least one space")]
    InvalidSpacing,
    /// Error from writing to target.
    #[error("Target error: {0}")]
    FromIo(#[from] io::Error),
}
