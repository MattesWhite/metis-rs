//! Structs and types to serialize RDF.

mod _error;
pub use self::_error::*;

pub mod config;
pub mod turtle;

use self::config::Config;
use self::turtle::regex::*;
use sophia::term::TermData;
use std::io;

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
    /// To check this Turtle's production rule `IRIREF` is used.
    #[inline]
    fn is_valid_ns<N>(ns: &N) -> bool
    where
        N: AsRef<str>,
    {
        IRIREF.is_match(ns.as_ref())
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

/// Allow the serialization of `Self`.
pub trait Serializable<F: Format> {
    /// Error raised at serialization. May differ from `io::Error`
    type Error: From<io::Error> + std::error::Error;

    /// Serialize the current state in the given `Format`.
    fn serialize<TD>(
        &self,
        target: &mut impl io::Write,
        conf: &Config<F, TD>,
    ) -> Result<(), Self::Error>
    where
        TD: TermData;
}

#[cfg(test)]
mod test {}
