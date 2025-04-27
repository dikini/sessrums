//! End protocol type for session types.
//!
//! This module defines the `End` protocol type, which represents the end of a
//! communication session. It is a terminal protocol that indicates no further
//! communication will occur.

use super::Protocol;

/// A protocol that represents the end of a communication session.
///
/// The `End` protocol is a terminal protocol that indicates no further
/// communication will occur. It is typically used as the final protocol
/// in a chain of protocol compositions.
///
/// # Duality
///
/// The dual of `End` is `End` itself, as ending a session is symmetric
/// for both parties involved in the communication.
///
/// # Examples
///
/// ```
/// use sessrums::proto::{Protocol, Send, End};
///
/// // A protocol that sends an i32 and then ends
/// type SimpleProtocol = Send<i32, End>;
/// ```
#[derive(Debug)]
pub struct End;

impl Protocol for End {
    /// The dual of `End` is `End` itself.
    ///
    /// This reflects the fact that ending a session is symmetric
    /// for both parties involved in the communication.
    type Dual = End;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{Send, Recv};

    #[test]
    fn test_end_protocol_implementation() {
        // This test verifies that End implements Protocol
        // and that its Dual is defined as End itself
        
        // We can't directly test the equality of types in Rust,
        // but we can verify that End implements Protocol
        fn assert_implements_protocol<T: Protocol>() {}
        
        // This will compile only if End implements Protocol
        assert_implements_protocol::<End>();
        
        // We can also verify that the associated type Dual is defined correctly
        fn assert_dual_type<T, D>()
        where
            T: Protocol<Dual = D>,
        {}
        
        // This will compile only if End::Dual is End
        assert_dual_type::<End, End>();
    }

    #[test]
    fn test_end_duality_symmetry() {
        // This test verifies the symmetry of the duality relationship for End
        // End::Dual is End, and End::Dual::Dual is End
        
        // Check that End::Dual is End and End::Dual::Dual is End
        fn check_duality<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if the duality relationship is correctly implemented
        check_duality::<End, End>();
    }

    #[test]
    fn test_end_with_send_composition() {
        // Test End in composition with Send
        
        // A protocol that sends an i32 and then ends
        type SendThenEnd = Send<i32, End>;
        
        // The dual should be Recv<i32, End>
        type ExpectedDual = Recv<i32, End>;
        
        // Check that the dual relationships are correctly established
        fn check_dual<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if SendThenEnd's dual is correctly derived
        check_dual::<SendThenEnd, ExpectedDual>();
    }

    #[test]
    fn test_end_with_recv_composition() {
        // Test End in composition with Recv
        
        // A protocol that receives an i32 and then ends
        type RecvThenEnd = Recv<i32, End>;
        
        // The dual should be Send<i32, End>
        type ExpectedDual = Send<i32, End>;
        
        // Check that the dual relationships are correctly established
        fn check_dual<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if RecvThenEnd's dual is correctly derived
        check_dual::<RecvThenEnd, ExpectedDual>();
    }

    #[test]
    fn test_end_with_complex_composition() {
        // Test End in a more complex protocol composition
        
        // A protocol that sends an i32, receives a String, and then ends
        type ComplexProtocol = Send<i32, Recv<String, End>>;
        
        // The dual should be Recv<i32, Send<String, End>>
        type ExpectedDual = Recv<i32, Send<String, End>>;
        
        // Check that the dual relationships are correctly established
        fn check_complex_dual<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if ComplexProtocol's dual is correctly derived
        check_complex_dual::<ComplexProtocol, ExpectedDual>();
    }
}