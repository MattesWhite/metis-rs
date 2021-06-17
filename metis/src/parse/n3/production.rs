//! Production rules of N3.
//!
//! # Relation Turtle parsers
//!
//! As Turtle is a subset of Notation3 many Turtle parsers could be used here.
//! However, both parsers use different `Context`'s So they can not be mixed.
//! The general strategie is to fic bugs in the Turtle parser and copy them to
//! N3 if suitable. Not very nice but for now okay.
//!
//! # Known issues
//!
//! The internal stack of the context is suitable for Turtle but not for
//! formats with multiple graphs, e.g. N3's formulae.
//! => the stack must be removed and parsed triples returned alongside the
//! parser.
//!

use super::terminals::*;
use super::CowTerm;
use crate::n3::{Formula, N3Term};
use crate::ns::log;
use crate::parse::{
    parse_regex, turtle::production as ttl_production, turtle::terminals as ttl_terminal,
    unwrap_str, Context,
};
use crate::N3;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, map_opt, opt};
use nom::multi::{many0, separated_list};
use nom::sequence::{preceded, tuple};
use nom::{error::ErrorKind, error_position, Err as NomError, IResult};
use sophia::ns::{rdf, xsd};
use sophia::term::Term;
use std::borrow::Cow;
use std::cell::RefCell;

/// A context wrapped in a RefCell.
///
/// This is necessary due to the constraints of `nom`'s parser generators (they
/// only take `Fn`).
pub type RefContext<'a> = RefCell<Context<'a, N3>>;

/// Formula with Cow
pub type CowFormula<'a> = Formula<Cow<'a, str>>;

/// Triple of `N3Term`s
pub type N3Triple<'a> = [CowTerm<'a>; 3];

/// Iterator of `N3Term`s
pub type N3TriplesIter<'a> = Box<dyn 'a + Iterator<Item = [CowTerm<'a>; 3]>>;

/// A `Vec` of `N3Triple`s
pub type N3TriplesList<'a> = Vec<N3Triple<'a>>;

/// Used when an iterator is reuiqured but none provided.
fn empty_iter<'a>() -> N3TriplesIter<'a> {
    Box::new(None.into_iter()) as _
}

/// Parses rule `statement`
pub fn statement<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, N3TriplesList<'a>> {
    unimplemented!()
}

/// Parses rule `subject`
pub fn subject<'a>(
    i: &'a str,
    ctx: &RefContext<'a>,
) -> IResult<&'a str, (CowTerm<'a>, N3TriplesIter<'a>)> {
    expression(i, ctx)
}

/// Parses rule `property_list`.
///
/// All parserd triples are pushed onto the context's stack.
pub fn property_list<'a, 's>(
    i: &'a str,
    s: CowTerm<'a>,
    ctx: &RefContext<'a>,
) -> IResult<&'a str, N3TriplesIter<'a>> {
    let (rest, list) = separated_list(
        tuple((
            ttl_terminal::multispace0,
            tag(";"),
            ttl_terminal::multispace0,
        )),
        map(
            tuple((
                |i| predicate(i, ctx),
                ttl_terminal::multispace1,
                |i| object_list(i, ctx),
            )),
            |((verb, v_others), _, (objects, o_others))| {
                // TODO: dont allocate an extra vector
                let base: Vec<_> = objects
                    .into_iter()
                    .map(|o| [s.clone(), verb.clone(), o])
                    .collect();
                base.into_iter().chain(v_others).chain(o_others)
            },
        ),
    )(i)?;

    if list.len() == 0 {
        return Err(NomError::Error(error_position!(rest, ErrorKind::Verify)));
    }

    Ok((rest, Box::new(list.into_iter().flatten())))
}

/// Parses rule `object_list`
pub fn object_list<'a>(
    i: &'a str,
    ctx: &RefContext<'a>,
) -> IResult<&'a str, (Vec<CowTerm<'a>>, N3TriplesIter<'a>)> {
    let (rest, list) = separated_list(
        tuple((
            ttl_terminal::multispace0,
            tag(","),
            ttl_terminal::multispace0,
        )),
        |i| object(i, ctx),
    )(i)?;

    if list.len() == 0 {
        Err(NomError::Error(error_position!(rest, ErrorKind::Verify)))
    } else {
        let mut objects = vec![];
        let mut others = vec![];
        list.into_iter().for_each(|(object, iter)| {
            objects.push(object);
            others.push(iter);
        });

        Ok((rest, (objects, Box::new(others.into_iter().flatten()))))
    }
}

