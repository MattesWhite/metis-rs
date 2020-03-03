//! This module provides the configuration of serialization the supported
//! formats.

mod _indentation;
pub use self::_indentation::*;

use crate::common::Prolog;
use crate::error::{Error, Result};
use crate::Format;
use sophia::term::TermData;
use std::io;

/// Options to serialize format `F`.
#[derive(Clone, Debug)]
pub struct Config<F, TD>
where
    F: Format,
    TD: TermData,
{
    pub(crate) prolog: Prolog<F, TD>,
    pub(crate) indent: Indentation,
    pub(crate) space: Indentation,
    pub(crate) format: F::ConfigData,
}

impl<F, TD> Default for Config<F, TD>
where
    F: Format,
    F::ConfigData: Default,
    TD: TermData,
{
    fn default() -> Self {
        Self {
            prolog: Prolog::default(),
            indent: Indentation::default(),
            space: Indentation::space(),
            format: F::ConfigData::default(),
        }
    }
}

impl<F, TD> std::ops::Deref for Config<F, TD>
where
    F: Format,
    TD: TermData,
{
    type Target = Prolog<F, TD>;

    fn deref(&self) -> &Self::Target {
        &self.prolog
    }
}

impl<F, TD> std::ops::DerefMut for Config<F, TD>
where
    F: Format,
    TD: TermData,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.prolog
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
            prolog: Prolog::default(),
            indent: Indentation::default(),
            space: Indentation::space(),
            format,
        }
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
