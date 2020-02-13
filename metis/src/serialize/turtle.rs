//! Serialize the turtle format.

mod _stream;
pub use self::_stream::*;

use crate::Turtle;
use crate::serialize::{Config, Serializable};
use crate::common::Prolog;
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


impl<TD: TermData> Prolog<Turtle, TD> {
    /// Write the preamble according to the `prolog` to the target.
    #[allow(clippy::useless_let_if_seq)]
    pub fn write_preamble<T>(&self, target: &mut T) -> io::Result<()>
    where
        T: io::Write,
    {
        // this is more clear than clippy's suggestion
        let mut wrote_something = false;

        if !self.prefixes.is_empty() {
            self
                .prefixes
                .iter()
                .map(|(p, ns)| writeln!(target, "@prefix {}: <{}> .", p.as_ref(), ns.as_ref()))
                .collect::<io::Result<Vec<()>>>()?;
            wrote_something = true;
        }

        if let Some(base) = &self.base {
            writeln!(target, "@base <{}> .", base.as_ref())?;
            wrote_something = true;
        }

        if wrote_something {
            writeln!(target)?;
        }

        Ok(())
    }
}
