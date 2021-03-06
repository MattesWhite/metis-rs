//! A Turtle parser.
//!
//! # Performance
//!
//! For now this is a straight forward implementation. No efforts were taken
//! regarding performance.
//!
//! # Completeness
//!
//! While aiming at feature-completeness later, for now some basic Turtle
//! features are (knowingly) missing:
//!
//! - Functionality of base IRI.
//! - Correct resolving of relative IRIs.
//! - Escape characters (ECHAR and UCHAR).
//!
//! In the future it is planned to execute the parser against the W3C test
//! suite where probably further missing features will pop up.
//!

pub mod production;
pub mod terminals;

#[cfg(test)]
mod test_suite;

use self::production::statement;
use self::terminals::multispace0;
use crate::error::{Error, Result};
use crate::parse::Context;
use crate::Turtle;
use sophia::term::{iri::Iri, mown_str::MownStr, Term};

/// Shortcut for `Term<MownStr<'doc>>`.
pub type MownTerm<'doc> = Term<MownStr<'doc>>;

/// The Turtle parser that parses a document step by step.
pub struct Parser<'doc> {
    /// Gathered metadata.
    ctx: Context<'doc, Turtle>,
    /// Current position within the document.
    current: &'doc str,
    /// true if the parser failed once or if it is at EOF.
    ///
    /// In both cases the `next() = None`.
    end_or_failed: bool,
}

impl<'doc> Parser<'doc> {
    /// Creates a new Parser.
    pub fn new(doc: &'doc str) -> Self {
        // trim leading whitespaces
        let (doc, _) = multispace0(doc).unwrap();
        Self {
            ctx: Context::default(),
            current: doc,
            end_or_failed: false,
        }
    }
    /// A new parser with a pre-set base IRI to resolve `iri` productions.
    ///
    /// _Note:_ If the document contains an own `base` directive the pre-set
    /// value is overridden.
    pub fn with_base(doc: &'doc str, base: Iri<MownStr<'doc>>) -> Result<Self> {
        let mut ctx = Context::default();
        ctx.prolog.set_base(base)?;

        // trim leading whitespaces
        let (doc, _) = multispace0(doc).unwrap();
        Ok(Self {
            ctx,
            current: doc,
            end_or_failed: false,
        })
    }
}

impl<'doc> Iterator for Parser<'doc> {
    type Item = Result<[MownTerm<'doc>; 3]>;

    /// Returns parsed triples.
    ///
    /// The parsing is done statement per statement. The parsed triples from
    /// a statement are stored internally. When all triples of a parsed
    /// statement are returned the next statement is parsed.
    fn next(&mut self) -> Option<Self::Item> {
        if self.end_or_failed {
            // parser finished
            return None;
        } else if let Some(tri) = self.ctx.pop_triple() {
            // triples are left from the last parsing
            return Some(Ok(tri));
        } else if self.current.is_empty() {
            // parser has finished but has it not yet recognized
            self.end_or_failed = true;
            return None;
        }

        // parse new triples
        let step = statement(&self.current, &self.ctx);
        let rest = match step {
            Ok((rest, _)) => rest,
            Err(e) => {
                self.end_or_failed = true;
                return Some(Err(Error::Parser(e.to_string())));
            }
        };
        // multispace0 never fails
        let (rest, _) = multispace0(rest).unwrap();
        self.current = rest;

        self.next()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sophia::serializer::{nt, Stringifier, TripleSerializer};
    use sophia::triple::stream::TripleSource;

    #[test]
    fn parse_example() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let example = r#"   # initial comment
        @prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#>.
        @prefix sosa: <http://www.w3.org/ns/sosa/> .
        @prefix ssn: <http://www.w3.org/ns/ssn/> .
        @prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
        @prefix qudt-1-1: <http://qudt.org/1.1/schema/qudt#> .
        @prefix qudt-unit-1-1: <http://qudt.org/1.1/vocab/unit#> .
        @base <http://example.org/data/> .
        
        # rangefinder #30 is a laser range finder sensor that was used 
        # to observe the height of tree #124 and #125.
        
        <rangefinder/30>        rdf:type           sosa:Sensor ;
          rdfs:label "rangefinder #30"@en ;
          rdfs:comment "rangefinder #30 is a laser range finder sensor."@en .
        
        # rangefinder #30 made observation #1087 of the height of tree #124.
        
        <observation/1087>  rdf:type               sosa:Observation ;
          rdfs:label "observation #1087"@en ;
          sosa:hasFeatureOfInterest  <tree/124> ;
          sosa:observedProperty  <tree/124#height> ;
          sosa:madeBySensor <rangefinder/30> ;
          sosa:hasResult [ 
            qudt-1-1:unit qudt-unit-1-1:Meter ; 
            qudt-1-1:numericalValue "15.3"^^xsd:double ] .
        
        <tree/124>  rdf:type         sosa:FeatureOfInterest ;
          rdfs:label "tree #124"@en .
        
        <tree/124#height>  rdf:type    sosa:ObservableProperty , ssn:Property ;
          rdfs:label "the height of tree #124"@en .
        
        # rangefinder #30 made observation #1088 of the height of tree #125.
        
        <observation/1088>  rdf:type               sosa:Observation ;
          rdfs:label "observation #1088"@en ;
          sosa:hasFeatureOfInterest  <tree/125> ;
          sosa:observedProperty  <tree/125/height> ;
          sosa:madeBySensor <rangefinder/30> ;
          sosa:hasResult [ 
            qudt-1-1:numericValue "23.0"^^xsd:double ;
            qudt-1-1:unit qudt-unit-1-1:Meter ] .
        
        <tree/125>  rdf:type    sosa:FeatureOfInterest ;
          rdfs:label "tree #125"@en .
        
        <tree/125#height>  rdf:type    sosa:ObservableProperty , ssn:Property ;
          rdfs:label "the height of tree #125"@en .
        "#;

        let mut parser = Parser::new(example);
        let mut g = vec![];

        parser.in_graph(&mut g)?;
        let s = nt::NtSerializer::new_stringifier()
            .serialize_graph(&g)?
            .to_string();
        println!("Serialized: \n\n {}", s);
        Ok(())
    }
}
