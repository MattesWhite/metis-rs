//! The prolog of prefixes and base typical for Notation3-derived serialization
//! formats.

use crate::error::{Error, Result};
use crate::Format;
use sophia_term::{
    iri::{Iri, IriParsed, Resolve},
    mown_str::MownStr,
    ns::Namespace,
    TermData, TermError,
};
use std::collections::HashMap;
use std::marker::PhantomData;

/// Result of matching an IRI against the base and prefixes of a `Prolog`.
pub enum PrologMatch<'a, I> {
    /// Nothing matched.
    NoMatch,
    /// The base IRI matched. Contains the rest of the matched IRI.
    Base(I),
    /// A prefix matches. Contains the prefix and the rest of the matched IRI.
    Prefix(&'a str, I),
}

/// Options to serialize format `F`.
#[derive(Clone, Debug)]
pub struct Prolog<'td, F>
where
    F: Format,
{
    _f: PhantomData<F>,
    pub(crate) base: Option<(Iri<MownStr<'td>>, IriParsed<'static>)>,
    pub(crate) prefixes: HashMap<&'td str, Namespace<MownStr<'td>>>,
}

impl<'td, F> Default for Prolog<'td, F>
where
    F: Format,
{
    /// A default prolog is completely empty.
    ///
    /// Neither base nor prefixes.
    fn default() -> Self {
        Self {
            _f: PhantomData,
            base: None,
            prefixes: HashMap::new(),
        }
    }
}

impl<'td, F> Prolog<'td, F>
where
    F: Format,
{
    /// Create the default configuration with the default prefixes.
    ///
    /// Uses internally [`add_default_prefixes()`](#method.add_default_prefixes)
    pub fn with_default_prefixes() -> Self
    where
        Self: Default,
    {
        let mut tsc = Self::default();
        tsc.add_default_prefixes();
        tsc
    }
    /// Set the base IRI.
    ///
    /// If set it is printed with the `@base` directive into the document's
    /// preamble.
    pub fn set_base(&mut self, base: Iri<MownStr<'td>>) -> Result<&mut Self, TermError> {
        if base.has_suffix() {
            return Err(TermError::InvalidIri(
                "Base IRIs must not be suffixed".to_owned(),
            ));
        }

        let base = if let Some((_, o_base)) = self.base {
            o_base.resolve(&base)
        } else {
            base
        };

        let mut buf = String::new();
        let parsed = base.parse_components(&mut buf);
        // (Mostly) safe as always altered together with origin `base`
        // TODO: This is not safe if a panic occurs => use third-party crate!
        let parsed = unsafe { std::mem::transmute(parsed) };

        self.base.replace((base, parsed));
        Ok(self)
    }
    /// Removes the base IRI if it was set.
    ///
    /// In the default setting no base IRI is set.
    pub fn unset_base(&mut self) -> &mut Self {
        self.base.take();
        self
    }
    /// Read the current base IRI.
    ///
    /// If `None` is returned no base IRI is set.
    pub fn base(&self) -> Option<&Iri<MownStr<'td>>> {
        self.base.as_ref().map(|(iri, _)| iri)
    }
    /// Add a prefix.
    ///
    /// # Error
    ///
    /// Checks if prefix is valid.
    pub fn add_prefix(&mut self, p: &'td str, ns: MownStr<'td>) -> Result<&mut Self> {
        if F::is_valid_prefix(&p) {
            let ns = Namespace::new(ns)?;
            let ns = self.resolve(&ns);
            self.prefixes.insert(p, ns);
            Ok(self)
        } else {
            Err(Error::InvalidPrefix(p.to_string()))
        }
    }
    /// Add the list of prefixes.
    ///
    /// # Error
    ///
    /// Checks if prefixes are valid.
    pub fn add_prefixes<I>(&mut self, prefixes: I) -> Result<&mut Self>
    where
        I: Iterator<Item = (&'td str, MownStr<'td>)>,
    {
        if let Some(Err(e)) = prefixes
            .map(|(p, ns)| self.add_prefix(p, ns).map(|_| ()))
            .find(Result::is_err)
        {
            Err(e)
        } else {
            Ok(self)
        }
    }
    /// Adds prefixes for `rdf`, `rdfs` and `xsd` namespaces.
    pub fn add_default_prefixes(&mut self) -> &mut Self {
        self.prefixes.insert(
            "rdf",
            Namespace::new(sophia::ns::rdf::PREFIX.into()).unwrap(),
        );
        self.prefixes.insert(
            "rdfs",
            Namespace::new(sophia::ns::rdfs::PREFIX.into()).unwrap(),
        );
        self.prefixes.insert(
            "xsd",
            Namespace::new(sophia::ns::xsd::PREFIX.into()).unwrap(),
        );
        self
    }
    /// Deletes all prefixes.
    pub fn clear_prefixes(&mut self) -> &mut Self {
        self.prefixes.clear();
        self
    }
    /// Searches for a matching prefix.
    ///
    /// If one matches the prefix (without `:`) and the rest of the target are
    /// returned.
    pub fn matches<'t, TD2: TermData>(
        &'t self,
        target: &'t Iri<TD2>,
    ) -> PrologMatch<'t, impl 't + Iterator<Item = char>> {
        if let Some((base, _)) = &self.base {
            if let Some(iter) = target.match_ns(base) {
                return PrologMatch::Base(iter);
            }
        }

        if let Some(matched) = self.prefixes.iter().find_map(|(p, ns)| {
            let ns = ns.clone().into();
            target
                .match_ns(&ns)
                .map(|iter| PrologMatch::Prefix(p, iter))
        }) {
            matched
        } else {
            PrologMatch::NoMatch
        }
    }

    /// Resolves against the base IRI or returns unchanged if no base IRI is
    /// set.
    pub fn resolve<'i, I, O>(&self, other: &'i I) -> O
    where
        IriParsed<'td>: Resolve<&'i I, O>,
        I: Clone,
        O: From<I>,
    {
        if let Some((_, base)) = self.base {
            base.resolve(other)
        } else {
            other.clone().into()
        }
    }
}
