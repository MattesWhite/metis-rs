//! This module provides the configuration of serialization the supported
//! formats.

mod _indentation;
pub use self::_indentation::*;

use super::{Error, Format, Result};
use sophia::term::TermData;
use std::collections::HashMap;
use std::io;

/// Options to serialize format `F`.
#[derive(Clone, Debug)]
pub struct Config<F, TD>
where
    F: Format,
    TD: TermData,
{
    pub(super) base: Option<TD>,
    pub(super) prefixes: HashMap<TD, TD>,
    pub(super) indent: Indentation,
    pub(super) space: Indentation,
    pub(super) format: F::ConfigData,
}

impl<F, TD> Default for Config<F, TD>
where
    F: Format,
    F::ConfigData: Default,
    TD: TermData,
{
    fn default() -> Self {
        Self {
            base: None,
            prefixes: HashMap::new(),
            indent: Indentation::default(),
            space: Indentation::space(),
            format: F::ConfigData::default(),
        }
    }
}

impl<F, TD> Config<F, TD>
where
    F: Format,
    TD: TermData,
    Self: Default,
{
    /// Create the default configuration with the default prefixes.
    ///
    /// Uses internally [`add_default_prefixes()`](#method.add_default_prefixes)
    pub fn with_default_prefixes() -> Self
    where
        TD: From<&'static str>,
    {
        let mut tsc = Self::default();
        tsc.add_default_prefixes();
        tsc
    }
}

impl<F, TD> Config<F, TD>
where
    F: Format,
    TD: TermData,
{
    /// A default config with the given `Format`.
    pub fn new(format: F::ConfigData) -> Self {
        Self {
            base: None,
            prefixes: HashMap::new(),
            indent: Indentation::default(),
            space: Indentation::space(),
            format,
        }
    }
    /// Set the base IRI.
    ///
    /// If set it is printed with the `@base` directive into the documents
    /// preamble.
    ///
    /// # Errors
    ///
    /// This method fails if base is not a valid prefix IRI.
    pub fn set_base<U>(&mut self, base: U) -> Result<&mut Self>
    where
        U: AsRef<str>,
        TD: From<U>,
    {
        if F::is_valid_ns(&base) {
            self.base = Some(base.into());
            Ok(self)
        } else {
            Err(Error::InvalidIri(base.as_ref().to_owned()))
        }
    }
    /// Removes the base IRI if it was set.
    ///
    /// In the default setting no base IRI is set.
    pub fn unset_base(&mut self) -> &mut Self {
        self.base = None;
        self
    }
    /// Read the current base IRI.
    ///
    /// If `None` is returned no base IRI is set.
    pub fn base(&self) -> &Option<TD> {
        &self.base
    }
    /// Add a prefix.
    ///
    /// # Error
    ///
    /// Checks if both prefix and namespace is valid
    pub fn add_prefix<P, N>(&mut self, p: P, ns: N) -> Result<&mut Self>
    where
        P: AsRef<str>,
        N: AsRef<str>,
        TD: From<P> + From<N>,
    {
        F::check_prefix_id(p, ns).map(|(p, ns)| {
            self.prefixes.insert(p.into(), ns.into());
            self
        })
    }
    /// Add the list of prefixes.
    ///
    /// # Error
    ///
    /// Checks if both prefixes and namespaces are valid
    pub fn add_prefixes<P, N>(
        &mut self,
        prefixes: impl Iterator<Item = (P, N)>,
    ) -> Result<&mut Self>
    where
        P: AsRef<str>,
        N: AsRef<str>,
        TD: From<P> + From<N>,
    {
        if let Some(Err(e)) = prefixes
            .map(|(p, ns)| {
                F::check_prefix_id(p, ns).map(|(p, ns)| self.prefixes.insert(p.into(), ns.into()))
            })
            .find(Result::is_err)
        {
            Err(e)
        } else {
            Ok(self)
        }
    }
    /// Add the list of prefixes.
    ///
    /// # Unsafe
    ///
    /// Neither checks for prefixes nor namespaces are done
    pub unsafe fn set_prefixes_unchecked<P, N>(
        &mut self,
        prefixes: impl Iterator<Item = (P, N)>,
    ) -> &mut Self
    where
        P: AsRef<str>,
        N: AsRef<str>,
        TD: From<P> + From<N>,
    {
        prefixes.for_each(|(p, ns)| {
            self.prefixes.insert(p.into(), ns.into());
        });
        self
    }
    /// Adds prefixes for `rdf`, `rdfs` and `xsd` namespaces.
    pub fn add_default_prefixes(&mut self) -> &mut Self
    where
        TD: From<&'static str>,
    {
        self.prefixes
            .insert("rdf".into(), sophia::ns::rdf::PREFIX.into());
        self.prefixes
            .insert("rdfs".into(), sophia::ns::rdfs::PREFIX.into());
        self.prefixes
            .insert("xsd".into(), sophia::ns::xsd::PREFIX.into());
        self
    }
    /// Deletes all prefixes.
    pub fn clear_prefixes(&mut self) -> &mut Self {
        self.prefixes.clear();
        self
    }
    /// Set the indentation for new rows.
    ///
    /// Is applied once for each level of indentation.
    pub fn set_indentation(&mut self, indent: Indentation) -> &mut Self {
        self.indent = indent;
        self
    }
    /// Writes the configured indentation to `target`.
    pub fn write_indent(&self, target: &mut impl io::Write) -> io::Result<()> {
        self.indent.serialize(target)
    }
    /// Set the spacing between terms.
    ///
    /// # Error
    ///
    /// Spacing must be set to at least one space.
    pub fn set_spacing(&mut self, space: Indentation) -> Result<&mut Self> {
        if space.is_empty() {
            Err(Error::InvalidSpacing)
        } else {
            self.space = space;
            Ok(self)
        }
    }
    /// Writes the configured space to `target`.
    pub fn write_space(&self, target: &mut impl io::Write) -> io::Result<()> {
        self.space.serialize(target)
    }
}
