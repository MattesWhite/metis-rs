//! Error handling.

use sophia::term::TermError;
use std::io;

/// Type alias for `Result` with default `Error`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that are raised by this crate. Probably will be split later into the
/// modules.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The given IRI is invalid.
    #[error("The text {0} is not a valid IRI")]
    InvalidIri(String),
    /// The given term is not an IRI.
    #[error("The given term is not an IRI")]
    InvalidBase,
    /// The given prefix is invalid.
    #[error("The text {0} is not a valid Prefix")]
    InvalidPrefix(String),
    /// The requested indentation is to wide.
    #[error(
        "Requested to much spaces ({0}) to indent (max is {})",
        crate::serialize::config::MAX_SPACES
    )]
    ToMuchSpaces(u8),
    /// The defined spacing is to small.
    #[error("Spacing must be at least one space")]
    InvalidSpacing,
    /// Error from writing to target.
    #[error("Target error: {0}")]
    FromIo(#[from] io::Error),
    /// Error from parsing.
    #[error("Parser error: {0}")]
    Parser(String),
    /// Error from sophia.
    #[error("{0}")]
    Term(#[from] TermError),
}
