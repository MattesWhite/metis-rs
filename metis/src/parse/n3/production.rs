//! Production rules of N3.

use super::CowTerm;
use crate::N3;
use crate::parse::{
    turtle::terminals as ttl_terminal,
    turtle::production as ttl_production,
    Context
};
use std::cell::RefCell;

/// A context wrapped in a RefCell.
///
/// This is necessary due to the constraints of `nom`'s parser generators (they
/// only take `Fn`).
pub type RefContext<'a> = RefCell<Context<'a, N3>>;
