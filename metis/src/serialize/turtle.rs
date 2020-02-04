//! Serialization in Turtle format.

pub mod regex;
pub mod stream;

pub use self::stream::Serializer as StreamSerializer;

use super::config::Config;
use super::{Format, Result, Serializable};
use sophia::term::{Term, TermData};
use std::io;

/// Type level representation of the [Turtle serialization](https://www.w3.org/TR/turtle/).
#[derive(Debug, Default, Clone, Copy)]
pub struct Turtle;

impl Format for Turtle {
    /// `Self` as no additional data is required.
    type ConfigData = Self;

    // validation functions are default impl.
}

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

impl Turtle {
    /// Write the preamble according to the `config` to the target.
    #[allow(clippy::useless_let_if_seq)]
    pub fn write_preamble<TD, T>(config: &Config<Self, TD>, target: &mut T) -> io::Result<()>
    where
        TD: TermData,
        T: io::Write,
    {
        // this is more clear than clippy's suggestion
        let mut wrote_something = false;

        if !config.prefixes.is_empty() {
            config
                .prefixes
                .iter()
                .map(|(p, ns)| writeln!(target, "@prefix {}: <{}> .", p.as_ref(), ns.as_ref()))
                .collect::<io::Result<Vec<()>>>()?;
            wrote_something = true;
        }

        if let Some(base) = &config.base {
            writeln!(target, "@base <{}> .", base.as_ref())?;
            wrote_something = true;
        }

        if wrote_something {
            writeln!(target)?;
        }

        Ok(())
    }
}
