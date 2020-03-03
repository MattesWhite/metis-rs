//! Structs and types to parse RDF documents

mod util;
pub use self::util::*;

pub mod n3;
pub mod turtle;

use crate::common::*;
use std::borrow::Cow;
use std::collections::VecDeque;

/// The current context of the parser.
#[derive(Debug)]
pub struct Context<'a, F>
where
    F: Format + Valid<Cow<'a, str>>,
{
    /// Prefixes and Base
    prolog: Prolog<F, Cow<'a, str>>,
    /// Number of parsed blank nodes. Used for naming anonymous nodes.
    bnode_cnt: usize,
    /// When a list is parsed its surrounding block is parsed first. The
    /// list's triples are stored and returned afterwards.
    triple_stack: VecDeque<[CowTerm<'a, F>; 3]>,
}

impl<'a, F> Default for Context<'a, F>
where
    F: Format + Valid<Cow<'a, str>>,
{
    fn default() -> Self {
        Self {
            prolog: Prolog::default(),
            bnode_cnt: 0,
            triple_stack: VecDeque::new(),
        }
    }
}

impl<'a, F> Context<'a, F>
where
    F: Format + Valid<Cow<'a, str>>,
{
    /// Similar to `Prolog`'s mwthod
    pub fn with_default_prefixes() -> Self {
        Self {
            prolog: Prolog::with_default_prefixes(),
            bnode_cnt: 0,
            triple_stack: VecDeque::new(),
        }
    }
    fn new_labeled_bnode(&mut self, label: &'a str) -> CowTerm<'a, F> {
        self.bnode_cnt += 1;
        CowTerm::<'a, F>::new_blank_node(label)
    }
    fn new_anon_bnode(&mut self) -> CowTerm<'a, F> {
        let bn = CowTerm::<'a, F>::new_blank_node(format!("anon{}", self.bnode_cnt));
        self.bnode_cnt += 1;
        bn
    }
    fn new_iri(&self, iri: &'a str) -> CowTerm<'a, F> {
        // TODO: Resolve properly with base
        CowTerm::<'a, F>::new_iri(iri)
    }
    fn pop_triple(&mut self) -> Option<[CowTerm<'a, F>; 3]> {
        self.triple_stack.pop_front()
    }
    fn push_triple(&mut self, triple: [CowTerm<'a, F>; 3]) {
        self.triple_stack.push_back(triple)
    }
    fn push_triples(&mut self, src: impl Iterator<Item = [CowTerm<'a, F>; 3]>) {
        self.triple_stack.extend(src);
    }
}

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
