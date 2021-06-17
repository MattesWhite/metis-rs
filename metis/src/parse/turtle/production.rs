//! Production rules of Turtle.
//!
//! In general productions are two split:
//!
//! 1. A _pure_ nom-parser that only parses the `&str` which returns
//!   `nom::Err::Error` if the parser not matches.
//! 2. A method of `Parser` that uses first the nom-parser and then builds
//!   the target object from the `&str`.
//!   If building fails `nom::Err::Failure` is returned.

use super::{terminals::*, Context, MownTerm, Parser};
use crate::collections::*;
use crate::parse::{
    parse_regex, unwrap_str, Error, IntoPR as _, MapPR as _, OrIntoPR as _, PResult, PosError,
};
use crate::Turtle;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, map_opt, opt};
use nom::multi::{many0, separated_list};
use nom::sequence::{preceded, tuple};
use nom::{error::ErrorKind, error_position, Err as NomError, IResult};
use sophia::ns::{rdf, xsd};
use sophia::term::{blank_node::BlankNode, iri::Iri, literal::Literal, mown_str::MownStr, Term};
use std::cell::RefCell;

/// A context wrapped in a RefCell.
///
/// This is necessary due to the constraints of `nom`'s parser generators (they
/// only take `Fn`).
pub type RefContext<'a> = RefCell<Context<'a, Turtle>>;

/// Apply the escape sequences of ECHAR
///
/// TODO: implement
pub(crate) fn string_escape<'a>(i: impl Into<MownStr<'a>>) -> MownStr<'a> {
    i.into()
}

/// Apply the escape sequences of UCHAR
///
/// TODO: implement
pub(crate) fn numeric_escape<'a>(i: impl Into<MownStr<'a>>) -> MownStr<'a> {
    i.into()
}

/// Parses Turtle's production
/// [1] turtleDoc ::= statement*
// pub fn doc<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, ()> {
//     many0(tuple((|i| statement(i, ctx), multispace0)))(i).map(|(rest, _)| (rest, ()))
// }

/// Parses Turtle's production
/// [2] statement ::= directive | triples '.'
// pub fn statement<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, ()> {
//     let (rest, parsed) = alt((
//         map(|i| directive(i, ctx), |_| None),
//         map(
//             tuple((|i| triples(i, ctx), multispace0, tag("."))),
//             |(spo, _, _)| Some(spo),
//         ),
//     ))(i)?;

//     if let Some(spo) = parsed {
//         ctx.borrow_mut().push_triples(spo.into_iter());
//     }
//     Ok((rest, ()))
// }

/// Parses Turtle's production
/// [3] directive ::= prefixID | base | sparqlPrefix | sparqlBase
// pub fn directive<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, ()> {
//     alt((|i| prefix_id(i, ctx), |i| base(i, ctx)))(i)
// }

/// Parses Turtle's production
/// [6] triples ::= subject predicateObjectList | blankNodePropertyList predicateObjectList?
// pub fn triples<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, SpoList<'a, Turtle>> {
//     alt((
//         map(
//             tuple((
//                 |i| subject(i, ctx),
//                 multispace1,
//                 |i| predicate_object_list(i, ctx),
//             )),
//             |(s, _, po)| SpoList::new(s, po),
//         ),
//         map(
//             tuple((
//                 |i| blank_node_property_list(i, ctx),
//                 opt(|i| predicate_object_list(i, ctx)),
//             )),
//             |(bn_po, outer_po)| {
//                 let mut bn_po = bn_po;
//                 if let Some(outer) = outer_po {
//                     bn_po.extend(outer.into_iter());
//                 }

//                 let bn = ctx.borrow_mut().new_anon_bnode();
//                 SpoList::new(bn.clone(), bn_po)
//             },
//         ),
//     ))(i)
// }