/// Parses rule `predicate`
pub fn predicate<'a>(
    i: &'a str,
    ctx: &RefContext<'a>,
) -> IResult<&'a str, (CowTerm<'a>, N3TriplesIter<'a>)> {
    alt((
        |i| expression(i, ctx),
        map(tag("a"), |_| (Term::from(&rdf::type_).into(), empty_iter())),
        map(tag("=>"), |_| {
            (Term::from(&log::implies).into(), empty_iter())
        }),
    ))(i)
}

/// Parses rule `object`
pub fn object<'a>(
    i: &'a str,
    ctx: &RefContext<'a>,
) -> IResult<&'a str, (CowTerm<'a>, N3TriplesIter<'a>)> {
    expression(i, ctx)
}

/// Parses rule `list`
///
/// # Result
///
/// Returns the subject of the first element in the list. If the collection is
/// empty `rdf:nil` is returned. In addition, the triples representing the rest
/// of the list are pushed on the context's stack.
pub fn list<'a>(
    i: &'a str,
    ctx: &RefContext<'a>,
) -> IResult<&'a str, (CowTerm<'a>, N3TriplesIter<'a>)> {
    let (rest, _) = tag("(")(i)?;
    let (rest, _) = ttl_terminal::multispace0(rest)?;
    let (rest, contents) = separated_list(ttl_terminal::multispace1, |i| expression(i, ctx))(rest)?;
    let (rest, _) = ttl_terminal::multispace0(rest)?;
    let (rest, _) = tag(")")(rest)?;

    if contents.is_empty() {
        Ok((rest, (Term::from(&rdf::nil).into(), empty_iter())))
    } else {
        let mut cur;
        let mut next = ctx.borrow_mut().new_anon_bnode();
        let first = next.clone();
        let max = contents.len() - 1;
        let mut triples = Vec::with_capacity(contents.len());
        let mut others = Vec::with_capacity(contents.len());

        for (idx, (o, other)) in contents.into_iter().enumerate() {
            cur = next;
            next = if idx != max {
                ctx.borrow_mut().new_anon_bnode()
            } else {
                Term::from(&rdf::nil).into()
            };
            triples.push([cur.clone(), Term::from(&rdf::first).into(), o]);
            triples.push([cur, Term::from(&rdf::rest).into(), next.clone()]);
            others.push(other);
        }

        Ok((
            rest,
            (
                first,
                Box::new(triples.into_iter().chain(others.into_iter().flatten())),
            ),
        ))
    }
}

/// Parses rule `expression`
///
/// Returns the 'value' of the expression and triples parsed while evaluating.
pub fn expression<'a>(
    i: &'a str,
    ctx: &RefContext<'a>,
) -> IResult<&'a str, (CowTerm<'a>, N3TriplesIter<'a>)> {
    alt((
        |i| iri(i, ctx).map(|(r, t)| (r, (t, empty_iter()))),
        |i| formula(i, ctx).map(|(r, f)| (r, (f.into(), empty_iter()))),
        |i| variable(i).map(|(r, t)| (r, (t, empty_iter()))),
        |i| literal(i, ctx).map(|(r, t)| (r, (t, empty_iter()))),
        |i| list(i, ctx),
        |i| bnode_property_list(i, ctx),
    ))(i)
}

/// Parses rule `iri`
pub fn iri<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    alt((
        map_opt(parse_regex(&ttl_terminal::IRIREF), |s| {
            if s.len() < 2 {
                None
            } else {
                Some(ctx.borrow().new_iri(unwrap_str(s, 1)))
            }
        }),
        |i| prefixed_name(i, ctx),
    ))(i)
}

/// Parses rule `prefixed_name`
pub fn prefixed_name<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    if let Ok((rest, parsed)) = ttl_terminal::pname_ln(i) {
        let mut parts = parsed.split(':');
        let (ns, suffix) = (parts.next().unwrap(), parts.next().unwrap());
        // TODO: introduce proper error handling
        let ctx = ctx.borrow();
        let ns = ctx
            .prolog
            .prefixes
            .get(ns)
            .ok_or_else(|| NomError::Error(error_position!(i, ErrorKind::Verify)))?;
        return Ok((rest, unsafe {
            Term::new_iri2_unchecked(ns.clone(), ttl_production::escape(suffix), None).into()
        }));
    };

    map_opt(parse_regex(&ttl_terminal::PNAME_NS), |s| {
        let ns = &s[..s.len() - 1]; // last char is ':'
        ctx.borrow()
            .prolog
            .prefixes
            .get(ns)
            .map(|ns| unsafe { Term::new_iri_unchecked(ns.clone(), None).into() })
    })(i)
}

