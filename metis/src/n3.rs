//! Implementation of Notation3 (N3).
//!
//! # Conformance to the spec
//!
//! For now the implementation of N3 does not fully support the features and
//! syntax of the official [specification](https://www.w3.org/TeamSubmission/n3/).
//!
//! ## Grammar
//!
//! The following syntax and features not implemented according to the
//! specification:
//!
//! - *Declarations:* According to the spec, `@prefix` and `@base` declarations
//!   can be placed anywhere where a statement is valid. Relative `@base` IRI
//!   are commutative. Here only one prolog is allowed with the `@prefix`
//!   declarations first then at most one `@base` declaration.
//! - *Logic quantifiers:* Only the short annotation for variables (`?x`) is
//!   supported whereas he declarations `@forAll` and `@forSome` are not.
//! - *Rationals:* Only the numeric literals of SPARQL are supported.
//! - *Magic predicates:* In N3 calculations like `(2 2) math:sum ?x .` can be
//!   calculated. These are not implemented.
//!
//! Due to the age of the Notation3 specification the more modern specification
//! of [Turtle](https://www.w3.org/TR/turtle/) is used where equally.
//!
//! The following grammar is supported:
//!
//! | Production      | Rule | Comment |
//! | --------------- | ---- | ------- |
//! | `document`      | `statement ('.' statement)* '.' EOF` | |
//! | `statement`     | `directive \| simple_statement` | |
//! | `directive`     | `prefix_id \| base` | |
//! | `prefix_id`     | `'@prefix' PNAME_NS IRIREF` | |
//! | `base`          | `'@base' IRIREF` | |
//! | `simple_statement` | `subject property_list` | |
//! | `subject`       | `expression` | |
//! | `property_list` | `predicate object (',' object)* (';' property_list)*` | |
//! | `predicate`     | `expression \| 'a' \| '=>'` | |
//! | `object`        | `expression` | |
//! | `list`          | `'(' expression* ')'` | |
//! | `expression`    | `iri \| formula \| variable \| literal \| bnode_property_list \| list` | |
//! | `iri`           | `IRIREF \| prefixed_name` | |
//! | `prefixed_name` | `PNAME_LN \| PNAME_NS` | [1] |
//! | `formula`       | `'{' ( statement ('.' statement)* )? '}'` | |
//! | `variable`      | `'?' VARNAME` | SPARQL - VAR1 |
//! | `literal`       | `rdf_literal \| numeric_literal \| boolean_literal` | |
//! | `rdf_literal`   | `string (LANGTAG \| ('^^' iri))?` | [1] |
//! | `numeric_literal` | `INTEGER \| DECIMAL \| DOUBLE` | [1] |
//! | `boolean_literal` | `'false' \| 'true'` | from Turtle |
//! | `string`        | `STRING_LITERAL_QUOTE \| STRING_LITERAL_SINGLE_QUOTE \| STRING_LITERAL_LONG_QUOTE \| STRING_LITERAL_LONG_SINGLE_QUOTE` | [1] |
//! | `bnode_property_list` | `'[' property_list ']'` | |
//! | `blank_node`    | `BLANK_NODE_LABEL \| ANON` | [1] |
//!
//! [1] The terminals of Turtle are used.

use crate::{
    common::{RdfTerm, Valid},
    Format,
};
use sophia::term::{
    blank_node::BlankNode, iri::Iri, literal::Literal, variable::Variable, Term, TermData,
};

/// Type level representation of the Notation3 format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct N3;

impl Format for N3 {
    /// `Self` as no additional data is required.
    type ConfigData = Self;
}

/// A term in N3.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum N3Term<TD: TermData> {
    /// An IRI.
    Iri(Iri<TD>),
    /// An RDF-literal.
    Literal(Literal<TD>),
    /// An existentially quantified variable.
    Existential(BlankNode<TD>),
    /// An universally quantified variable.
    Universal(Variable<TD>),
    /// A formula.
    Formula(Formula<TD>),
}

impl<TD: TermData> From<Term<TD>> for N3Term<TD> {
    fn from(t: Term<TD>) -> Self {
        match t {
            Term::Iri(iri) => N3Term::Iri(iri),
            Term::Literal(lit) => N3Term::Literal(lit),
            Term::BNode(bn) => N3Term::Existential(bn),
            Term::Variable(var) => N3Term::Universal(var),
        }
    }
}

impl<TD: TermData> From<Iri<TD>> for N3Term<TD> {
    fn from(f: Iri<TD>) -> Self {
        N3Term::Iri(f)
    }
}

impl<TD: TermData> From<Literal<TD>> for N3Term<TD> {
    fn from(f: Literal<TD>) -> Self {
        N3Term::Literal(f)
    }
}

impl<TD: TermData> From<BlankNode<TD>> for N3Term<TD> {
    fn from(f: BlankNode<TD>) -> Self {
        N3Term::Existential(f)
    }
}

impl<TD: TermData> From<Variable<TD>> for N3Term<TD> {
    fn from(f: Variable<TD>) -> Self {
        N3Term::Universal(f)
    }
}

impl<TD: TermData> From<Formula<TD>> for N3Term<TD> {
    fn from(f: Formula<TD>) -> Self {
        N3Term::Formula(f)
    }
}

impl<TD: TermData + std::fmt::Debug> Valid<TD> for N3 {
    type Term = N3Term<TD>;
}

impl<TD: TermData + std::fmt::Debug> RdfTerm<TD> for N3Term<TD> {}

/// A N3 formula.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Formula<TD: TermData>(Vec<[N3Term<TD>; 3]>);

impl<TD: TermData> std::ops::Deref for Formula<TD> {
    type Target = Vec<[N3Term<TD>; 3]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<TD> std::ops::DerefMut for Formula<TD>
where
    TD: TermData,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<TD: TermData> From<Vec<[N3Term<TD>; 3]>> for Formula<TD> {
    fn from(v: Vec<[N3Term<TD>; 3]>) -> Self {
        Self(v)
    }
}