/// Parses Turtle's production
/// [7] predicateObjectList ::= verb objectList (';' (verb objectList)?)*
// pub fn predicate_object_list<'a>(
//     i: &'a str,
//     ctx: &RefContext<'a>,
// ) -> IResult<&'a str, Vec<PoList<'a, Turtle>>> {
//     separated_list(
//         tuple((multispace0, tag(";"), multispace0)),
//         map(
//             tuple((|i| verb(i, ctx), multispace1, |i| object_list(i, ctx))),
//             |(verb, _, objects)| PoList::new(verb, objects),
//         ),
//     )(i)
// }

/// Parses Turtle's production
/// [8] objectList ::= object (',' object)*
// pub fn object_list<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, TermList<'a, Turtle>> {
//     let (rest, list) = separated_list(tuple((multispace0, tag(","), multispace0)), |i| {
//         object(i, ctx)
//     })(i)?;

//     if list.len() == 0 {
//         Err(NomError::Error(error_position!(rest, ErrorKind::Verify)))
//     } else {
//         Ok((rest, list))
//     }
// }

/// Parses Turtle's production
/// [9] verb ::= predicate | 'a'
// pub fn verb<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, MownTerm<'a>> {
//     alt((
//         |i| predicate(i, ctx),
//         map(tag("a"), |_| Term::from(&rdf::type_)),
//     ))(i)
// }

/// Parses Turtle's production
/// [10] subject ::= iri | BlankNode | collection
// pub fn subject<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, MownTerm<'a>> {
//     alt((
//         |i| iri(i, ctx),
//         |i| blank_node(i, ctx),
//         |i| collection(i, ctx),
//     ))(i)
// }

/// Parses Turtle's production
/// [11] predicate ::= iri
// pub fn predicate<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, MownTerm<'a>> {
//     iri(i, ctx)
// }

/// Parses Turtle's production
/// [12] object ::= iri | BlankNode | collection | blankNodePropertyList | literal
// pub fn object<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, MownTerm<'a>> {
//     alt((
//         |i| iri(i, ctx),
//         |i| blank_node(i, ctx),
//         |i| collection(i, ctx),
//         map(
//             |i| blank_node_property_list(i, ctx),
//             |pol| {
//                 let bn = ctx.borrow_mut().new_anon_bnode();
//                 let spos = SpoList::new(bn.clone(), pol);
//                 ctx.borrow_mut().push_triples(spos.into_iter());
//                 bn
//             },
//         ),
//         |i| literal(i, ctx),
//     ))(i)
// }

/// Parses Turtle's production
/// [14] blankNodePropertyList ::= '[' predicateObjectList ']'
// pub fn blank_node_property_list<'a>(
//     i: &'a str,
//     ctx: &RefContext<'a>,
// ) -> IResult<&'a str, Vec<PoList<'a, Turtle>>> {
//     let (rest, _) = tag("[")(i)?;
//     let (rest, _) = multispace0(rest)?;
//     let (rest, contents) = predicate_object_list(rest, ctx)?;
//     let (rest, _) = multispace0(rest)?;
//     let (rest, _) = tag("]")(rest)?;

//     Ok((rest, contents))
// }

/// Parses Turtle's production
/// [15] collection ::= '(' object* ')'
///
/// # Result
///
/// Returns the subject of the first element in the collection. If the
/// collection is empty `rdf:nil` is returned.
// pub fn collection<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, MownTerm<'a>> {
//     let (rest, _) = tag("(")(i)?;
//     let (rest, _) = multispace0(rest)?;
//     let (rest, contents) = separated_list(multispace1, |i| object(i, ctx))(rest)?;
//     let (rest, _) = multispace0(rest)?;
//     let (rest, _) = tag(")")(rest)?;

//     if contents.is_empty() {
//         Ok((rest, Term::from(&rdf::nil)))
//     } else {
//         let mut cur;
//         let mut next = ctx.borrow_mut().new_anon_bnode();
//         let first = next.clone();
//         let max = contents.len() - 1;
//         for (idx, o) in contents.into_iter().enumerate() {
//             cur = next;
//             next = if idx != max {
//                 ctx.borrow_mut().new_anon_bnode()
//             } else {
//                 Term::from(&rdf::nil)
//             };
//             ctx.borrow_mut()
//                 .push_triple([cur.clone(), Term::from(&rdf::first), o]);
//             ctx.borrow_mut()
//                 .push_triple([cur, Term::from(&rdf::rest), next.clone()]);
//         }

