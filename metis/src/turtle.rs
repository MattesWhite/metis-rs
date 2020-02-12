//! Serialization in Turtle format.

pub mod parse;
pub mod serialize;

use crate::common::Prolog;
use crate::Format;
use sophia::term::TermData;
use std::io;

/// Type level representation of the [Turtle serialization](https://www.w3.org/TR/turtle/).
#[derive(Debug, Default, Clone, Copy)]
pub struct Turtle;

impl Format for Turtle {
    /// `Self` as no additional data is required.
    type ConfigData = Self;

    // validation functions are default impl.
}

impl Turtle {
    /// Write the preamble according to the `prolog` to the target.
    #[allow(clippy::useless_let_if_seq)]
    pub fn write_preamble<TD, T>(prolog: &Prolog<Self, TD>, target: &mut T) -> io::Result<()>
    where
        TD: TermData,
        T: io::Write,
    {
        // this is more clear than clippy's suggestion
        let mut wrote_something = false;

        if !prolog.prefixes.is_empty() {
            prolog
                .prefixes
                .iter()
                .map(|(p, ns)| writeln!(target, "@prefix {}: <{}> .", p.as_ref(), ns.as_ref()))
                .collect::<io::Result<Vec<()>>>()?;
            wrote_something = true;
        }

        if let Some(base) = &prolog.base {
            writeln!(target, "@base <{}> .", base.as_ref())?;
            wrote_something = true;
        }

        if wrote_something {
            writeln!(target)?;
        }

        Ok(())
    }
}
