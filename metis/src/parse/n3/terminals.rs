//! Parsers for the terminals of Notation3.
//!
//! As the mostly the terminals of Turtle are used, this module is relatively
//! empty.
//!

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Production of SPARQL's VARNAME according to the
    /// [SPARQL spec](https://www.w3.org/TR/sparql11-query/#rVARNAME).
    ///
    /// `VARNAME ::= ( PN_CHARS_U | [0-9] ) ( PN_CHARS_U | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040] )*`
    pub static ref VARNAME: Regex = Regex::new(r#"^[_A-Za-z0-9\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}][_A-Za-z0-9\u{00B7}\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0300}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{203F}-\u{2040}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]*"#).unwrap();
    /// Production of the own rule `variable`
    ///
    /// `variable ::= '?' VARNAME`
    pub static ref VARIABLE: Regex = Regex::new(r#"^\?[_A-Za-z0-9\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}][_A-Za-z0-9\u{00B7}\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0300}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{203F}-\u{2040}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]*"#).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case("" => false ; "empty")]
    #[test_case("?" => false ; "empty name")]
    #[test_case("hans" => false ; "no questionmark")]
    #[test_case("?hans" => true ; "alpha")]
    #[test_case("?_" => true ; "underscore")]
    #[test_case("?1" => true ; "number")]
    #[test_case("?hans_the_1" => true ; "mixed")]
    fn check_variable(to_check: &str) -> bool {
        VARIABLE.is_match(to_check)
    }
}