//         Ok((rest, first))
//     }
// }

// SPARQL rules.
impl<'doc> Parser<'doc> {
    /// Parses SPARQL's production and sets the new base IRI accordingly
    /// (resolve new IRI against old base).
    ///
    /// [5s] sparqlBase ::= "BASE" IRIREF
    pub fn sparql_base(&mut self, i: &'doc str) -> PResult<'doc, ()> {
        let (rest, base) = sparql_base(i).map_pr()?;
        self.ctx
            .prolog
            .set_base(Iri::new(base).expect("Just check if absolute"))
            .map(|_| ())
            .into_pr(i, rest)
    }

    /// Parses SPARQL's production and adds the new prefix to the parser's
    /// context.
    ///
    /// [6s] sparqlPrefix ::= "PREFIX" PNAME_NS IRIREF
    pub fn sparql_prefix(&mut self, i: &'doc str) -> PResult<'doc, ()> {
        let (rest, (prefix, ns)) = sparql_prefix(i).map_pr()?;
        self.ctx.prolog.add_prefix(prefix, ns)
            .map(|_| ())
            .map_err(|_| Error::InvalidPrefix(prefix.to_string()))
            .into_pr(i, rest)
    }
}

// Turtle rules.
impl<'doc> Parser<'doc> {
    /// Parses Turtle's production
    /// [1] turtleDoc ::= statement*
    pub fn ttl_doc(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }
    
    /// Parses Turtle's production
    /// [2] statement ::= directive | triples '.'
    pub fn ttl_statement(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }
    
    /// Parses Turtle's production
    /// [3] directive ::= prefixID | base | sparqlPrefix | sparqlBase
    pub fn ttl_directive(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }
    
    /// Parses Turtle's production
    /// [6] triples ::= subject predicateObjectList | blankNodePropertyList predicateObjectList?
    pub fn ttl_triples(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }
    
    /// Parses Turtle's production
    /// [7] predicateObjectList ::= verb objectList (';' (verb objectList)?)*
    pub fn ttl_predicate_object_list(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }
    
    /// Parses Turtle's production
    /// [8] objectList ::= object (',' object)*
    pub fn ttl_object_list(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }
    
    /// Parses Turtle's production
    /// [9] verb ::= predicate | 'a'
    pub fn ttl_verb(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }

    /// Parses Turtle's production
    /// [10] subject ::= iri | BlankNode | collection
    pub fn ttl_subject(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }

    /// Parses Turtle's production
    /// [11] predicate ::= iri
    pub fn ttl_predicate(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }

    /// Parses Turtle's production
    /// [12] object ::= iri | BlankNode | collection | blankNodePropertyList | literal
    pub fn ttl_object(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }

    /// Parses Turtle's production
    /// [14] blankNodePropertyList ::= '[' predicateObjectList ']'
    pub fn ttl_blank_node_property_list(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }

    /// Parses Turtle's production
    /// [15] collection ::= '(' object* ')'
    ///
    /// # Result
    ///
    /// Returns the subject of the first element in the collection. If the
    /// collection is empty `rdf:nil` is returned.
    pub fn ttl_collection(&mut self, i: &'doc str) -> PResult<'doc, MownTerm<'doc>> {
        unimplemented!()
    }
}

// Common rules of N3-derivatives.
impl<'doc> Parser<'doc> {
    /// Parses Turtle's production and sets the new base IRI accordingly
    /// (resolve new IRI against old base).
    ///
    /// [5] base ::= '@base' IRIREF '.'
    pub fn base(&mut self, i: &'doc str) -> PResult<'doc, ()> {
        let (rest, base) = base(i).map_pr()?;
        self.ctx
            .prolog
            .set_base(Iri::new(base).expect("Just check if absolute"))
            .map(|_| ())
            .into_pr(i, rest)?;
        Ok((rest, ()))
    }

    /// Parses Turtle's production and adds the new prefix to the parser's
    /// context.
    ///
    /// [4] prefixID ::= '@prefix' PNAME_NS IRIREF '.'
    pub fn prefix(&mut self, i: &'doc str) -> PResult<'doc, ()> {
        let (rest, (prefix, ns)) = prefix(i).map_pr()?;
        self.ctx.prolog.add_prefix(prefix, ns)
            .map(|_| ())
            .map_err(|_| Error::InvalidPrefix(prefix.to_string()))
            .into_pr(i, rest)
    }

    /// Parses Turtle's production
    /// [13] literal ::= RDFLiteral | NumericLiteral | BooleanLiteral
    pub fn literal(&mut self, i: &'doc str) -> PResult<'doc, Literal<MownStr<'doc>>> {
        self.alt(
            i,
            &[
                &Self::rdf_literal,
                &Self::numeric_literal,
                &Self::boolean_literal,
            ],
        )
    }

    /// Parses Turtle's production
    /// [16] NumericLiteral ::= INTEGER | DECIMAL | DOUBLE
    pub fn numeric_literal(&mut self, i: &'doc str) -> PResult<'doc, Literal<MownStr<'doc>>> {
        alt((
            map(parse_regex(&INTEGER), |txt| {
                Literal::new_dt(txt, xsd::iri::integer)
            }),
            map(parse_regex(&DECIMAL), |txt| {
                Literal::new_dt(txt, xsd::iri::decimal)
            }),
            map(parse_regex(&DOUBLE), |txt| {
                Literal::new_dt(txt, xsd::iri::double)
            }),
        ))(i)
        .map_pr()
    }

    /// Parses Turtle's production
    /// [128s] RDFLiteral ::= String (LANGTAG | '^^' iri)?
    pub fn rdf_literal(&mut self, i: &'doc str) -> PResult<'doc, Literal<MownStr<'doc>>> {
        let (rest, txt) = lexical_value(i).map_pr()?;

        if let Ok((rest, _)) = tag("^^")(rest).map_pr() {
            let (rest, dt) = self.iri(rest)?;
            Ok((rest, Literal::new_dt(txt, dt)))
        } else if let Ok((rest, lang)) = parse_regex(&LANGTAG)(rest) {
            Literal::new_lang(txt, lang).into_pr(i, rest)
        } else {
            Ok((rest, Literal::new_dt(txt, xsd::iri::string)))
        }
    }

    /// Parses Turtle's production
    /// [133s] BooleanLiteral ::= 'true' | 'false'
    pub fn boolean_literal(&mut self, i: &'doc str) -> PResult<'doc, Literal<MownStr<'doc>>> {
        let (rest, lexical) = boolean_literal(i).map_pr()?;
        Ok((rest, Literal::new_dt(lexical, xsd::iri::boolean)))
    }

    /// Parses Turtle's production
    /// [135s] iri ::= IRIREF | PrefixedName
    pub fn iri(&mut self, i: &'doc str) -> PResult<'doc, Iri<MownStr<'doc>>> {
        self.alt(i, &[&Self::iriref, &Self::prefixed_name])
    }

    /// Parses Turtle's terminal `IRIREF` into an `Iri`.
    /// [18] IRIREF ::= '<' ([^#x00-#x20<>"{}|^`\] | UCHAR)* '>'
    pub fn iriref(&mut self, i: &'doc str) -> PResult<'doc, Iri<MownStr<'doc>>> {
        let (rest, iri) = iriref(i).map_pr()?;
        Ok((rest, self.ctx.new_iri(iri)))
    }

    /// Parses Turtle's production
    /// [136s] PrefixedName ::= PNAME_LN | PNAME_NS
    pub fn prefixed_name(&mut self, i: &'doc str) -> PResult<'doc, Iri<MownStr<'doc>>> {
        let (rest, (ns, suffix)) =
            alt((pname_ln_split, map(pname_ns, |s| (s, None))))(i).map_pr()?;
        let (_, ns) = self
            .ctx
            .prolog
            .prefixes
            .get(ns)
            .or_into_pr(i, Error::InvalidPrefix(ns.to_string()), rest)?;

        if let Some(suffix) = suffix {
            ns.get_iri(suffix).into_pr(i, rest)
        } else {
            Ok((rest, ns.clone().into()))
        }
    }

    /// Parses Turtle's production
    /// [137s] BlankNode ::= BLANK_NODE_LABEL | ANON
    pub fn blank_node(&mut self, i: &'doc str) -> PResult<'doc, BlankNode<MownStr<'doc>>> {
        let (rest, id) = alt((blank_node_label, anon))(i).map_pr()?;

        let bn = match id {
            Some(id) => self.ctx.new_labeled_bnode(&id[2..]),
            None => self.ctx.new_anon_bnode(),
        };

        Ok((rest, bn))
    }

    /// Like `nom::combinator::alt` but allows `&mut self`.
    fn alt<O>(
        &mut self,
        i: &'doc str,
        parsers: &[&dyn FnMut(&mut Self, &'doc str) -> PResult<'doc, O>],
    ) -> PResult<'doc, O> {
        for p in parsers {
            match p(self, i) {
                Err(NomError::Error(_)) => {}
                res => return res,
            };
        }
        Err(PosError::err(i, Error::NoMatch))
    }
}

/// Returns the parsed base IRI.
/// [5s] sparqlBase ::= "BASE" IRIREF
fn sparql_base(i: &str) -> IResult<&str, MownStr<'_>> {
    map(
        tuple((tag_no_case("BASE"), multispace0, iriref)),
        |(_, _, base)| base,
    )(i)
}

/// Parses SPARQL's production
/// [6s] sparqlPrefix ::= "PREFIX" PNAME_NS IRIREF
///
/// Returns `(prefix, iriref)` where `prefix` has no trailing `:` and `iriref`
/// has no wrapping `<` and `>`.
pub fn sparql_prefix(i: &str) -> IResult<&str, (&str, MownStr<'_>)> {
    map(
        tuple((
            tag_no_case("PREFIX"),
            multispace1,
            pname_ns,
            multispace0,
            iriref,
        )),
        |(_, _, p, _, ns)| (p, ns),
    )(i)
}

/// Returns the parsed base IRI.
/// [5] base ::= '@base' IRIREF '.'
pub fn base(i: &str) -> IResult<&str, MownStr<'_>> {
    map(
        tuple((tag("@base"), multispace0, iriref, multispace0, tag("."))),
        |(_, _, base, _, _)| base,
    )(i)
}

/// Parses Turtle's production
/// [4] prefixID ::= '@prefix' PNAME_NS IRIREF '.'
///
/// Returns `(prefix, iriref)` where `prefix` has no trailing `:` and `iriref`
/// has no wrapping `<` and `>`.
pub fn prefix(i: &str) -> IResult<&str, (&str, MownStr<'_>)> {
    map(
        tuple((
            tag("@prefix"),
            multispace1,
            pname_ns,
            multispace0,
            iriref,
            multispace0,
            tag("."),
        )),
        |(_, _, p, _, ns, _, _)| (p, ns),
    )(i)
}

/// Parses Turtle's production
/// [133s] BooleanLiteral ::= 'true' | 'false'
fn boolean_literal(i: &str) -> IResult<&str, &str> {
    alt((tag("true"), tag("false")))(i)
}

/// Parses Turtle's production
/// [17] String ::= STRING_LITERAL_QUOTE | STRING_LITERAL_SINGLE_QUOTE | STRING_LITERAL_LONG_SINGLE_QUOTE | STRING_LITERAL_LONG_QUOTE
fn lexical_value(i: &str) -> IResult<&str, MownStr<'_>> {
    map(
        alt((
            map(parse_regex(&STRING_LITERAL_LONG_QUOTE), |s| {
                unwrap_str(s, 3)
            }),
            map(parse_regex(&STRING_LITERAL_QUOTE), |s| unwrap_str(s, 1)),
            map(parse_regex(&STRING_LITERAL_LONG_SINGLE_QUOTE), |s| {
                unwrap_str(s, 3)
            }),
            map(parse_regex(&STRING_LITERAL_SINGLE_QUOTE), |s| {
                unwrap_str(s, 1)
            }),
        )),
        |s| string_escape(numeric_escape(s)),
    )(i)
}

/// Returns the IRI without the enclosing `<` and `>`.
fn iriref(i: &str) -> IResult<&str, MownStr<'_>> {
    map_opt(parse_regex(&IRIREF), |s| {
        if s.len() < 2 {
            None
        } else {
            Some(numeric_escape(unwrap_str(s, 1)))
        }
    })(i)
}

