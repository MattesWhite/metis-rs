//! Production rules of Turtle.

use super::{terminals::*, Context, CowTerm};
use crate::parse::util::parse_regex;
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
pub type RefContext<'a> = RefCell<Context<'a>>;

/// A list of RDF terms
pub type TermList<'a> = Vec<CowTerm<'a>>;

/// A predicate with a list of objects
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoList<'a> {
    predicate: CowTerm<'a>,
    objects: TermList<'a>,
}

impl<'a> PoList<'a> {
    /// Creates a new PO-list from a predicate and a list of objects.
    pub fn new(predicate: CowTerm<'a>, objects: TermList<'a>) -> Self {
        Self { predicate, objects }
    }
    /// Returns the list of predicates and objects.
    ///
    /// This consumes the list. For each pair the predicate is copied.
    #[allow(clippy::should_implement_trait)] // TODO: Implement IntoIterator
    pub fn into_iter(self) -> impl Iterator<Item = (CowTerm<'a>, CowTerm<'a>)> {
        let p = self.predicate;
        self.objects.into_iter().map(move |o| (p.clone(), o))
    }
    /// Returns the list of predicates and objects by reference.
    pub fn iter(&self) -> impl Iterator<Item = (&CowTerm<'a>, &CowTerm<'a>)> {
        self.objects.iter().map(move |o| (&self.predicate, o))
    }
}

/// A subject with a list of predicate-object-lists
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpoList<'a> {
    subject: CowTerm<'a>,
    po_lists: Vec<PoList<'a>>,
}

impl<'a> SpoList<'a> {
    /// Creates a new PO-list from a predicate and a list of objects.
    pub fn new(subject: CowTerm<'a>, po_lists: Vec<PoList<'a>>) -> Self {
        Self { subject, po_lists }
    }
    /// Returns the list of subject, predicates and objects.
    ///
    /// This consumes the list. For each triple the subject and predicate are
    /// copied!
    #[allow(clippy::should_implement_trait)] // TODO: Implement IntoIterator
    pub fn into_iter(self) -> impl Iterator<Item = [CowTerm<'a>; 3]> {
        let s_outer = self.subject;
        self.po_lists
            .into_iter()
            .map(move |pol| {
                let s = s_outer.clone();
                pol.into_iter().map(move |(p, o)| [s.clone(), p, o])
            })
            .flatten()
    }
    /// Returns the list of subject, predicates and objects by reference.
    pub fn iter(&self) -> impl Iterator<Item = [&CowTerm<'a>; 3]> {
        self.po_lists
            .iter()
            .map(move |pol| pol.iter().map(move |(p, o)| [&self.subject, p, o]))
            .flatten()
    }
}

/// Apply the escape sequences of UCHAR and ECHAR
///
/// TODO: implement
fn escape(i: &str) -> Cow<'_, str> {
    i.into()
}

#[inline]
fn unwrap_str(i: &str, margin: usize) -> &str {
    &i[margin..i.len() - margin]
}

/// Parses Turtle's production
/// [1] turtleDoc ::= statement*
pub fn doc<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, ()> {
    many0(tuple((|i| statement(i, ctx), multispace0)))(i).map(|(rest, _)| (rest, ()))
}

/// Parses Turtle's production
/// [2] statement ::= directive | triples '.'
pub fn statement<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, ()> {
    let (rest, parsed) = alt((
        map(|i| directive(i, ctx), |_| None),
        map(
            tuple((|i| triples(i, ctx), multispace0, tag("."))),
            |(spo, _, _)| Some(spo),
        ),
    ))(i)?;

    if let Some(spo) = parsed {
        ctx.borrow_mut().push_triples(spo.into_iter());
    }
    Ok((rest, ()))
}

/// Parses Turtle's production
/// [3] directive ::= prefixID | base | sparqlPrefix | sparqlBase
pub fn directive<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, ()> {
    alt((|i| prefix_id(i, ctx), |i| base(i, ctx)))(i)
}

