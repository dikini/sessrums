//! Choose protocol type for session types.
//!
//! This module defines the `Choose<L, R>` protocol type, which represents a protocol
//! that chooses between two continuations, `L` and `R`.

use std::marker::PhantomData;
use super::Protocol;
use super::offer::Offer;

/// A protocol that chooses between two continuations, `L` and `R`.
///
/// The `Choose<L, R>` type represents a protocol that chooses between
/// two continuations, `L` and `R`. The choice is made by this party.
///
/// # Duality
///
/// The dual of `Choose<L, R>` is `Offer<L::Dual, R::Dual>`. This reflects the fact
/// that if one party chooses, the other party must offer the options.
///
/// # Type Parameters
///
/// * `L` - The first continuation protocol type. Must implement the `Protocol` trait.
/// * `R` - The second continuation protocol type. Must implement the `Protocol` trait.
///
/// # Examples
///
/// ```
/// use sez::proto::{Protocol, Send, Recv, End, Choose};
///
/// // A protocol that chooses between:
/// // 1. Sending an i32 and then ending, or
/// // 2. Receiving a String and then ending
/// type MyProtocol = Choose<Send<i32, End>, Recv<String, End>>;
///
/// // The dual protocol would offer:
/// // 1. Receiving an i32 and then ending, or
/// // 2. Sending a String and then ending
/// // (This would be: Offer<Recv<i32, End>, Send<String, End>>)
/// ```
///
/// # Protocol Composition
///
/// `Choose<L, R>` can be composed with other protocol types to create more complex
/// communication patterns:
///
/// ```
/// use sez::proto::{Protocol, Send, Recv, End, Choose};
///
/// // A protocol that first sends a boolean, then chooses between
/// // sending an i32 or receiving a String
/// type ComplexProtocol = Send<bool, Choose<Send<i32, End>, Recv<String, End>>>;
/// ```
pub struct Choose<L, R> {
    _marker: PhantomData<(L, R)>,
}

impl<L: Protocol, R: Protocol> Protocol for Choose<L, R> {
    /// The dual of `Choose<L, R>` is `Offer<L::Dual, R::Dual>`.
    ///
    /// This reflects the fact that if one party chooses, the other party
    /// must offer the options.
    type Dual = Offer<L::Dual, R::Dual>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{Send, Recv, End};

    #[test]
    fn test_choose_protocol_implementation() {
        // This test verifies that Choose<L, R> implements Protocol
        
        // We can't directly test the equality of types in Rust,
        // but we can verify that Choose<L, R> implements Protocol
        fn assert_implements_protocol<T: Protocol>() {}
        
        // This will compile only if Choose<Send<i32, End>, Recv<String, End>> implements Protocol
        assert_implements_protocol::<Choose<Send<i32, End>, Recv<String, End>>>();
    }

    #[test]
    fn test_choose_duality() {
        // This test verifies the duality relationship for Choose<L, R>
        
        // Define some simple protocol types for testing
        type ChooseProtocol = Choose<Send<i32, End>, Recv<String, End>>;
        type ExpectedDual = Offer<Recv<i32, End>, Send<String, End>>;
        
        // Check that ChooseProtocol::Dual is ExpectedDual
        fn check_dual<P, D>()
        where
            P: Protocol<Dual = D>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if ChooseProtocol::Dual is ExpectedDual
        check_dual::<ChooseProtocol, ExpectedDual>();
    }

    #[test]
    fn test_choose_duality_symmetry() {
        // This test verifies the symmetry of the duality relationship for Choose<L, R>
        // Choose<L, R>::Dual is Offer<L::Dual, R::Dual>
        // Offer<L::Dual, R::Dual>::Dual should be Choose<L, R>
        
        // Define some simple protocol types for testing
        type ChooseProtocol = Choose<Send<i32, End>, Recv<String, End>>;
        type DualProtocol = Offer<Recv<i32, End>, Send<String, End>>;
        
        // Check that the duality relationship is symmetric
        fn check_duality_symmetry<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if the duality relationship is correctly implemented
        check_duality_symmetry::<ChooseProtocol, DualProtocol>();
    }

    #[test]
    fn test_choose_with_complex_composition() {
        // Test Choose in a more complex protocol composition
        
        // A protocol that sends a boolean, then chooses between
        // sending an i32 or receiving a String
        type ComplexProtocol = Send<bool, Choose<Send<i32, End>, Recv<String, End>>>;
        
        // The dual should be Recv<bool, Offer<Recv<i32, End>, Send<String, End>>>
        type ExpectedDual = Recv<bool, Offer<Recv<i32, End>, Send<String, End>>>;
        
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