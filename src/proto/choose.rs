//! Choose protocol type for session types.
//!
//! This module defines the `Choose<L, R>` protocol type, which represents a protocol
//! that chooses between two continuations, `L` and `R`.
//!
//! Note: This is a placeholder implementation. The full implementation will be done in Task 2.4.

use std::marker::PhantomData;
use super::Protocol;

/// A protocol that chooses between two continuations, `L` and `R`.
///
/// The `Choose<L, R>` type represents a protocol that chooses between
/// two continuations, `L` and `R`. The choice is made by this party.
///
/// This is a placeholder implementation. The full implementation will be done in Task 2.4.
pub struct Choose<L, R> {
    _marker: PhantomData<(L, R)>,
}

// Minimal implementation of Protocol for Choose<L, R> to support Offer<L, R>
// The full implementation will be done in Task 2.4.
impl<L: Protocol, R: Protocol> Protocol for Choose<L, R> {
    /// The dual of `Choose<L, R>` is `Offer<L::Dual, R::Dual>`.
    ///
    /// This reflects the fact that if one party chooses, the other party
    /// must offer the options.
    type Dual = super::offer::Offer<L::Dual, R::Dual>;
}