/// Parses Turtle's production
/// [5] base ::= '@base' IRIREF '.'
/// [5s] sparqlBase ::= "BASE" IRIREF
///
/// Overrides the base IRI of the context which is from now on used to resolve
/// IRIREFs.
pub fn base<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, ()> {
    let (rest, base) = alt((
        map(
            tuple((
                tag("@base"),
                multispace0,
                parse_regex(&IRIREF),
                multispace0,
                tag("."),
            )),
            |(_, _, base, _, _)| base,
        ),
        map(
            tuple((tag_no_case("BASE"), multispace0, parse_regex(&IRIREF))),
            |(_, _, base)| base,
        ),
    ))(i)?;
    let base = unwrap_str(base, 1); // cut '<' and '>'

    let prolog = &mut ctx.borrow_mut().prolog;
    prolog
        .set_base(base)
        .map_err(|_| NomError::Error(error_position!(rest, ErrorKind::Verify)))?;

    Ok((rest, ()))
}

/// Parses Turtle's production
/// [4] prefixID ::= '@prefix' PNAME_NS IRIREF '.'
/// [6s] sparqlPrefix ::= "PREFIX" PNAME_NS IRIREF
///
/// Adds the parsed prefix and namespace to the context.
pub fn prefix_id<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, ()> {
    let (rest, (p, ns)) = alt((
        map(
            tuple((
                tag("@prefix"),
                multispace1,
                parse_regex(&PNAME_NS),
                multispace0,
                parse_regex(&IRIREF),
                multispace0,
                tag("."),
            )),
            |(_, _, p, _, ns, _, _)| (p, ns),
        ),
        map(
            tuple((
                tag_no_case("PREFIX"),
                multispace1,
                parse_regex(&PNAME_NS),
                multispace0,
                parse_regex(&IRIREF),
            )),
            |(_, _, p, _, ns)| (p, ns),
        ),
    ))(i)?;
    let p = &p[..p.len() - 1]; // cut the trailing ':'
    let ns = unwrap_str(ns, 1); // cut '<' and '>'

    let prolog = &mut ctx.borrow_mut().prolog;
    prolog
        .add_prefix(p, ns)
        .map_err(|_| NomError::Error(error_position!(rest, ErrorKind::Verify)))?;

    Ok((rest, ()))
}

/// Parses Turtle's production
/// [6] triples ::= subject predicateObjectList | blankNodePropertyList predicateObjectList?
pub fn triples<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, SpoList<'a>> {
    alt((
        map(
            tuple((
                |i| subject(i, ctx),
                multispace1,
                |i| predicate_object_list(i, ctx),
            )),
            |(s, _, po)| SpoList::new(s, po),
        ),
        map(
            tuple((
                |i| blank_node_property_list(i, ctx),
                opt(|i| predicate_object_list(i, ctx)),
            )),
            |(bn_po, outer_po)| {
                let mut bn_po = bn_po;
                if let Some(outer) = outer_po {
                    bn_po.extend(outer.into_iter());
                }

                let bn = ctx.borrow_mut().new_anon_bnode();
                SpoList::new(bn.clone(), bn_po)
            },
        ),
    ))(i)
}

/// Parses Turtle's production
/// [7] predicateObjectList ::= verb objectList (';' (verb objectList)?)*
pub fn predicate_object_list<'a>(
    i: &'a str,
    ctx: &RefContext<'a>,
) -> IResult<&'a str, Vec<PoList<'a>>> {
    separated_list(
        tuple((multispace0, tag(";"), multispace0)),
        map(
            tuple((|i| verb(i, ctx), multispace1, |i| object_list(i, ctx))),
            |(verb, _, objects)| PoList::new(verb, objects),
        ),
    )(i)
}

/// Parses Turtle's production
/// [8] objectList ::= object (',' object)*
pub fn object_list<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, TermList<'a>> {
    separated_list(tuple((multispace0, tag(","), multispace0)), |i| {
        object(i, ctx)
    })(i)
}

/// Parses Turtle's production
/// [9] verb ::= predicate | 'a'
pub fn verb<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    alt((
        |i| predicate(i, ctx),
        map(tag("a"), |_| Term::from(&rdf::type_)),
    ))(i)
}

/// Parses Turtle's production
/// [10] subject ::= iri | BlankNode | collection
pub fn subject<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    alt((
        |i| iri(i, ctx),
        |i| blank_node(i, ctx),
        |i| collection(i, ctx),
    ))(i)
}

