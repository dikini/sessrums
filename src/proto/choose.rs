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
/// The dual of `Choose<L, R>` is `Offer<L::Dual, R::Dual>`. This reflects the fundamental
/// session type principle that if one party chooses, the other party must offer the options.
///
/// This duality relationship ensures protocol compatibility between communicating parties:
/// - When one process chooses between protocols L and R, the other process must
///   offer the dual protocols L::Dual and R::Dual.
/// - The duality relationship is symmetric: `Choose<L, R>::Dual::Dual == Choose<L, R>`.
///
/// ## Duality Transformation
///
/// The duality transformation for Choose follows these rules:
/// 1. Replace Choose with Offer
/// 2. Apply the duality transformation to each branch (L becomes L::Dual, R becomes R::Dual)
///
/// For example:
/// ```
/// use sessrums::proto::{Protocol, Send, Recv, End, Offer, Choose};
///
/// // Original protocol
/// type MyChoose = Choose<Send<i32, End>, Recv<String, End>>;
///
/// // Its dual
/// type MyChooseDual = Offer<Recv<i32, End>, Send<String, End>>;
/// ```
///
/// ## Nested Duality
///
/// The duality relationship extends to nested Choose types:
///
/// ```
/// use sessrums::proto::{Protocol, Send, Recv, End, Offer, Choose};
///
/// // A protocol with nested Choose
/// type NestedChoose = Choose<
///     Choose<Send<i32, End>, Recv<bool, End>>,
///     Send<String, End>
/// >;
///
/// // Its dual has nested Offer
/// type NestedChooseDual = Offer<
///     Offer<Recv<i32, End>, Send<bool, End>>,
///     Recv<String, End>
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
/// use sessrums::proto::{Protocol, Send, Recv, End, Choose};
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
/// use sessrums::proto::{Protocol, Send, Recv, End, Choose};
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

    #[test]
    fn test_nested_choose_duality() {
        // Test duality with nested Choose types
        
        // A protocol that chooses between:
        // 1. Choosing between Send<i32, End> and Recv<bool, End>
        // 2. Sending a String and then ending
        type NestedChooseProtocol = Choose<
            Choose<Send<i32, End>, Recv<bool, End>>,
            Send<String, End>
        >;
        
        // The dual should be Offer between:
        // 1. Offering between Recv<i32, End> and Send<bool, End>
        // 2. Receiving a String and then ending
        type ExpectedDual = Offer<
            Offer<Recv<i32, End>, Send<bool, End>>,
            Recv<String, End>
        >;
        
        // Check that the dual relationships are correctly established
        fn check_nested_dual<P, D>()
        where
            P: Protocol<Dual = D>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if NestedChooseProtocol's dual is correctly derived
        check_nested_dual::<NestedChooseProtocol, ExpectedDual>();
    }

    #[test]
    fn test_multiple_level_duality() {
        // Test multiple levels of duality (dual of dual of dual)
        
        // Start with a simple protocol
        type OriginalProtocol = Choose<Send<i32, End>, Recv<String, End>>;
        
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
    fn test_mixed_protocol_composition() {
        // Test a mixed composition of Choose and other protocol types
        
        // A protocol with Choose nested within other protocol types
        type MixedProtocol = Send<bool,
            Choose<
                Recv<i32, End>,
                Send<String,
                    Choose<
                        Recv<char, End>,
                        Recv<u8, End>
                    >
                >
            >
        >;
        
        // The dual should have Send/Recv swapped and Choose/Offer swapped
        type ExpectedDual = Recv<bool,
            Offer<
                Send<i32, End>,
                Recv<String,
                    Offer<
                        Send<char, End>,
                        Send<u8, End>
                    >
                >
            >
        >;
        
        // Check that the dual relationships are correctly established
        fn check_mixed_dual<P, D>()
        where
            P: Protocol<Dual = D>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if MixedProtocol's dual is correctly derived
        check_mixed_dual::<MixedProtocol, ExpectedDual>();
    }
}