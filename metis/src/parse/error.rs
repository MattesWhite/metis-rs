//! Custom errors for parsing N3 and its derivative formats.

use nom::error::{ErrorKind, ParseError};
use nom::{Err as NErr, IResult};
use sophia_term::TermError;
use std::fmt;

/// Maximal length of context given for errors.
pub const MAX_CTX_LEN: usize = 48;

/// Errors raised at parsing.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The text {0} is not a valid Prefix")]
    InvalidPrefix(String),
    /// Error from `sophia`.
    #[error("{0}")]
    Term(#[from] TermError),
    /// Error from a `nom` parser.
    #[error("Parser failed: {:?}", 0)]
    Kind(ErrorKind),
    /// Returned if no valid rule matches.
    #[error("No parser rule matched")]
    NoMatch,
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::Kind(kind)
    }
}

/// An error with information where it occurred.
#[derive(Debug, thiserror::Error)]
pub struct PosError<'a>(&'a str, Error);

impl<'a> PosError<'a> {
    pub fn new(i: &'a str, err: impl Into<Error>) -> Self {
        PosError(i, err.into())
    }
    pub fn err(i: &'a str, err: impl Into<Error>) -> NErr<Self> {
        NErr::Error(Self::new(i, err))
    }
    pub fn failed(i: &'a str, err: impl Into<Error>) -> NErr<Self> {
        NErr::Failure(Self::new(i, err))
    }
}

impl<'a> fmt::Display for PosError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error at: ")?;
        if self.0.len() > MAX_CTX_LEN {
            write!(f, "{}...", &self.0[..MAX_CTX_LEN])
        } else {
            write!(f, "{}", self.0)
        }?;
        write!(f, " => {}", self.1)
    }
}

impl<'a> ParseError<&'a str> for PosError<'a> {
    fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
        PosError(input, kind.into())
    }
    fn append(input: &'a str, kind: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> From<(&'a str, ErrorKind)> for PosError<'a> {
    /// This allows to use `?` on `IResult` in functions returning `PResult`.
    fn from((i, kind): (&'a str, ErrorKind)) -> Self {
        Self::new(i, kind)
    }
}

/// Parser result using own error type.
pub type PResult<'a, O> = IResult<&'a str, O, PosError<'a>>;

/// Extension trait for `IResult`.
pub trait MapPR<'a, O> {
    fn map_pr(self) -> PResult<'a, O>;
}

impl<'a, O> MapPR<'a, O> for IResult<&'a str, O> {
    fn map_pr(self) -> PResult<'a, O> {
        self.map_err(|nerr| match nerr {
            NErr::Error((i, kind)) => NErr::Error(PosError::from_error_kind(i, kind)),
            NErr::Failure((i, kind)) => NErr::Failure(PosError::from_error_kind(i, kind)),
            _ => unimplemented!(),
        })
    }
}

/// Extension trait for `std::result::Result`.
pub trait IntoPR<'a, O> {
    /// Build a `PResult` with a given context.
    fn into_pr(self, before: &'a str, after: &'a str) -> PResult<'a, O>;
}

impl<'a, T, E> IntoPR<'a, T> for Result<T, E>
where
    E: Into<Error>,
{
    /// This implementation returns in the error-case a `nom::Err::Failed(_)`
    /// as validation usually means an invalid document.
    fn into_pr(self, before: &'a str, after: &'a str) -> PResult<'a, T> {
        self.map(|t| (after, t)).map_err(|err| PosError::failed(before, err))
    }
}

/// Extension trait for `Option`.
pub trait OrIntoPR<'a, O> {
    /// Build a `PResult` with a given context.
    fn or_into_pr(self, before: &'a str, err: Error, after: &'a str) -> PResult<'a, O>;
}

impl<'a, T> OrIntoPR<'a, T> for Option<T> {
    /// This implementation returns in the error-case a `nom::Err::Failed(_)`
    /// as validation usually means an invalid document.
    fn or_into_pr(self, before: &'a str, err: Error, after: &'a str) -> PResult<'a, T> {
        self.map(|t| (after, t)).ok_or_else(|| PosError::failed(before, err))
    }
}