/// Returns the prefix without ':' at the end and the suffix.
fn pname_ln_split(i: &str) -> IResult<&str, (&str, Option<&str>)> {
    let (rest, full) = pname_ln(i)?;
    let mut parts = full.split(':');
    Ok((rest, (parts.next().unwrap(), parts.next())))
}

/// Returns the prefix without ':' at the end.
fn pname_ns(i: &str) -> IResult<&str, &str> {
    parse_regex(&PNAME_NS)(i).map(|(rest, prefix)| (rest, &prefix[..prefix.len() - 1]))
}

/// Return none if successful.
fn anon(i: &str) -> IResult<&str, Option<&str>> {
    parse_regex(&ANON)(i).map(|(rest, _)| (rest, None))
}

#[cfg(test)]
mod test {
    // use super::*;
    // use sophia::ns::xsd;
    // use sophia::term::LiteralKind;
    // use test_case::test_case;

    // fn ctx<'a>() -> RefContext<'a> {
    //     RefCell::new(Context::with_default_prefixes())
    // }

    // #[test]
    // fn check_triples() -> std::result::Result<(), Box<dyn std::error::Error>> {
    //     let check1 = Term::new_literal_dt_unchecked("45", Term::from(&xsd::integer));
    //     let check2 = Term::new_literal_dt_unchecked("false", Term::from(&xsd::boolean));
    //     let objects1 = vec![check1, Term::from(&rdf::type_)];
    //     let objects2 = vec![check2];
    //     let po1 = PoList::new(Term::from(&rdf::value), objects1);
    //     let po2 = PoList::new(Term::from(&rdf::type_), objects2);
    //     let pos = vec![po1, po2];
    //     let check3 = unsafe { Term::new_bnode_unchecked("anon0") };
    //     let res = SpoList::new(check3, pos);

