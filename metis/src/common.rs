//! Data structures common to Notation3 and its derivates.

pub mod prolog;
pub use self::prolog::*;

use crate::error::{Error, Result};
use crate::turtle::parse::terminals::*;

/// Marker trait for serialization formats.
///
/// # Size of formats
///
/// As this trait is meant to be used as a marker, implementers should be
/// unit-structs.
pub trait Format {
    /// Specific information added to `Config` in order to be able to serialize
    /// it.
    type ConfigData;

    /// Checks if a given value is valid IRI to be used as namespace.
    ///
    /// # Default impl
    ///
    /// To check this the production rule `IRIREF_ONLY` is used. This is euqal to
    /// Turtle's `IRIREF` except that the angled brackets must not be included
    /// and `ns` must only contain the IRI.
    #[inline]
    fn is_valid_ns<N>(ns: &N) -> bool
    where
        N: AsRef<str>,
    {
        IRIREF_ONLY.is_match(ns.as_ref())
    }
    /// Checks if a given value is valid prefix.
    ///
    /// # Default impl
    ///
    /// To check this Turtle's production rule `PN_PREFIX` is used.
    #[inline]
    fn is_valid_prefix<P>(p: &P) -> bool
    where
        P: AsRef<str>,
    {
        let p = p.as_ref();
        p.is_empty() || PN_PREFIX.is_match(p)
    }
    /// Checks if both prefix and namespace are valid.
    ///
    /// Uses [`Self::is_valid_ns()`](#method.is_valid_ns) and
    /// [`Self::is_valid_prefix()`](#method.is_valid_prefix).
    #[inline]
    fn check_prefix_id<P, N>(p: P, ns: N) -> Result<(P, N)>
    where
        P: AsRef<str>,
        N: AsRef<str>,
    {
        match (p, ns) {
            (p, _) if !Self::is_valid_prefix(&p.as_ref()) => {
                Err(Error::InvalidPrefix(p.as_ref().to_owned()))
            }
            (_, ns) if !Self::is_valid_ns(&ns.as_ref()) => {
                Err(Error::InvalidIri(ns.as_ref().to_owned()))
            }
            (p, ns) => Ok((p, ns)),
        }
    }
}
