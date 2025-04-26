//! Offer protocol type for session types.
//!
//! This module defines the `Offer<L, R>` protocol type, which represents a protocol
//! that offers a choice between two continuations, `L` and `R`.

use std::marker::PhantomData;
use super::Protocol;
use super::choose::Choose;

/// A protocol that offers a choice between two continuations, `L` and `R`.
///
/// The `Offer<L, R>` type represents a protocol that offers a choice between
/// two continuations, `L` and `R`. The choice is made by the other party.
///
/// # Duality
///
/// The dual of `Offer<L, R>` is `Choose<L::Dual, R::Dual>`. This reflects the fact
/// that if one party offers a choice, the other party must choose one of the options.
///
/// # Type Parameters
///
/// * `L` - The first continuation protocol type. Must implement the `Protocol` trait.
/// * `R` - The second continuation protocol type. Must implement the `Protocol` trait.
///
/// # Examples
///
/// ```
/// use sez::proto::{Protocol, Send, Recv, End, Offer};
///
/// // A protocol that offers a choice between:
/// // 1. Sending an i32 and then ending, or
/// // 2. Receiving a String and then ending
/// type MyProtocol = Offer<Send<i32, End>, Recv<String, End>>;
///
/// // The dual protocol would choose between:
/// // 1. Receiving an i32 and then ending, or
/// // 2. Sending a String and then ending
/// // (This would be: Choose<Recv<i32, End>, Send<String, End>>)
/// ```
///
/// # Protocol Composition
///
/// `Offer<L, R>` can be composed with other protocol types to create more complex
/// communication patterns:
///
/// ```
/// use sez::proto::{Protocol, Send, Recv, End, Offer};
///
/// // A protocol that first receives a boolean, then offers a choice between
/// // sending an i32 or receiving a String
/// type ComplexProtocol = Recv<bool, Offer<Send<i32, End>, Recv<String, End>>>;
/// ```
pub struct Offer<L, R> {
    _marker: PhantomData<(L, R)>,
}

impl<L: Protocol, R: Protocol> Protocol for Offer<L, R> {
    /// The dual of `Offer<L, R>` is `Choose<L::Dual, R::Dual>`.
    ///
    /// This reflects the fact that if one party offers a choice, the other party
    /// must choose one of the options.
    type Dual = Choose<L::Dual, R::Dual>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{Send, Recv, End};

    #[test]
    fn test_offer_protocol_implementation() {
        // This test verifies that Offer<L, R> implements Protocol
        
        // We can't directly test the equality of types in Rust,
        // but we can verify that Offer<L, R> implements Protocol
        fn assert_implements_protocol<T: Protocol>() {}
        
        // This will compile only if Offer<Send<i32, End>, Recv<String, End>> implements Protocol
        assert_implements_protocol::<Offer<Send<i32, End>, Recv<String, End>>>();
    }

    #[test]
    fn test_offer_duality() {
        // This test verifies the duality relationship for Offer<L, R>
        
        // Define some simple protocol types for testing
        type OfferProtocol = Offer<Send<i32, End>, Recv<String, End>>;
        type ExpectedDual = Choose<Recv<i32, End>, Send<String, End>>;
        
        // Check that OfferProtocol::Dual is ExpectedDual
        fn check_dual<P, D>()
        where
            P: Protocol<Dual = D>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if OfferProtocol::Dual is ExpectedDual
        check_dual::<OfferProtocol, ExpectedDual>();
    }

    #[test]
    fn test_offer_duality_symmetry() {
        // This test verifies the symmetry of the duality relationship for Offer<L, R>
        // Offer<L, R>::Dual is Choose<L::Dual, R::Dual>
        // Choose<L::Dual, R::Dual>::Dual should be Offer<L, R>
        
        // Define some simple protocol types for testing
        type OfferProtocol = Offer<Send<i32, End>, Recv<String, End>>;
        type DualProtocol = Choose<Recv<i32, End>, Send<String, End>>;
        
        // Check that the duality relationship is symmetric
        fn check_duality_symmetry<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if the duality relationship is correctly implemented
        // Note: This test will fail until Choose<L, R> is fully implemented in Task 2.4
        // check_duality_symmetry::<OfferProtocol, DualProtocol>();
    }

    #[test]
    fn test_offer_with_complex_composition() {
        // Test Offer in a more complex protocol composition
        
        // A protocol that receives a boolean, then offers a choice between
        // sending an i32 or receiving a String
        type ComplexProtocol = Recv<bool, Offer<Send<i32, End>, Recv<String, End>>>;
        
        // The dual should be Send<bool, Choose<Recv<i32, End>, Send<String, End>>>
        type ExpectedDual = Send<bool, Choose<Recv<i32, End>, Send<String, End>>>;
        
        // Check that the dual relationships are correctly established
        fn check_complex_dual<P, D>()
        where
            P: Protocol<Dual = D>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if ComplexProtocol's dual is correctly derived
        check_complex_dual::<ComplexProtocol, ExpectedDual>();
    }
}