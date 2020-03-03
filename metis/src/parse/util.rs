//! Utility to make parsing easier.

use nom::{error::ErrorKind, error_position, Err as NomError, IResult};
use regex::Regex;

/// Tries to capture the given regex.
///
/// The leftmost match is returned. The input is consumed to the end of the
/// match.
///
/// # Skipped content
///
/// The way regular expressions work it is possible that this parser skipps
/// significant content. It is recommended to start regexes with '^' to prevent
/// this.
pub fn parse_regex(re: &'static Regex) -> impl Fn(&str) -> IResult<&str, &str> {
    move |i: &str| {
        if let Some(found) = re.find(i) {
            let captured = found.as_str();
            let end = found.end();

            Ok((&i[end..], captured))
        } else {
            Err(NomError::Error(error_position!(
                i,
                ErrorKind::RegexpCapture
            )))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref AB: Regex = Regex::new(r#"a+b"#).unwrap();
    }

    #[test]
    fn check_regex() {
        let parser = parse_regex(&AB);
        let (r, f) = parser("habt").unwrap();
        assert_eq!(f, "ab");
        assert_eq!(r, "t");

        let (r, f) = parser("taabaabt").unwrap();
        assert_eq!(f, "aab");
        assert_eq!(r, "aabt");
        let (r, f) = parser(r).unwrap();
        assert_eq!(f, "aab");
        assert_eq!(r, "t");

        assert!(parser("tt").is_err());
    }
}