/// Parses rule `formula`
pub fn formula<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowFormula<'a>> {
    tuple((
        tag("{"),
        ttl_terminal::multispace0,
        opt(tuple((
            |i| statement(i, ctx),
            ttl_terminal::multispace0,
            opt(separated_list(
                tuple((
                    ttl_terminal::multispace0,
                    tag("."),
                    ttl_terminal::multispace0,
                )),
                |i| statement(i, ctx),
            )),
        ))),
        ttl_terminal::multispace0,
        tag("}"),
    ))(i)
    .map(|(rest, (_, _, statements, _, _))| match statements {
        Some((mut first, _, others)) => match others {
            Some(others) => {
                first.extend(others.into_iter().flatten());
                (rest, first.into())
            }
            None => (rest, first.into()),
        },
        None => (rest, Formula::default()),
    })
}

/// Parses rule `variable`
pub fn variable<'a>(i: &'a str) -> IResult<&'a str, CowTerm<'a>> {
    // cut the leading '?'
    map(parse_regex(&VARIABLE), |s| {
        unsafe { Term::new_variable_unchecked(&s[1..]) }.into()
    })(i)
}

/// Parses rule `literal`
pub fn literal<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    alt((
        |i| rdf_literal(i, ctx),
        |i| numeric_literal(i),
        |i| boolean_literal(i),
    ))(i)
}

/// Parses rule `rdf_literal`
pub fn rdf_literal<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    let (rest, string) = string(i)?;

    if let Ok((rest, N3Term::Term(dt))) = preceded(tag("^^"), |i| iri(i, ctx))(rest) {
        Ok((rest, Term::new_literal_dt_unchecked(string, dt).into()))
    } else if let Ok((rest, lang)) = parse_regex(&ttl_terminal::LANGTAG)(rest) {
        let term = Term::new_literal_lang(string, &lang[1..]) // cut the '@'
            .map(Into::into)
            .map_err(|_| NomError::Error(error_position!(rest, ErrorKind::Verify)))?;
        Ok((rest, term))
    } else {
        Ok((
            rest,
            Term::new_literal_dt_unchecked(string, Term::from(&xsd::string)).into(),
        ))
    }
}

/// Parses rule `numeric_literal`
pub fn numeric_literal<'a>(i: &'a str) -> IResult<&'a str, CowTerm<'a>> {
    ttl_production::numeric_literal(i).map(|(rest, t)| (rest, t.into()))
}

/// Parses rule `boolean_literal`
pub fn boolean_literal<'a>(i: &'a str) -> IResult<&'a str, CowTerm<'a>> {
    ttl_production::boolean_literal(i).map(|(rest, t)| (rest, t.into()))
}

/// Parses rule `string`
pub fn string<'a>(i: &'a str) -> IResult<&'a str, Cow<'a, str>> {
    ttl_production::string(i)
}

/// Parses rule `bnode_property_list`
///
/// Returns the node representing the parsed anonymous blank node and the
/// triples contained are pushed to the context's stack.
pub fn bnode_property_list<'a>(
    i: &'a str,
    ctx: &RefContext<'a>,
) -> IResult<&'a str, (CowTerm<'a>, N3TriplesIter<'a>)> {
    let (rest, _) = tag("[")(i)?;
    let (rest, _) = ttl_terminal::multispace0(rest)?;
    let node = ctx.borrow_mut().new_anon_bnode();
    let (rest, triples) = property_list(rest, node.clone(), ctx)?;
    let (rest, _) = ttl_terminal::multispace0(rest)?;
    let (rest, _) = tag("]")(rest)?;

    Ok((rest, (node, triples)))
}

/// Parses rule `blank_node`
pub fn blank_node<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    alt((
        // ok because validity of label is checked.
        map(ttl_terminal::blank_node_label, |s| {
            ctx.borrow_mut().new_labeled_bnode(&s[2..]) // skip the parsed `_:`
        }),
        map(parse_regex(&ttl_terminal::ANON), |_| {
            ctx.borrow_mut().new_anon_bnode()
        }),
    ))(i)
}
