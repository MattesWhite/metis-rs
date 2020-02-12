//! Structs and types to serialize RDF.

pub mod config;
pub use self::config::*;

use crate::Format;
use sophia::term::TermData;
use std::io;

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
