//! Serialize the turtle format.

mod _stream;
pub use self::_stream::*;

use super::Turtle;
use crate::serialize::{Config, Serializable};
use sophia::term::{Term, TermData};
use std::io;

impl<TD> Serializable<Turtle> for Term<TD>
where
    TD: TermData,
{
    type Error = io::Error;

    fn serialize<U>(
        &self,
        target: &mut impl io::Write,
        _: &Config<Turtle, U>,
    ) -> Result<(), Self::Error>
    where
        U: TermData,
    {
        match self {
            Term::Iri(iri) => {
                // unimplemented!("Needs progress in `sophia` (term.match_ns())")
                write!(target, "<{}>", iri)?
            }
            Term::Literal(txt, _) => {
                // unimplemented!("Needs progress in `sophia`")
                write!(target, "\"{}\"", txt.as_ref())?
            }
            Term::BNode(label) => write!(target, "_:{}", label.as_ref())?,
            Term::Variable(name) => write!(target, "?{}", name.as_ref())?,
        };

        Ok(())
    }
}
