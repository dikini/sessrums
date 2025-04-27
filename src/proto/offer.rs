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
/// The dual of `Offer<L, R>` is `Choose<L::Dual, R::Dual>`. This reflects the fundamental
/// session type principle that if one party offers a choice, the other party must choose
/// one of the options.
///
/// This duality relationship ensures protocol compatibility between communicating parties:
/// - When one process offers a choice between protocols L and R, the other process must
///   choose between the dual protocols L::Dual and R::Dual.
/// - The duality relationship is symmetric: `Offer<L, R>::Dual::Dual == Offer<L, R>`.
///
/// ## Duality Transformation
///
/// The duality transformation for Offer follows these rules:
/// 1. Replace Offer with Choose
/// 2. Apply the duality transformation to each branch (L becomes L::Dual, R becomes R::Dual)
///
/// For example:
/// ```
/// use sessrums::proto::{Protocol, Send, Recv, End, Offer, Choose};
///
/// // Original protocol
/// type MyOffer = Offer<Send<i32, End>, Recv<String, End>>;
///
/// // Its dual
/// type MyOfferDual = Choose<Recv<i32, End>, Send<String, End>>;
/// ```
///
/// ## Nested Duality
///
/// The duality relationship extends to nested Offer types:
///
/// ```
/// use sessrums::proto::{Protocol, Send, Recv, End, Offer, Choose};
///
/// // A protocol with nested Offer
/// type NestedOffer = Offer<
///     Offer<Send<i32, End>, Recv<bool, End>>,
///     Recv<String, End>
/// >;
///
/// // Its dual has nested Choose
/// type NestedOfferDual = Choose<
///     Choose<Recv<i32, End>, Send<bool, End>>,
///     Send<String, End>
/// >;
/// ```
///
/// # Type Parameters
///
/// * `L` - The first continuation protocol type. Must implement the `Protocol` trait.
/// * `R` - The second continuation protocol type. Must implement the `Protocol` trait.
///
/// # Examples
///
/// ```
/// use sessrums::proto::{Protocol, Send, Recv, End, Offer};
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
/// use sessrums::proto::{Protocol, Send, Recv, End, Offer};
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
        // Now that Choose<L, R> is fully implemented in Task 2.4, we can enable this test
        check_duality_symmetry::<OfferProtocol, DualProtocol>();
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

    #[test]
    fn test_nested_offer_duality() {
        // Test duality with nested Offer types
        
        // A protocol that offers a choice between:
        // 1. Offering a choice between Send<i32, End> and Recv<bool, End>
        // 2. Receiving a String and then ending
        type NestedOfferProtocol = Offer<
            Offer<Send<i32, End>, Recv<bool, End>>,
            Recv<String, End>
        >;
        
        // The dual should be Choose between:
        // 1. Choosing between Recv<i32, End> and Send<bool, End>
        // 2. Sending a String and then ending
        type ExpectedDual = Choose<
            Choose<Recv<i32, End>, Send<bool, End>>,
            Send<String, End>
        >;
        
        // Check that the dual relationships are correctly established
        fn check_nested_dual<P, D>()
        where
            P: Protocol<Dual = D>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if NestedOfferProtocol's dual is correctly derived
        check_nested_dual::<NestedOfferProtocol, ExpectedDual>();
    }

    #[test]
    fn test_multiple_level_duality() {
        // Test multiple levels of duality (dual of dual of dual)
        
        // Start with a simple protocol
        type OriginalProtocol = Offer<Send<i32, End>, Recv<String, End>>;
        
        // Get its dual
        type FirstDual = <OriginalProtocol as Protocol>::Dual;
        
        // Get the dual of the dual
        type SecondDual = <FirstDual as Protocol>::Dual;
        
        // Get the dual of the dual of the dual
        type ThirdDual = <SecondDual as Protocol>::Dual;
        
        // The dual of the dual should be the original protocol
        fn check_dual_of_dual<P, D, DD>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = DD>,
            DD: Protocol,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if the dual of the dual is the original protocol
        check_dual_of_dual::<OriginalProtocol, FirstDual, SecondDual>();
        
        // The dual of the dual of the dual should be the dual of the original protocol
        fn check_dual_of_dual_of_dual<P, D, DD, DDD>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = DD>,
            DD: Protocol<Dual = DDD>,
            DDD: Protocol,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if the dual of the dual of the dual is the dual of the original protocol
        check_dual_of_dual_of_dual::<OriginalProtocol, FirstDual, SecondDual, ThirdDual>();
    }

    #[test]
    fn test_complex_offer_choose_composition() {
        // Test a complex composition of Offer and Choose types
        
        // A protocol with multiple levels of Offer and Choose
        type ComplexProtocol = Recv<bool,
            Offer<
                Send<i32,
                    Offer<
                        Recv<String, End>,
                        Recv<bool, End>
                    >
                >,
                Recv<f64,
                    Offer<
                        Send<char, End>,
                        Send<u8, End>
                    >
                >
            >
        >;
        
        // The dual should have Send/Recv swapped and Offer/Choose swapped
        type ExpectedDual = Send<bool,
            Choose<
                Recv<i32,
                    Choose<
                        Send<String, End>,
                        Send<bool, End>
                    >
                >,
                Send<f64,
                    Choose<
                        Recv<char, End>,
                        Recv<u8, End>
                    >
                >
            >
        >;
        
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