/// Parses Turtle's production
/// [11] predicate ::= iri
pub fn predicate<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    iri(i, ctx)
}

/// Parses Turtle's production
/// [12] object ::= iri | BlankNode | collection | blankNodePropertyList | literal
pub fn object<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    alt((
        |i| iri(i, ctx),
        |i| blank_node(i, ctx),
        |i| collection(i, ctx),
        map(
            |i| blank_node_property_list(i, ctx),
            |pol| {
                let bn = ctx.borrow_mut().new_anon_bnode();
                let spos = SpoList::new(bn.clone(), pol);
                ctx.borrow_mut().push_triples(spos.into_iter());
                bn
            },
        ),
        |i| literal(i, ctx),
    ))(i)
}

/// Parses Turtle's production
/// [13] literal ::= RDFLiteral | NumericLiteral | BooleanLiteral
pub fn literal<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    alt((
        |i| rdf_literal(i, ctx),
        |i| numeric_literal(i, ctx),
        |i| boolean_literal(i, ctx),
    ))(i)
}

/// Parses Turtle's production
/// [14] blankNodePropertyList ::= '[' predicateObjectList ']'
pub fn blank_node_property_list<'a>(
    i: &'a str,
    ctx: &RefContext<'a>,
) -> IResult<&'a str, Vec<PoList<'a>>> {
    let (rest, _) = tag("[")(i)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, contents) = predicate_object_list(rest, ctx)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = tag("]")(rest)?;

    Ok((rest, contents))
}

/// Parses Turtle's production
/// [15] collection ::= '(' object* ')'
///
/// # Result
///
/// Returns the subject of the first element in the collection. If the
/// collection is empty `rdf:nil` is returned.
pub fn collection<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    let (rest, _) = tag("(")(i)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, contents) = separated_list(multispace1, |i| object(i, ctx))(rest)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = tag(")")(rest)?;

    if contents.is_empty() {
        Ok((rest, Term::from(&rdf::nil)))
    } else {
        let mut cur;
        let mut next = ctx.borrow_mut().new_anon_bnode();
        let first = next.clone();
        let max = contents.len() - 1;
        for (idx, o) in contents.into_iter().enumerate() {
            cur = next;
            next = if idx != max {
                ctx.borrow_mut().new_anon_bnode()
            } else {
                Term::from(&rdf::nil)
            };
            ctx.borrow_mut()
                .push_triple([cur.clone(), Term::from(&rdf::first), o]);
            ctx.borrow_mut()
                .push_triple([cur, Term::from(&rdf::rest), next.clone()]);
        }

        Ok((rest, first))
    }
}

/// Parses Turtle's production
/// [16] NumericLiteral ::= INTEGER | DECIMAL | DOUBLE
pub fn numeric_literal<'a>(i: &'a str, _: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    alt((
        map(parse_regex(&INTEGER), |i| {
            Term::new_literal_dt_unchecked(i, Term::from(&xsd::integer))
        }),
        map(parse_regex(&DECIMAL), |d| {
            Term::new_literal_dt_unchecked(d, Term::from(&xsd::decimal))
        }),
        map(parse_regex(&DOUBLE), |f| {
            Term::new_literal_dt_unchecked(f, Term::from(&xsd::double))
        }),
    ))(i)
}

/// Parses Turtle's production
/// [128s] RDFLiteral ::= String (LANGTAG | '^^' iri)?
pub fn rdf_literal<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    let (rest, string) = string(i, ctx)?;

    if let Ok((rest, dt)) = preceded(tag("^^"), |i| iri(i, ctx))(rest) {
        Ok((rest, Term::new_literal_dt_unchecked(string, dt)))
    } else if let Ok((rest, lang)) = parse_regex(&LANGTAG)(rest) {
        let term = Term::new_literal_lang(string, &lang[1..]) // cut the '@'
            .map_err(|_| NomError::Error(error_position!(rest, ErrorKind::Verify)))?;
        Ok((rest, term))
    } else {
        Ok((
            rest,
            Term::new_literal_dt_unchecked(string, Term::from(&xsd::string)),
        ))
    }
}

