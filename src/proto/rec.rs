//! Recursive protocol type for session types.
//!
//! This module defines the `Rec<P>` protocol type, which represents a recursive
//! protocol that can refer to itself. This is essential for expressing protocols
//! with repetitive or looping behavior.

use std::marker::PhantomData;
use super::Protocol;

/// A protocol that represents a recursive protocol definition.
///
/// The `Rec<P>` type allows for the definition of recursive protocols where
/// the protocol can refer to itself. This is achieved by using `Var<N>` types
/// within the protocol definition `P` to reference back to the enclosing `Rec`.
///
/// # Type Parameters
///
/// * `P` - The protocol body that may contain references to itself via `Var<N>`.
///
/// # Duality
///
/// The dual of `Rec<P>` is `Rec<P::Dual>`, which means that the recursive structure
/// is preserved while the inner protocol is dualized.
///
/// # Examples
///
/// ```
/// use sessrums::proto::{Protocol, Rec, Var, Send, Recv, End};
///
/// // A protocol that repeatedly sends an i32 until it decides to end
/// // Rec<Send<i32, Choose<Var<0>, End>>>
/// // (Note: Choose implementation would be needed for this example)
/// 
/// // A simpler example: a protocol that sends an i32 and then repeats itself
/// type LoopingSend = Rec<Send<i32, Var<0>>>;
///
/// // The dual protocol would receive an i32 and then repeat itself
/// type LoopingRecv = <LoopingSend as Protocol>::Dual; // Rec<Recv<i32, Var<0>>>
/// ```
pub struct Rec<P> {
    _marker: PhantomData<P>,
}

impl<P: Protocol> Protocol for Rec<P> {
    /// The dual of `Rec<P>` is `Rec<P::Dual>`.
    ///
    /// This preserves the recursive structure while dualizing the inner protocol.
    type Dual = Rec<P::Dual>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{End, Send, Recv, Var};

    #[test]
    fn test_rec_protocol_implementation() {
        // This test verifies that Rec<P> implements Protocol
        // and that its Dual is defined as Rec<P::Dual>
        
        // We can't directly test the equality of types in Rust,
        // but we can verify that Rec<End> implements Protocol
        fn assert_implements_protocol<T: Protocol>() {}
        
        // This will compile only if Rec<End> implements Protocol
        assert_implements_protocol::<Rec<End>>();
        
        // We can also verify that the associated type Dual is defined correctly
        fn assert_dual_type<T, D>()
        where
            T: Protocol<Dual = D>,
        {}
        
        // This will compile only if Rec<End>::Dual is Rec<End>
        assert_dual_type::<Rec<End>, Rec<End>>();
    }

    #[test]
    fn test_rec_duality_symmetry() {
        // This test verifies the symmetry of the duality relationship for Rec<P>
        // Rec<P>::Dual is Rec<P::Dual>, and Rec<P::Dual>::Dual is Rec<P>
        
        // Check that Rec<End>::Dual is Rec<End> and Rec<End>::Dual::Dual is Rec<End>
        fn check_duality<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if the duality relationship is correctly implemented
        check_duality::<Rec<End>, Rec<End>>();
    }

    #[test]
    fn test_rec_with_send_composition() {
        // Test Rec in composition with Send
        
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
    fn test_rec_with_complex_composition() {
        // Test Rec in a more complex protocol composition
        
        // A protocol that recursively sends an i32, receives a String, and then refers back to itself
        type ComplexRec = Rec<Send<i32, Recv<String, Var<0>>>>;
        
        // The dual should be Rec<Recv<i32, Send<String, Var<0>>>>
        type ExpectedDual = Rec<Recv<i32, Send<String, Var<0>>>>;
        
        // Check that the dual relationships are correctly established
        fn check_complex_dual<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if ComplexRec's dual is correctly derived
        check_complex_dual::<ComplexRec, ExpectedDual>();
    }
}