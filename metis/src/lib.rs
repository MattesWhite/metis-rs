#![deny(missing_docs)]

//! # Metis
//!
//! Implementation of Notation 3 and related technology conforming to `sophia`'s API.

pub mod common;
pub mod error;
pub mod parse;
pub mod serialize;

pub mod n3;
pub mod turtle;

pub use self::common::Format;