/// Parses Turtle's production
/// [133s] BooleanLiteral ::= 'true' | 'false'
pub fn boolean_literal<'a>(i: &'a str, _: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    map(alt((tag("true"), tag("false"))), |s| {
        Term::new_literal_dt_unchecked(s, Term::from(&xsd::boolean))
    })(i)
}

/// Parses Turtle's production
/// [17] String ::= STRING_LITERAL_QUOTE | STRING_LITERAL_SINGLE_QUOTE | STRING_LITERAL_LONG_SINGLE_QUOTE | STRING_LITERAL_LONG_QUOTE
pub fn string<'a>(i: &'a str, _: &RefContext<'a>) -> IResult<&'a str, Cow<'a, str>> {
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
        escape,
    )(i)
}

/// Parses Turtle's production
/// [135s] iri ::= IRIREF | PrefixedName
pub fn iri<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    alt((
        map_opt(parse_regex(&IRIREF), |s| {
            if s.len() < 2 {
                None
            } else {
                Some(ctx.borrow().new_iri(unwrap_str(s, 1)))
            }
        }),
        |i| prefixed_name(i, ctx),
    ))(i)
}

/// Parses Turtle's production
/// [136s] PrefixedName ::= PNAME_LN | PNAME_NS
pub fn prefixed_name<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    if let Ok((rest, parsed)) = pname_ln(i) {
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
            Term::new_iri2_unchecked(ns.clone(), escape(suffix), None)
        }));
    };

    map_opt(parse_regex(&PNAME_NS), |s| {
        let ns = &s[..s.len() - 1]; // last char is ':'
        ctx.borrow()
            .prolog
            .prefixes
            .get(ns)
            .map(|ns| unsafe { Term::new_iri_unchecked(ns.clone(), None) })
    })(i)
}

/// Parses Turtle's production
/// [137s] BlankNode ::= BLANK_NODE_LABEL | ANON
pub fn blank_node<'a>(i: &'a str, ctx: &RefContext<'a>) -> IResult<&'a str, CowTerm<'a>> {
    alt((
        // ok because validity of label is checked.
        map(blank_node_label, |s| {
            ctx.borrow_mut().new_labeled_bnode(&s[2..]) // skip the parsed `_:`
        }),
        map(parse_regex(&ANON), |_| ctx.borrow_mut().new_anon_bnode()),
    ))(i)
}

#[cfg(test)]
mod test {
    use super::*;
    use sophia::ns::xsd;
    use sophia::term::LiteralKind;
    use test_case::test_case;

    fn ctx<'a>() -> RefContext<'a> {
        let mut ctx = Context::default();
        ctx.prolog.add_default_prefixes();
        RefCell::new(ctx)
    }

    #[test_case("12345a54321", 0 => "12345a54321" ; "margin 0")]
    #[test_case("12345a54321", 1 =>  "2345a5432" ; "margin 1")]
    #[test_case("12345a54321", 2 =>   "345a543" ; "margin 2")]
    fn check_unwrap_str(i: &str, margin: usize) -> &str {
        unwrap_str(i, margin)
    }

    #[test]
    fn check_triples() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let check1 = Term::new_literal_dt_unchecked("45", Term::from(&xsd::integer));
        let check2 = Term::new_literal_dt_unchecked("false", Term::from(&xsd::boolean));
        let objects1 = vec![check1, Term::from(&rdf::type_)];
        let objects2 = vec![check2];
        let po1 = PoList::new(Term::from(&rdf::value), objects1);
        let po2 = PoList::new(Term::from(&rdf::type_), objects2);
        let pos = vec![po1, po2];
        let check3 = unsafe { Term::new_bnode_unchecked("anon0") };
        let res = SpoList::new(check3, pos);

        let ctx = ctx();
        let check = "[] rdf:value \"45\"^^xsd:integer, rdf:type  ;  a  false  rest";
        let (rest, list) = triples(check, &ctx).unwrap();
        assert_eq!(3, list.iter().count());
        assert_eq!("  rest", rest);
        assert_eq!(res, list);