    //     let ctx = ctx();
    //     let check = "[] rdf:value \"45\"^^xsd:integer, rdf:type  ;  a  false  rest";
    //     let (rest, list) = triples(check, &ctx).unwrap();
    //     assert_eq!(3, list.iter().count());
    //     assert_eq!("  rest", rest);
    //     assert_eq!(res, list);

    //     Ok(())
    // }

    // #[test]
    // fn check_predicate_object_list() -> std::result::Result<(), Box<dyn std::error::Error>> {
    //     let check1 = Term::new_literal_dt_unchecked("45", Term::from(&xsd::integer));
    //     let check2 = Term::new_literal_dt_unchecked("false", Term::from(&xsd::boolean));
    //     let objects1 = vec![check1, Term::from(&rdf::type_)];
    //     let objects2 = vec![check2];
    //     let po1 = PoList::new(Term::from(&rdf::value), objects1);
    //     let po2 = PoList::new(Term::from(&rdf::type_), objects2);
    //     let res = vec![po1, po2];

    //     let ctx = ctx();
    //     let check = "rdf:value \"45\"^^xsd:integer, rdf:type  ;  a  false  rest";
    //     let (rest, list) = predicate_object_list(check, &ctx).unwrap();
    //     assert_eq!(2, list.len());
    //     assert_eq!(
    //         3,
    //         list.iter().map(|objects| objects.iter()).flatten().count()
    //     );
    //     assert_eq!("  rest", rest);
    //     assert_eq!(res, list);

