use std::io;

/// Maximal number of allowed spaces to indent.
pub const MAX_SPACES: usize = 32;
/// Default number of spaces to indent.
///
/// Used by [`Indentation::default()`](enum.Indentation.html#method.default).
pub const DEFAULT_SPACES: u8 = 4;
/// Utility constant for serializing indentation.
const SPACES: [u8; MAX_SPACES] = [b' '; MAX_SPACES];

/// Determines how indentation is done.
///
/// It specifies the indentation for one level, i.e. indentation at level 3
/// applies the indentation 3 times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indentation {
    /// Indent by spaces.
    Spaces(u8),
    /// Indent by a tab (`\t`),
    Tab,
    /// No indentation.
    None,
}

impl Default for Indentation {
    /// The default is to indent [`DEFAULT_SPACES`](constant.DEFAULT_SPACES.html).
    fn default() -> Self {
        Indentation::Spaces(DEFAULT_SPACES)
    }
}

impl Indentation {
    /// Indentation by 1 space.
    pub fn space() -> Self {
        Self::Spaces(1)
    }
    /// Indentation by 2 spaces.
    pub fn spaces_2() -> Self {
        Self::Spaces(2)
    }
    /// Indentation by 4 spaces.
    pub fn spaces_4() -> Self {
        Self::Spaces(4)
    }
    /// Indentation by 8 spaces.
    pub fn spaces_8() -> Self {
        Self::Spaces(8)
    }
    /// Indentation by 16 spaces.
    pub fn spaces_16() -> Self {
        Self::Spaces(16)
    }
    /// Indentation by a custom number of spaces.
    ///
    /// # Error
    ///
    /// The number must be smaller than [`MAX_SPACES`](constant.MAX_SPACES.html).
    pub fn spaces_custom(times: u8) -> Result<Self, super::Error> {
        if times <= MAX_SPACES as u8 {
            Ok(Self::Spaces(times))
        } else {
            Err(super::Error::ToMuchSpaces(times))
        }
    }
    /// Indentation by tab (`\t`).
    pub fn tab() -> Self {
        Self::Tab
    }
    /// No indentation.
    pub fn none() -> Self {
        Self::None
    }
    /// `true` is the serialization results in no change, i.e. nothing is written.
    pub fn is_empty(self) -> bool {
        match self {
            Self::None | Self::Spaces(0) => true,
            Self::Spaces(_) | Self::Tab => false,
        }
    }

    /// Apply the indentation to the writer.
    pub fn serialize(self, w: &mut impl io::Write) -> io::Result<()> {
        match self {
            Indentation::Spaces(n) => w.write_all(&SPACES[0..n as usize]),
            Indentation::Tab => {
                w.write_all(b"\t")?;
                Ok(())
            }
            Indentation::None => Ok(()),
        }
    }
}