        Ok(())
    }

    #[test]
    fn check_predicate_object_list() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let check1 = Term::new_literal_dt_unchecked("45", Term::from(&xsd::integer));
        let check2 = Term::new_literal_dt_unchecked("false", Term::from(&xsd::boolean));
        let objects1 = vec![check1, Term::from(&rdf::type_)];
        let objects2 = vec![check2];
        let po1 = PoList::new(Term::from(&rdf::value), objects1);
        let po2 = PoList::new(Term::from(&rdf::type_), objects2);
        let res = vec![po1, po2];

        let ctx = ctx();
        let check = "rdf:value \"45\"^^xsd:integer, rdf:type  ;  a  false  rest";
        let (rest, list) = predicate_object_list(check, &ctx).unwrap();
        assert_eq!(2, list.len());
        assert_eq!(
            3,
            list.iter().map(|objects| objects.iter()).flatten().count()
        );
        assert_eq!("  rest", rest);
        assert_eq!(res, list);

        Ok(())
    }

    #[test]
    fn check_object_list() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let ctx = ctx();
        let check = "\"45\"^^xsd:integer, rdf:type  ,  false  rest";
        let (rest, list) = object_list(check, &ctx).unwrap();
        assert_eq!(3, list.len());
        assert_eq!("  rest", rest);
        let check1 = Term::new_literal_dt_unchecked("45", xsd::integer.clone());
        let check3 = Term::new_literal_dt_unchecked("false", xsd::boolean.clone());
        assert_eq!(check1, list[0]);
        assert_eq!(rdf::type_, list[1]);
        assert_eq!(check3, list[2]);

        Ok(())
    }

    #[test]
    fn check_iri() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let ctx = ctx();
        let (rest, iri) = iri("<http://www.w3.org/1999/02/22-rdf-syntax-ns#>  rest", &ctx)?;
        assert_eq!("  rest", rest);
        assert_eq!("http://www.w3.org/1999/02/22-rdf-syntax-ns#", iri.value());

        Ok(())
    }

    #[test]
    fn check_rdf_literal() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let ctx = ctx();
        let check = "\"45\"^^xsd:integer  rest";
        let (rest, term) = rdf_literal(check, &ctx).unwrap();
        assert_eq!("  rest", rest);
        match term {
            Term::Literal(txt, LiteralKind::Datatype(dt)) => {
                assert_eq!("45", txt);
                assert_eq!(xsd::integer, dt);
            }
            _ => panic!("Wrong parsed"),
        };

        let check = "\"lorem ipsum\"   rest";
        let (rest, term) = rdf_literal(check, &ctx).unwrap();
        assert_eq!("   rest", rest);
        match term {
            Term::Literal(txt, LiteralKind::Datatype(dt)) => {
                assert_eq!("lorem ipsum", txt);
                assert_eq!(xsd::string, dt);
            }
            _ => panic!("Wrong parsed"),
        };

        let check = "\"hello\"@en  rest";
        let (rest, term) = rdf_literal(check, &ctx).unwrap();
        assert_eq!("  rest", rest);
        match term {
            Term::Literal(txt, LiteralKind::Lang(lang)) => {
                assert_eq!("hello", txt);
                assert_eq!("en", lang);
            }
            _ => panic!("Wrong parsed"),
        };

        Ok(())
    }

    #[test_case("\"quote\"  rest" => ("  rest", "quote") ; "quote")]
    #[test_case("'quote'  rest" => ("  rest", "quote") ; "single")]
    #[test_case("\"\"\"quote\"\"\"  rest" => ("  rest", "quote") ; "long quote")]
    #[test_case("'''quote'''  rest" => ("  rest", "quote") ; "long single quote")]
    fn check_string(i: &str) -> (&str, &str) {
        let ctx = ctx();
        let (rest, string) = string(i, &ctx).unwrap();
        if let Cow::Borrowed(string) = string {
            (rest, string)
        } else {
            panic!()
        }
    }

    #[test]
    fn check_blank_node() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let ctx = ctx();
        let (rest, res) = blank_node("_:example  rest", &ctx)?;
        assert_eq!("  rest", rest);
        assert_eq!("example", res.value());

        Ok(())
    }
}
