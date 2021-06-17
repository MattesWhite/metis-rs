//! Serialization in Turtle format.

use crate::common::{RdfTerm, Valid};
use crate::Format;
use sophia::term::{Term, TermData};

/// Type level representation of the [Turtle serialization](https://www.w3.org/TR/turtle/).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Turtle;

impl Format for Turtle {
    /// `Self` as no additional data is required.
    type ConfigData = Self;

    // validation functions are default impl.
}

impl<TD: TermData + std::fmt::Debug> Valid<TD> for Turtle {
    type Term = Term<TD>;
}

impl<TD: TermData + std::fmt::Debug> RdfTerm<TD> for Term<TD> {}
