//! A Notation3 parser.
//!
//! # Performance
//!
//! For now this is a straight forward implementation. No efforts were taken
//! regarding performance.
//!
//! # Completeness
//!
//! See the module level description of [N3](../../n3/index.html).
//!

pub mod production;
pub mod terminals;

use super::turtle::terminals as ttl_terminal;
use crate::error::Result;
use crate::parse::Context;
use crate::CowTerm as FormatCowTerm;
use crate::N3;
use std::borrow::Cow;
use std::cell::RefCell;

/// Shortcut for `Term<Cow<'a, str>>`.
pub type CowTerm<'a> = FormatCowTerm<'a, N3>;

/// The Turtle parser that parses a document step by step.
pub struct Parser<'a> {
    /// Gathered metadata.
    ctx: RefCell<Context<'a, N3>>,
    /// Current position within the document.
    current: &'a str,
    /// true if the parser failed once of is at EOF.
    ///
    /// In both cases the `next() = None`.
    end_or_failed: bool,
}

impl<'a> Parser<'a> {
    /// Creates a new Parser.
    pub fn new(doc: &'a str) -> Self {
        // trim leading whitespaces
        let (doc, _) = ttl_terminal::multispace0(doc).unwrap();
        Self {
            ctx: RefCell::new(Context::default()),
            current: doc,
            end_or_failed: false,
        }
    }
    /// A new parser with a pre-set base IRI to resolve `iri` productions.
    ///
    /// _Note:_ If the document contains an own `base` directive the pre-set
    /// value is overridden.
    pub fn with_base(doc: &'a str, base: impl Into<Cow<'a, str>>) -> Result<Self> {
        let mut ctx = Context::default();
        ctx.prolog.set_base(base.into())?;

        // trim leading whitespaces
        let (doc, _) = ttl_terminal::multispace0(doc).unwrap();
        Ok(Self {
            ctx: RefCell::new(ctx),
            current: doc,
            end_or_failed: false,
        })
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<[CowTerm<'a>; 3]>;

    /// Returns parsed triples.
    ///
    /// The parsing is done statement per statement. The parsed triples from
    /// a statement are stored internally. When all triples of a parsed
    /// statement are returned the next statement is parsed.
    fn next(&mut self) -> Option<Self::Item> {
        None
        // if self.end_or_failed {
        //     // parser finished
        //     return None;
        // } else if let Some(tri) = self.ctx.borrow_mut().pop_triple() {
        //     // triples are left from the last parsing
        //     return Some(Ok(tri));
        // } else if self.current.is_empty() {
        //     // parser has finished but has it not yet recognized
        //     self.end_or_failed = true;
        //     return None;
        // }

        // // parse new triples
        // let step = statement(&self.current, &self.ctx);
        // let rest = match step {
        //     Ok((rest, _)) => rest,
        //     Err(e) => {
        //         self.end_or_failed = true;
        //         return Some(Err(Error::Parser(e.to_string())));
        //     }
        // };
        // // multispace0 never fails
        // let (rest, _) = multispace0(rest).unwrap();
        // self.current = rest;

        // self.next()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_turtle() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

        let g = Parser::new(example).collect::<Result<Vec<[CowTerm<'_>; 3]>, _>>()?;

        println!("Serialized: \n\n {:#?}", g);
        Ok(())
    }

    #[test]
    fn parse_n3() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let example = r#"   # initial comment
        @prefix : <#> .
        @prefix math: <http://www.w3.org/2000/10/swap/math#> .

        :config :iterations "10" .
        :ix :input "28561" . # 169^2

        # calculate seed
        {
            ?id :input ?x .
            (?x "0.6") math:product ?y .
        } => {
            ?id :iteration [
                :result ?y ;
                :number "1" ;
            ].
        } .

        # calculate guess
        {   # Get values
            ?id :input ?a .
            ?id :iteration [
                :result ?guess ;
                :number ?i 
            ] .
            # check for recursion end
            :config :iterations ?max .
            ?i math:lessThan ?max .
            # iteration step
            (?a ?guess) math:quotient ?inter .
            (?inter ?guess) math:sum ?inter2 .
            (?inter2 "2") math:quotient ?res .
            (?i "1") math:sum ?next_i .
        } => {
            ?id :iteration [
                :result ?res ;
                :number ?next_i 
            ] . 
        } .

        # finish
        {   # get iteration's result
            ?id :iteration [
                :result ?res ;
                :number ?i
            ] .
            # check for recursion's end
            :config :iterations ?max .
            ?i math:equalTo ?max . 
        } => { ?id :sqrt ?res . } .
        "#;

        let g = Parser::new(example).collect::<Result<Vec<[CowTerm<'_>; 3]>, _>>()?;

        println!("Serialized: \n\n {:#?}", g);
        Ok(())
    }
}
