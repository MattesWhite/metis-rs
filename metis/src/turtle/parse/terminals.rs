//! Parsers for the terminals of Turtle.
//!
//! # Escape
//!
//! All terminals perform no escape resolution, meaning `"\u0020"` will be
//! recognized but not resolved.
//!
//! # Provided
//!
//! Most terminals are provided as regular expressions. Some were to complex
//! for one expression. Therefore, those are implemented as `nom` parser
//! functions.
//!

use crate::parse::util::parse_regex;
use lazy_static::lazy_static;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{opt, recognize};
use nom::multi::many0;
use nom::sequence::tuple;
use nom::IResult;
use regex::Regex;

lazy_static! {
    /// Matches all characters invalid within IRIREF.
    pub static ref UNALLOWED_IRIREF_CHARS: Regex = Regex::new(r#"[^\u{00}-\u{20}<>"\{\}\|\^`\\]"#).unwrap();
    /// Production of IRIREF according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref IRIREF: Regex = Regex::new(r#"^<([^\u{00}-\u{20}<>"\{\}\|\^`\\]|(\\u[[:xdigit:]]{4})|(\\U[[:xdigit:]]{8}))*>"#).unwrap();
    /// Production of IRIREF according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar)
    /// (without the angle brackets).
    ///
    /// Used to determine if a complete string is a valid IRIREF.
    pub static ref IRIREF_ONLY: Regex = Regex::new(r#"(?x)
        ^(
            [^\u{00}-\u{20}<>"\{\}\|\^`\\]
            | (\\u [[:xdigit:]]{4})
            | (\\U [[:xdigit:]]{8})
        )* $"#).unwrap();

    /// Production of PN_CHARS_BASE according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PN_CHARS_BASE: Regex = Regex::new(r#"^[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]"#).unwrap();

    /// Production of PN_CHARS_U according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PN_CHARS_U: Regex = Regex::new(r#"^[_A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]"#).unwrap();

    /// Production of PN_CHARS according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PN_CHARS: Regex = Regex::new(r#"^[-0-9_A-Za-z\u{00B7}\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0300}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{203F}-\u{2040}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]"#).unwrap();

    /// Production of PN_PREFIX according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PN_PREFIX: Regex = Regex::new(r#"^([A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]([-\.0-9_A-Za-z\u{00B7}\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0300}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{203F}-\u{2040}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]*[-0-9_A-Za-z\u{00B7}\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0300}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{203F}-\u{2040}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}])?)"#).unwrap();
    /// Production of PN_PREFIX according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PNAME_NS: Regex = Regex::new(r#"^([A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]([-\.0-9_A-Za-z\u{00B7}\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0300}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{203F}-\u{2040}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}]*[-0-9_A-Za-z\u{00B7}\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0300}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{203F}-\u{2040}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\U{00010000}-\U{000EFFFF}])?)?:"#).unwrap();
    /// Production of LANGTAG according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref LANGTAG: Regex = Regex::new(r#"^@[[:alpha:]]+(-[[:alnum:]]+)*"#).unwrap();
    /// Production of INTEGER according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref INTEGER: Regex = Regex::new(r#"^[+-]?[[:digit:]]+"#).unwrap();
    /// Production of DECIMAL according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref DECIMAL: Regex = Regex::new(r#"^[+-]?[[:digit:]]*\.[[:digit:]]+"#).unwrap();
    /// Production of DOUBLE according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref DOUBLE: Regex = Regex::new(r#"^[+-]?(([[:digit:]]+\.[[:digit:]]*[eE][+-]?[[:digit:]]+)|(\.[[:digit:]]+[eE][+-]?[[:digit:]]+)|([[:digit:]]+[eE][+-]?[[:digit:]]+))"#).unwrap();
    /// Production of EXPONENT according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref EXPONENT: Regex = Regex::new(r#"^[eE][+-]?[[:digit:]]+"#).unwrap();
    /// Production of STRING_LITERAL_QUOTE according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref STRING_LITERAL_QUOTE: Regex = Regex::new(r#"^"([^\u{22}\u{5C}\u{A}\u{D}]|(\\[tbnrf"'\\])|(\\u[[:xdigit:]]{4})|(\\U[[:xdigit:]]{8}))*""#).unwrap();
    /// Production of STRING_LITERAL_SINGLE_QUOTE according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref STRING_LITERAL_SINGLE_QUOTE: Regex = Regex::new(r#"^'([^\u{27}\u{5C}\u{A}\u{D}]|(\\[tbnrf"'\\])|(\\u[[:xdigit:]]{4})|(\\U[[:xdigit:]]{8}))*'"#).unwrap();
    /// Production of STRING_LITERAL_LONG_QUOTE according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref STRING_LITERAL_LONG_QUOTE: Regex = Regex::new(r#"^"""((("|"")?([^"\\]|(\\[tbnrf"'\\])|(\\u[[:xdigit:]]{4})|(\\U[[:xdigit:]]{8}))))*""""#).unwrap();
    /// Production of STRING_LITERAL_LONG_SINGLE_QUOTE according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref STRING_LITERAL_LONG_SINGLE_QUOTE: Regex = Regex::new(r#"^'''((('|'')?([^'\\]|(\\[tbnrf"'\\])|(\\u[[:xdigit:]]{4})|(\\U[[:xdigit:]]{8}))))*'''"#).unwrap();
    /// Production of UCHAR according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref UCHAR: Regex = Regex::new(r#"^(\\u[[:xdigit:]]{4})|(\\U[[:xdigit:]]{8})"#).unwrap();
    /// Production of ECHAR according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref ECHAR: Regex = Regex::new(r#"^\\[tbnrf"'\\]"#).unwrap();
    /// Production of WS according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    /// Parses comments as well
    pub static ref WS: Regex = Regex::new(r#"^[ \t\n\r]|(#[^\n]*\n)"#).unwrap();
    /// Parses ^ WS?
    pub static ref WS_MANY1: Regex = Regex::new(r#"^([ \t\n\r]|(#[^\n]*\n))+"#).unwrap();
    /// Parses ^ WS*
    pub static ref WS_MANY0: Regex = Regex::new(r#"^([ \t\n\r]|(#[^\n]*\n))*"#).unwrap();
    /// Production of ANON according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref ANON: Regex = Regex::new(r#"^\[[ \t\n\r]*\]"#).unwrap();
    /// Production of PLX according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PLX: Regex = Regex::new(r#"^(%[[:xdigit:]]{2})|(\\[-_~\.!\$&'#\(\)\*\+,;=/\?@%])"#).unwrap();
    /// Production of PERCENT according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PERCENT: Regex = Regex::new(r#"^%[[:xdigit:]]{2}"#).unwrap();
    /// Production of HEX according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref HEX: Regex = Regex::new(r#"^[[:xdigit:]]"#).unwrap();
    /// Production of HEX according to the [Turtle spec](https://www.w3.org/TR/turtle/#sec-grammar).
    pub static ref PN_LOCAL_ESC: Regex = Regex::new(r#"^\\[-_~\.!\$&'\(\)\*\+,;=/\?#@%]"#).unwrap();

    /// Utiliy
    static ref DIGIT: Regex = Regex::new(r#"^[[:digit:]]"#).unwrap();
}

/// Parses Turtle's rule
/// [140s] PNAME_LN ::= PNAME_NS PN_LOCAL
pub fn pname_ln(i: &str) -> IResult<&str, &str> {
    recognize(tuple((parse_regex(&PNAME_NS), pn_local)))(i)
}

/// Parses Turtle's rule
/// [141s] BLANK_NODE_LABEL ::= '_:' (PN_CHARS_U | [[:digit:]]) ((PN_CHARS | '.')* PN_CHARS)?
pub fn blank_node_label(i: &str) -> IResult<&str, &str> {
    recognize(tuple((
        tag("_:"),
        alt((parse_regex(&PN_CHARS_U), parse_regex(&DIGIT))),
        many0(alt((parse_regex(&PN_CHARS), tag(".")))),
        opt(parse_regex(&PN_CHARS)),
    )))(i)
}

/// Parses Turtle's rule
/// [168s] PN_LOCAL ::= (PN_CHARS_U | ':' | [[:digit:]] | PLX) ((PN_CHARS | '.' | ':' | PLX)* (PN_CHARS | ':' | PLX))?
pub fn pn_local(i: &str) -> IResult<&str, &str> {
    recognize(tuple((
        alt((
            parse_regex(&PN_CHARS_U),
            tag(":"),
            parse_regex(&DIGIT),
            parse_regex(&PLX),
        )),
        many0(alt((
            parse_regex(&PN_CHARS),
            tag("."),
            tag(":"),
            parse_regex(&PLX),
        ))),
        opt(alt((parse_regex(&PN_CHARS), tag(":"), parse_regex(&PLX)))),
    )))(i)
}

/// Parses at least one whitespace (includeing comments).
pub fn multispace1(i: &str) -> IResult<&str, &str> {
    parse_regex(&WS_MANY1)(i)
}

/// Parses zero or more whitespaces (includeing comments).
pub fn multispace0(i: &str) -> IResult<&str, &str> {
    parse_regex(&WS_MANY0)(i)
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case("<>" => true ; "empty sting")]
    #[test_case("<http://www.w3.org/1999/02/>" => true ; "IRI")]
    #[test_case("<http://www.w3.org/1999/02/22-rdf-syntax-ns#>" => true ; "IRI ending with '#'")]
    #[test_case("<../ns/vocab#>" => true ; "relative IRI")]
    #[test_case("<\\u0ace>" => true ; "numeric escape small")]
    #[test_case("<\\UFeDc0123>" => true ; "numeric escape big")]
    #[test_case("<\0>" => false ; "null character")]
    #[test_case("<  >" => false ; "space")]
    #[test_case("<\">" => false ; "quote")]
    #[test_case("<{>" => false ; "open curly")]
    #[test_case("<}>" => false ; "close curly")]
    #[test_case("<|>" => false ; "bar")]
    #[test_case("<^>" => false ; "caret")]
    #[test_case("<`>" => false ; "back tick")]
    #[test_case("<\\>" => false ; "backslash")]
    #[test_case("<\\u000>" => false ; "numeric escape small less digits")]
    #[test_case("<\\uzzzz>" => false ; "numeric escape small wrong digits")]
    #[test_case("<\\U000000>" => false ; "numeric escape big less digits")]
    #[test_case("<\\Uzzzzzzzz>" => false ; "numeric escape big wrong digits")]
    fn check_iriref(to_check: &str) -> bool {
        IRIREF.is_match(to_check)
    }

    #[test_case("rBäôí" => true ; "alpha")]
    #[test_case("" => false ; "empty")]
    #[test_case("0123456789" => false ; "numeric")]
    #[test_case("_!?-:\\,.-<>#" => false ; "special")]
    #[test_case(" " => false ; "space")]
    fn check_pn_chars_base(to_check: &str) -> bool {
        PN_CHARS_BASE.is_match(to_check)
    }

    #[test_case("rBäôí" => true ; "alpha")]
    #[test_case("_" => true ; "allowed special")]
    #[test_case("" => false ; "empty")]
    #[test_case("0123456789" => false ; "numeric")]
    #[test_case("!?-:\\,.-<>#" => false ; "unallowed special")]
    #[test_case(" " => false ; "space")]
    fn check_pn_chars_u(to_check: &str) -> bool {
        PN_CHARS_U.is_match(to_check)
    }

    #[test_case("rBäôí" => true ; "alpha")]
    #[test_case("_-" => true ; "allowed special")]
    #[test_case("" => false ; "empty")]
    #[test_case("0123456789" => true ; "numeric")]
    #[test_case("!?:\\,.<>#" => false ; "unallowed special")]
    #[test_case(" " => false ; "space")]
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
    #[test_case(" " => false ; "space")]
    fn check_pn_prefix(to_check: &str) -> bool {
        PN_PREFIX.is_match(to_check)
    }

    #[test_case("@en" => true ; "simple")]
    #[test_case("@en-uk" => true ; "expanded")]
    #[test_case("@en-uk-man" => true ; "further")]
    #[test_case("en-uk-man" => false ; "missing at")]
    #[test_case("@1en-uk-man" => false ; "number in first")]
    #[test_case("@en-2uk2-man" => true ; "number in second")]
    #[test_case("@en-" => true ; "no extension")]
    #[test_case(" " => false ; "space")]
    fn check_langtag(to_check: &str) -> bool {
        LANGTAG.is_match(to_check)
    }

    #[test_case("123"        => true ; "integer")]
    #[test_case("-123"       => true ; "ninteger")]
    #[test_case("123.45"     => true ; "decimal")]
    #[test_case("-123.45"    => true ; "ndecimal")]
    #[test_case(".45"        => false ; "decimal dot")]
    #[test_case("-.45"       => false ; "ndecimal dot")]
    #[test_case("1.2345e2"   => true ; "double")]
    #[test_case("-12345E-2"  => true ; "ndouble")]
    #[test_case("-.12345E-2" => false ; "ndouble dot")]
    #[test_case(" " => false ; "space")]
    fn check_integer(to_check: &str) -> bool {
        INTEGER.is_match(to_check)
    }

    #[test_case("123"        => false ; "integer")]
    #[test_case("-123"       => false ; "ninteger")]
    #[test_case("123.45"     => true ; "decimal")]
    #[test_case("-123.45"    => true ; "ndecimal")]
    #[test_case(".45"        => true ; "decimal dot")]
    #[test_case("-.45"       => true ; "ndecimal dot")]
    #[test_case("1.2345e2"   => true ; "double")]
    #[test_case("-12345E-2"  => false ; "ndouble")]
    #[test_case("-.12345E-2" => true ; "ndouble dot")]
    #[test_case(" " => false ; "space")]
    fn check_decimal(to_check: &str) -> bool {
        DECIMAL.is_match(to_check)
    }

    #[test_case("123"        => false ; "integer")]
    #[test_case("-123"       => false ; "ninteger")]
    #[test_case("123.45"     => false ; "decimal")]
    #[test_case("-123.45"    => false ; "ndecimal")]
    #[test_case(".45"        => false ; "decimal dot")]
    #[test_case("-.45"       => false ; "ndecimal dot")]
    #[test_case("1.2345e2"   => true ; "double")]
    #[test_case("-12345E-2"  => true ; "ndouble")]
    #[test_case("-.12345E-2" => true ; "ndouble dot")]
    #[test_case(" " => false ; "space")]
    fn check_double(to_check: &str) -> bool {
        DOUBLE.is_match(to_check)
    }

    #[test_case("_:example  rest" => Ok(("  rest", "_:example")) ; "start alpha")]
    #[test_case("_:0  rest" => Ok(("  rest", "_:0")) ; "start num")]
    #[test_case("_:_  rest" => Ok(("  rest", "_:_")) ; "start under")]
    fn check_blank_node_label(i: &str) -> IResult<&str, &str> {
        blank_node_label(i)
    }

    #[test_case(" \t\n\r" => true ; "valid spaces")]
    #[test_case("# some comment \n" => true ; "comment only")]
    #[test_case("\n# some comment \n\t" => true ; "embedded comment")]
    #[test_case("text" => false ; "no comment")]
    fn check_ws(to_check: &str) -> bool {
        WS.is_match(to_check)
    }

    #[test_case(" \t\n\r" => true ; "valid spaces")]
    #[test_case("# some comment \n" => true ; "comment only")]
    #[test_case("\n# some comment \n\t" => true ; "embedded comment")]
    #[test_case("text\n# some comment \n\ttext" => false ; "full embedded comment")]
    #[test_case("text" => false ; "no comment")]
    #[test_case("\n# some comment \n\ttext" => true ; "half embedded comment")]
    fn check_ws_many1(to_check: &str) -> bool {
        WS_MANY1.is_match(to_check)
    }

    #[test_case(" \t\n\r" => true ; "valid spaces")]
    #[test_case("# some comment \n" => true ; "comment only")]
    #[test_case("\n# some comment \n\t" => true ; "embedded comment")]
    #[test_case("text\n# some comment \n\ttext" => true ; "full embedded comment")]
    #[test_case("text" => true ; "no comment")]
    #[test_case("\n# some comment \n\ttext" => true ; "half embedded comment")]
    fn check_ws_many0(to_check: &str) -> bool {
        WS_MANY0.is_match(to_check)
    }

    #[test_case("[]" => true ; "no space")]
    #[test_case("[ \t\n]" => true ; "valid space")]
    #[test_case("[ \thello\n]" => false ; "not empty")]
    #[test_case(" \t\n" => false ; "no brackets")]
    fn check_anon(to_check: &str) -> bool {
        ANON.is_match(to_check)
    }

    #[test_case("%ab" => true ; "hex valid")]
    #[test_case("%yz" => false ; "hex invalid")]
    #[test_case("\\." => true ; "escape")]
    #[test_case("." => false ; "unescape")]
    #[test_case(" " => false ; "space")]
    fn check_plx(to_check: &str) -> bool {
        PLX.is_match(to_check)
    }

    #[test_case("%ab" => true ; "hex valid")]
    #[test_case("%yz" => false ; "hex invalid")]
    #[test_case(" " => false ; "space")]
    fn check_percent(to_check: &str) -> bool {
        PERCENT.is_match(to_check)
    }

    #[test_case("b" => true ; "valid")]
    #[test_case("z" => false ; "invalid")]
    #[test_case(" " => false ; "space")]
    fn check_hex(to_check: &str) -> bool {
        HEX.is_match(to_check)
    }

    #[test_case("\\." => true ; "escape")]
    #[test_case("." => false ; "unescape")]
    #[test_case(" " => false ; "space")]
    fn check_pn_local_esc(to_check: &str) -> bool {
        PN_LOCAL_ESC.is_match(to_check)
    }

    #[test_case("0" => true ; "zero")]
    #[test_case("5" => true ; "digit")]
    #[test_case("a" => false ; "hex")]
    #[test_case(" " => false ; "space")]
    fn check_digit(to_check: &str) -> bool {
        DIGIT.is_match(to_check)
    }
}
