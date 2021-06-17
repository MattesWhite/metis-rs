//! Data structures common to Notation3 and its derivatives.

pub mod prolog;
pub use self::prolog::*;
pub mod collections;

use crate::parse::turtle::terminals::*;
use sophia::term::{
    blank_node::BlankNode, iri::Iri, literal::Literal, mown_str::MownStr, TermData,
};

/// Marker trait for serialization formats.
///
/// # Size of formats
///
/// As this trait is meant to be used as a marker, implementors should be
/// unit-structs.
pub trait Format {
    /// Specific information added to `Config` in order to be able to serialize
    /// it.
    type ConfigData;

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
}

/// Type-level hack to provide a term with the suiting TermData.
pub trait Valid<TD: TermData>: Format {
    /// The concrete type which should has the correct TermData.
    type Term: RdfTerm<TD> + From<Iri<TD>> + From<Literal<TD>> + From<BlankNode<TD>>;
}

/// The term type for a given `Format` and `TermData`
pub type FormatTerm<F, TD> = <F as Valid<TD>>::Term;

/// The term for a given `Format` with `TermData` `MownStr<'a>`.
pub type MownTerm<'a, F> = FormatTerm<F, MownStr<'a>>;

/// Abstraction of an RDF term.
pub trait RdfTerm<TD: TermData>:
    std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash
{
}