    //     Ok(())
    // }

    // #[test]
    // fn check_object_list() -> std::result::Result<(), Box<dyn std::error::Error>> {
    //     let ctx = ctx();
    //     let check = "\"45\"^^xsd:integer, rdf:type  ,  false  rest";
    //     let (rest, list) = object_list(check, &ctx).unwrap();
    //     assert_eq!(3, list.len());
    //     assert_eq!("  rest", rest);
    //     let check1 = Term::new_literal_dt_unchecked("45", xsd::integer.clone());
    //     let check3 = Term::new_literal_dt_unchecked("false", xsd::boolean.clone());
    //     assert_eq!(check1, list[0]);
    //     assert_eq!(rdf::type_, list[1]);
    //     assert_eq!(check3, list[2]);

    //     Ok(())
    // }

    // #[test]
    // fn check_iri() -> std::result::Result<(), Box<dyn std::error::Error>> {
    //     let ctx = ctx();
    //     let (rest, iri) = iri("<http://www.w3.org/1999/02/22-rdf-syntax-ns#>  rest", &ctx)?;
    //     assert_eq!("  rest", rest);
    //     assert_eq!("http://www.w3.org/1999/02/22-rdf-syntax-ns#", iri.value());

    //     Ok(())
    // }

    // #[test]
    // fn check_rdf_literal() -> std::result::Result<(), Box<dyn std::error::Error>> {
    //     let ctx = ctx();
    //     let check = "\"45\"^^xsd:integer  rest";
    //     let (rest, term) = rdf_literal(check, &ctx).unwrap();
    //     assert_eq!("  rest", rest);
    //     match term {
    //         Term::Literal(txt, LiteralKind::Datatype(dt)) => {
    //             assert_eq!("45", txt);
    //             assert_eq!(xsd::integer, dt);
    //         }
    //         _ => panic!("Wrong parsed"),
    //     };

