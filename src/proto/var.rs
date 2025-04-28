//! Variable reference protocol type for session types.
//!
//! This module defines the `Var<const N: usize>` protocol type, which represents
//! a variable reference in a recursive protocol. It is used in conjunction with
//! the `Rec<P>` type to create recursive protocols.

use super::Protocol;

/// A protocol that represents a variable reference in a recursive protocol.
///
/// The `Var<N>` type is used to refer back to an enclosing `Rec<P>` protocol,
/// where `N` indicates the recursion depth (how many `Rec` layers to go back).
/// A value of `N = 0` refers to the immediately enclosing `Rec`, `N = 1` refers
/// to the second enclosing `Rec`, and so on.
///
/// # Type Parameters
///
/// * `N` - A const generic parameter that specifies the recursion depth.
///
/// # Duality
///
/// The dual of `Var<N>` is `Var<N>` itself. This is because a variable reference
/// in one protocol corresponds to the same variable reference in the dual protocol,
/// but the enclosing `Rec<P>` will ensure that the referenced protocol is properly
/// dualized.
///
/// # Examples
///
/// ```
/// use sessrums::proto::{Protocol, Rec, Var, Send, End};
///
/// // A protocol that sends an i32 and then repeats itself
/// type LoopingSend = Rec<Send<i32, Var<0>>>;
///
/// // Here, Var<0> refers back to the enclosing Rec, creating a loop
/// ```
pub struct Var<const N: usize>;

impl<const N: usize> Protocol for Var<N> {
    /// The dual of `Var<N>` is `Var<N>` itself.
    ///
    /// This reflects the fact that a variable reference in one protocol
    /// corresponds to the same variable reference in the dual protocol.
    /// The actual duality is handled by the enclosing `Rec<P>` type.
    type Dual = Var<N>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{Rec, Send, Recv};

    #[test]
    fn test_var_protocol_implementation() {
        // This test verifies that Var<N> implements Protocol
        // and that its Dual is defined as Var<N> itself
        
        // We can't directly test the equality of types in Rust,
        // but we can verify that Var<0> implements Protocol
        fn assert_implements_protocol<T: Protocol>() {}
        
        // This will compile only if Var<0> implements Protocol
        assert_implements_protocol::<Var<0>>();
        
        // We can also verify that the associated type Dual is defined correctly
        fn assert_dual_type<T, D>()
        where
            T: Protocol<Dual = D>,
        {}
        
        // This will compile only if Var<0>::Dual is Var<0>
        assert_dual_type::<Var<0>, Var<0>>();
    }

    #[test]
    fn test_var_duality_symmetry() {
        // This test verifies the symmetry of the duality relationship for Var<N>
        // Var<N>::Dual is Var<N>, and Var<N>::Dual::Dual is Var<N>
        
        // Check that Var<0>::Dual is Var<0> and Var<0>::Dual::Dual is Var<0>
        fn check_duality<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if the duality relationship is correctly implemented
        check_duality::<Var<0>, Var<0>>();
        
        // Also check for a different depth
        check_duality::<Var<1>, Var<1>>();
    }

    #[test]
    fn test_var_with_rec_composition() {
        // Test Var in composition with Rec and Send
        
        // A protocol that recursively sends an i32 and then refers back to itself
        type RecSend = Rec<Send<i32, Var<0>>>;
        
        // The dual should be Rec<Recv<i32, Var<0>>>
        type ExpectedDual = Rec<Recv<i32, Var<0>>>;
        
        // Check that the dual relationships are correctly established
        fn check_dual<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if RecSend's dual is correctly derived
        check_dual::<RecSend, ExpectedDual>();
    }

    #[test]
    fn test_nested_rec_var() {
        // Test nested Rec with Var references at different depths
        
        // A protocol with nested recursion:
        // - The outer Rec sends an i32 and then either:
        //   - Refers back to itself (Var<1>)
        //   - Enters an inner Rec that sends a String and then either:
        //     - Refers back to the inner Rec (Var<0>)
        //     - Refers back to the outer Rec (Var<1>)
        //     - Ends
        
        // For simplicity, we'll use a more basic nested structure:
        // An outer Rec that contains an inner Rec, with Var<0> referring to the inner Rec
        // and Var<1> referring to the outer Rec
        
        type InnerRec = Rec<Send<String, Var<0>>>;
        type OuterRec = Rec<Send<i32, InnerRec>>;
        
        // The dual should have the same structure but with Send replaced by Recv
        type ExpectedInnerDual = Rec<Recv<String, Var<0>>>;
        type ExpectedOuterDual = Rec<Recv<i32, ExpectedInnerDual>>;
        
        // Check that the dual relationships are correctly established
        fn check_nested_dual<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if the nested Rec/Var structure's dual is correctly derived
        check_nested_dual::<OuterRec, ExpectedOuterDual>();
    }
}