//! Implementation of a streaming serializer for Turtle.

use super::Turtle;
use crate::error::{Error, Result};
use crate::serialize::Config;
use crate::serialize::Serializable;
use sophia::term::{Term, TermData};
use sophia::triple::{
    stream::{SinkError, SourceError, StreamError},
    Triple,
};
use std::io;

/// Serializer for streams of RDF triples.
pub struct Serializer<'a, T, TD>
where
    T: io::Write,
    TD: TermData,
{
    target: T,
    config: &'a Config<Turtle, TD>,
    indent_level: u32,
}

impl<'a, T, TD> Serializer<'a, T, TD>
where
    T: io::Write,
    TD: TermData,
{
    /// Create a new serializer for the given target with the given
    /// configuration.
    ///
    /// The preamble for `config` is written immediately.
    pub fn new(target: T, config: &'a Config<Turtle, TD>) -> io::Result<Self> {
        let mut target = target;
        Turtle::write_preamble(config, &mut target)?;

        Ok(Self {
            target,
            config,
            indent_level: 0,
        })
    }
    /// Serialize a triple source to the serializer's target.
    pub fn serialize<TS, Tri, E>(&mut self, ts: TS) -> Result<(), StreamError<E, Error>>
    where
        TS: Iterator<Item = Result<Tri, E>>,
        E: std::error::Error,
        Tri: Triple,
        Term<Tri::TermData>: Serializable<Turtle>,
        Error: From<<Term<Tri::TermData> as Serializable<Turtle>>::Error>,
    {
        let mut last_tri: Option<Tri> = None;

        for tri in ts {
            let tri = tri.map_err(SourceError)?;

            match last_tri {
                None => self.write_spo(&tri).map_err(SinkError)?,
                Some(lt) if lt.s() != tri.s() => {
                    self.finish_block().map_err(SinkError)?;
                    self.write_spo(&tri).map_err(SinkError)?;
                }
                Some(lt) if lt.s() == tri.s() && lt.p() != tri.p() => {
                    self.write_po(&tri).map_err(SinkError)?;
                }
                Some(lt) if lt.s() == tri.s() && lt.p() == tri.p() => {
                    self.write_o(&tri).map_err(SinkError)?;
                }
                _ => unreachable!(),
            };

            last_tri = Some(tri);
        }

        Ok(())
    }
    /// Drop the serializer and get the target back.
    pub fn finish(self) -> Result<T> {
        let mut ser = self;
        ser.finish_block()?;
        Ok(ser.target)
    }
    /// Write the indention according to the current level.
    fn indent(&mut self) -> io::Result<()> {
        for _ in 0..self.indent_level {
            self.config.write_indent(&mut self.target)?;
        }

        Ok(())
    }
    /// Write the whole triple.
    fn write_spo(&mut self, tri: &impl Triple) -> Result<(), Error> {
        let target = &mut self.target;
        tri.s().serialize(target, &self.config)?;
        self.config.write_space(target)?;
        tri.p().serialize(target, &self.config)?;
        self.config.write_space(target)?;
        tri.o().serialize(target, &self.config)?;
        self.indent_level += 1;
        Ok(())
    }
    /// Write predicate and object into the next line.
    fn write_po(&mut self, tri: &impl Triple) -> Result<(), Error> {
        self.target.write_all(b" ;\n")?;
        self.indent()?;

        let target = &mut self.target;
        tri.p().serialize(target, &self.config)?;
        self.config.write_space(target)?;
        tri.o().serialize(target, &self.config)?;
        Ok(())
    }
    /// Append the object to the list.
    fn write_o(&mut self, tri: &impl Triple) -> Result<(), Error> {
        let target = &mut self.target;
        target.write_all(b",")?;
        self.config.write_space(target)?;
        tri.o().serialize(target, &self.config)?;
        Ok(())
    }
    /// Finish a block and insert a blank line.
    fn finish_block(&mut self) -> Result<(), Error> {
        self.target.write_all(b" .\n\n")?;
        self.indent_level = 0;
        Ok(())
    }
}

// TODO: Find way to test serialization.
// #[cfg(test)]
// mod test {
//     use super::*;
//     use sophia::graph::inmem::FastGraph;
//     use sophia::graph::Graph;
//     use sophia::parser::nt::*;
//     use sophia::parser::*;
//     use sophia::triple::stream::TripleSource;

//     #[test]
//     fn check() -> std::result::Result<(), Box<dyn std::error::Error>> {
//         let nt = r#"
// 			_:s1 <http://example.org/p1> _:o1 .
// 			_:s1 <http://example.org/p2> _:o2 .
// 			_:s1 <http://example.org/p2> _:o3 .
// 			_:s2 <http://example.org/p1> _:o1 .
// 			_:s2 <http://example.org/p2> _:o2 .
// 			_:s3 <http://example.org/p2> _:o3 .
//         "#;

//         let mut g = FastGraph::new();
//         let p = NTriplesParser {};
//         let c = p.parse_str(&nt).in_graph(&mut g)?;
//         assert_eq!(c, 6);

//         let mut config = Config::<Turtle, &'static str>::default();
//         config
//             .add_default_prefixes()
//             .set_base("http://example.org/")?;
//         let mut ser = Serializer::new(vec![], &config)?;
//         ser.serialize(g.triples())?;

//         let turtle = String::from_utf8(ser.finish()?)?;
//         println!("serialized:\n{}", turtle);
//         assert_eq!(&turtle, "_:a1 _:a2");
//         Ok(())
//     }
// }