    //     let check = "\"lorem ipsum\"   rest";
    //     let (rest, term) = rdf_literal(check, &ctx).unwrap();
    //     assert_eq!("   rest", rest);
    //     match term {
    //         Term::Literal(txt, LiteralKind::Datatype(dt)) => {
    //             assert_eq!("lorem ipsum", txt);
    //             assert_eq!(xsd::string, dt);
    //         }
    //         _ => panic!("Wrong parsed"),
    //     };

    //     let check = "\"hello\"@en  rest";
    //     let (rest, term) = rdf_literal(check, &ctx).unwrap();
    //     assert_eq!("  rest", rest);
    //     match term {
    //         Term::Literal(txt, LiteralKind::Lang(lang)) => {
    //             assert_eq!("hello", txt);
    //             assert_eq!("en", lang);
    //         }
    //         _ => panic!("Wrong parsed"),
    //     };

    //     Ok(())
    // }

    // #[test_case("\"quote\"  rest" => ("  rest", "quote") ; "quote")]
    // #[test_case("'quote'  rest" => ("  rest", "quote") ; "single")]
    // #[test_case("\"\"\"quote\"\"\"  rest" => ("  rest", "quote") ; "long quote")]
    // #[test_case("'''quote'''  rest" => ("  rest", "quote") ; "long single quote")]
    // fn check_string(i: &str) -> (&str, &str) {
    //     let (rest, string) = string(i).unwrap();
    //     if let Cow::Borrowed(string) = string {
    //         (rest, string)
    //     } else {
    //         panic!()
    //     }
    // }

    // #[test]
    // fn check_blank_node() -> std::result::Result<(), Box<dyn std::error::Error>> {
    //     let ctx = ctx();
    //     let (rest, res) = blank_node("_:example  rest", &ctx)?;
    //     assert_eq!("  rest", rest);
    //     assert_eq!("example", res.value());

    //     Ok(())
    // }
}
