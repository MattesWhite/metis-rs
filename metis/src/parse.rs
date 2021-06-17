//! Structs and types to parse RDF documents

mod util;
pub use self::util::*;
mod error;
pub use self::error::*;

// later...
// pub mod n3;
pub mod turtle;

use crate::common::*;
use sophia::term::{blank_node::BlankNode, iri::Iri, mown_str::MownStr};
use std::collections::VecDeque;

/// The current context of the parser.
#[derive(Debug)]
pub struct Context<'td, F>
where
    F: Format + Valid<MownStr<'td>>,
{
    /// Prefixes and Base
    prolog: Prolog<'td, F>,
    /// Number of parsed blank nodes. Used for naming anonymous nodes.
    bnode_cnt: usize,
    /// When a list is parsed its surrounding block is parsed first. The
    /// list's triples are stored and returned afterwards.
    triple_stack: VecDeque<[MownTerm<'td, F>; 3]>,
}

impl<'td, F> Default for Context<'td, F>
where
    F: Format + Valid<MownStr<'td>>,
{
    fn default() -> Self {
        Self {
            prolog: Prolog::default(),
            bnode_cnt: 0,
            triple_stack: VecDeque::new(),
        }
    }
}

impl<'td, F> Context<'td, F>
where
    F: Format + Valid<MownStr<'td>>,
{
    /// Similar to `Prolog`'s method
    pub fn with_default_prefixes() -> Self {
        Self {
            prolog: Prolog::with_default_prefixes(),
            bnode_cnt: 0,
            triple_stack: VecDeque::new(),
        }
    }
    fn new_labeled_bnode(&mut self, label: &'td str) -> BlankNode<MownStr<'td>> {
        self.bnode_cnt += 1;
        // Should be ensured by parser.
        BlankNode::<MownStr<'td>>::new_unchecked(label)
    }
    fn new_anon_bnode(&mut self) -> BlankNode<MownStr<'td>> {
        let label = format!("anon{}", self.bnode_cnt);
        self.bnode_cnt += 1;
        BlankNode::<MownStr<'td>>::new_unchecked(label).into()
    }
    fn new_iri(&self, iri: MownStr<'td>) -> Iri<MownStr<'td>> {
        let iri = Iri::<MownStr<'td>>::new(iri).expect("IRI is valid through parser");
        self.prolog.resolve(&iri)
    }
    fn pop_triple(&mut self) -> Option<[MownTerm<'td, F>; 3]> {
        self.triple_stack.pop_front()
    }
    fn push_triple(&mut self, triple: [MownTerm<'td, F>; 3]) {
        self.triple_stack.push_back(triple)
    }
    fn push_triples(&mut self, src: impl Iterator<Item = [MownTerm<'td, F>; 3]>) {
        self.triple_stack.extend(src);
    }
}

/// Get the given string without a left and right margin of characters.
///
/// # Example
///
/// ```
/// let i = "123454321";
/// assert_eq!(unwrap_str(i, 1), "2345432");
/// assert_eq!(unwrap_str(i, 3), "454");
/// ```
#[inline]
fn unwrap_str(i: &str, margin: usize) -> &str {
    &i[margin..i.len() - margin]
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case("12345a54321", 0 => "12345a54321" ; "margin 0")]
    #[test_case("12345a54321", 1 =>  "2345a5432" ; "margin 1")]
    #[test_case("12345a54321", 2 =>   "345a543" ; "margin 2")]
    fn check_unwrap_str(i: &str, margin: usize) -> &str {
        unwrap_str(i, margin)
    }
}
