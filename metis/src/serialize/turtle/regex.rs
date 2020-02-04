//! Regular expressions to match Turtle's production rules.

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Production of IRIREF according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar) (without the angle brackets).
    pub static ref IRIREF: Regex = Regex::new(r#"(?x)
		^(
			[^\u{00}-\u{20}<>"\{\}\|\^`\\]
			| (\\u [0-9A-Fa-f]{4})
			| (\\U [0-9A-Fa-f]{8})
		)* $"#).unwrap();

    /// Production of PN_CHARS_BASE according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PN_CHARS_BASE: Regex = Regex::new(r#"(?x)
        [A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]"#).unwrap();

    /// Production of PN_CHARS_U according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PN_CHARS_U: Regex = Regex::new(r#"(?x)
        [_A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]"#).unwrap();

    /// Production of PN_CHARS according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PN_CHARS: Regex = Regex::new(r#"(?x)
        [-0-9_A-Za-z\u{00B7}\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0300}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{203F}-\u{2040}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]"#).unwrap();

    /// Production of PN_PREFIX according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PN_PREFIX: Regex = Regex::new(r#"(?x)
		^(
			# PN_CHARS_BASE
			[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]
			# ( (PN_CHARS | '.')* ...
			([-\.0-9_A-Za-z\u{00B7}\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0300}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{203F}-\u{2040}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]*
			# ... PN_CHARS)?
			[-0-9_A-Za-z\u{00B7}\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0300}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{203F}-\u{2040}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}])?
		)$"#).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case("" => true ; "empty sting")]
    #[test_case("http://www.w3.org/1999/02/" => true ; "IRI")]
    #[test_case("http://www.w3.org/1999/02/22-rdf-syntax-ns#" => true ; "IRI ending with '#'")]
    #[test_case("../ns/vocab#" => true ; "relative IRI")]
    #[test_case("\\u0ace\\UFeDc0123" => true ; "numeric excape")]
    #[test_case("\0" => false ; "null character")]
    #[test_case("  " => false ; "space")]
    #[test_case("<" => false ; "open angeled")]
    #[test_case(">" => false ; "close angeled")]
    #[test_case("\"" => false ; "quote")]
    #[test_case("{" => false ; "open curly")]
    #[test_case("}" => false ; "close curly")]
    #[test_case("|" => false ; "bar")]
    #[test_case("^" => false ; "caret")]
    #[test_case("`" => false ; "back tick")]
    #[test_case("\\" => false ; "backslash")]
    #[test_case("\\u000" => false ; "numeric escape small less digits")]
    #[test_case("\\uzzzz" => false ; "numeric escape small wrong digits")]
    #[test_case("\\U000000" => false ; "numeric escape big less digits")]
    #[test_case("\\Uzzzzzzzz" => false ; "numeric escape big wrong digits")]
    fn check_iriref(to_check: &str) -> bool {
        IRIREF.is_match(to_check)
    }

    #[test_case("rBäôí" => true ; "alpha")]
    #[test_case("" => false ; "empty")]
    #[test_case("0123456789" => false ; "numeric")]
    #[test_case("_!?-:\\,.-<>#" => false ; "special")]
    fn check_pn_chars_base(to_check: &str) -> bool {
        PN_CHARS_BASE.is_match(to_check)
    }

    #[test_case("rBäôí" => true ; "alpha")]
    #[test_case("_" => true ; "allowed special")]
    #[test_case("" => false ; "empty")]
    #[test_case("0123456789" => false ; "numeric")]
    #[test_case("!?-:\\,.-<>#" => false ; "unallowed special")]
    fn check_pn_chars_u(to_check: &str) -> bool {
        PN_CHARS_U.is_match(to_check)
    }

    #[test_case("rBäôí" => true ; "alpha")]
    #[test_case("_-" => true ; "allowed special")]
    #[test_case("" => false ; "empty")]
    #[test_case("0123456789" => true ; "numeric")]
    #[test_case("!?:\\,.<>#" => false ; "unallowed special")]
    fn check_pn_chars(to_check: &str) -> bool {
        PN_CHARS.is_match(to_check)
    }

    #[test_case("rBäôí" => true ; "alpha")]
    #[test_case("a_-" => true ; "contain allowed special")]
    #[test_case("a0123456789" => true ; "contain numeric")]
    #[test_case("a01.23.456.789" => true ; "with dots")]
    #[test_case("a_01.2-3.45ö6.78î9" => true ; "mixed")]
    #[test_case("" => false ; "empty")]
    #[test_case("0" => false ; "star numeric")]
    #[test_case("_" => false ; "star allowed special")]
    #[test_case("!?:\\,.<>#" => false ; "unallowed special")]
    fn check_pn_prefix(to_check: &str) -> bool {
        PN_PREFIX.is_match(to_check)
    }
}
