//! Implementation of Notation3 (N3).
//!
//! # Comformance to the spec
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
//!   are kommulative. Here only one prolog is allowed with the `@prefix`
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
//! The folowing grammar is supported:
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
//! | `list`          | `'(' expression* ')'` | |
//!
//! [1] The terminals of Turtle are used.

use crate::Format;

/// Type level representation of the Notation3 format.
#[derive(Debug, Clone, Copy)]
pub struct N3;

impl Format for N3 {
    /// `Self` as no additional data is required.
    type ConfigData = Self;
}
