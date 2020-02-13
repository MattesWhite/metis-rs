//! Serialization in Turtle format.

use crate::Format;
use crate::common::{Valid, RdfTerm};
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

impl<TD: TermData + std::fmt::Debug> RdfTerm<TD> for Term<TD> {
    fn new_iri<U>(iri: U) -> Self 
    where
        TD: From<U>
    {
        Term::new_iri(iri).unwrap()
    }
    fn new_iri2<U, V>(ns: U, suffix: V) -> Self 
    where
        TD: From<U> + From<V>
    {
        Term::new_iri2(ns, suffix).unwrap()
    }
    fn new_blank_node<U>(label: U) -> Self 
    where
        TD: From<U>
    {
        Term::new_bnode(label).unwrap()
    }
    fn new_literal_dt<U>(txt: U, dt: Self) -> Self 
    where
        TD: From<U>
    {
        Term::new_literal_dt(txt, dt).unwrap()
    }
    fn new_literal_lang<U, L>(txt: U, lang: L) -> Self 
    where
        TD: From<U> + From<L>
    {
        Term::new_literal_lang(txt, lang).unwrap()
    }
}