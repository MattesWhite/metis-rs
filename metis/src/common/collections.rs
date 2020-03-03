//! Collections of terms.

use super::*;

/// A list of RDF terms
pub type TermList<'a, F> = Vec<CowTerm<'a, F>>;

/// A predicate with a list of objects
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoList<'a, F>
where
    F: Format + Valid<Cow<'a, str>>,
{
    predicate: CowTerm<'a, F>,
    objects: TermList<'a, F>,
}

impl<'a, F> PoList<'a, F>
where
    F: Format + Valid<Cow<'a, str>>,
{
    /// Creates a new PO-list from a predicate and a list of objects.
    pub fn new(predicate: CowTerm<'a, F>, objects: TermList<'a, F>) -> Self {
        Self { predicate, objects }
    }
    /// Returns the list of predicates and objects.
    ///
    /// This consumes the list. For each pair the predicate is copied.
    #[allow(clippy::should_implement_trait)] // TODO: Implement IntoIterator
    pub fn into_iter(self) -> impl Iterator<Item = (CowTerm<'a, F>, CowTerm<'a, F>)> {
        let p = self.predicate;
        self.objects.into_iter().map(move |o| (p.clone(), o))
    }
    /// Returns the list of predicates and objects by reference.
    pub fn iter(&self) -> impl Iterator<Item = (&CowTerm<'a, F>, &CowTerm<'a, F>)> {
        self.objects.iter().map(move |o| (&self.predicate, o))
    }
}

/// A subject with a list of predicate-object-lists
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpoList<'a, F>
where
    F: Format + Valid<Cow<'a, str>>,
{
    subject: CowTerm<'a, F>,
    po_lists: Vec<PoList<'a, F>>,
}

impl<'a, F> SpoList<'a, F>
where
    F: Format + Valid<Cow<'a, str>>,
{
    /// Creates a new PO-list from a predicate and a list of objects.
    pub fn new(subject: CowTerm<'a, F>, po_lists: Vec<PoList<'a, F>>) -> Self {
        Self { subject, po_lists }
    }
    /// Returns the list of subject, predicates and objects.
    ///
    /// This consumes the list. For each triple the subject and predicate are
    /// copied!
    #[allow(clippy::should_implement_trait)] // TODO: Implement IntoIterator
    pub fn into_iter(self) -> impl Iterator<Item = [CowTerm<'a, F>; 3]> {
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
    pub fn iter(&self) -> impl Iterator<Item = [&CowTerm<'a, F>; 3]> {
        self.po_lists
            .iter()
            .map(move |pol| pol.iter().map(move |(p, o)| [&self.subject, p, o]))
            .flatten()
    }
}
