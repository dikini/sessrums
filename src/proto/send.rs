//! Send protocol type for session types.
//!
//! This module defines the `Send<T, P>` protocol type, which represents a protocol
//! that sends a value of type `T` and then continues with protocol `P`.
//!
//! The `Send` type is a fundamental building block in session types, allowing
//! for the specification of protocols that send data to a communication partner.

use std::marker::PhantomData;
use super::Protocol;
use super::recv::Recv;

/// A protocol that sends a value of type `T` and then continues with protocol `P`.
///
/// # Type Parameters
///
/// * `T` - The type of value to be sent.
/// * `P` - The protocol to continue with after sending the value.
///
/// # Duality
///
/// The dual of `Send<T, P>` is `Recv<T, P::Dual>`, which represents receiving a value
/// of type `T` and then continuing with the dual of protocol `P`.
///
/// # Examples
///
/// ```
/// use sez::proto::{Protocol, Send, Recv, End};
///
/// // Define a protocol that sends an i32, then a String, then ends
/// type MyProtocol = Send<i32, Send<String, End>>;
///
/// // The dual protocol would be:
/// type MyDualProtocol = <MyProtocol as Protocol>::Dual;
/// // Which is equivalent to: Recv<i32, Recv<String, End>>
/// ```
///
/// # Protocol Composition
///
/// `Send` can be composed with other protocol types to build complex communication
/// patterns:
///
/// ```
/// use sez::proto::{Protocol, Send, Recv, End};
///
/// // A protocol that sends an i32, receives a bool, then sends a String, then ends
/// type ComplexProtocol = Send<i32, Recv<bool, Send<String, End>>>;
/// ```
pub struct Send<T, P> {
    _marker: PhantomData<(T, P)>,
}

impl<T, P: Protocol> Protocol for Send<T, P> {
    /// The dual of `Send<T, P>` is `Recv<T, P::Dual>`.
    ///
    /// This implements the duality principle of session types:
    /// - If one party sends a value of type `T`, the other party must receive a value of type `T`.
    /// - After the communication, both parties continue with dual protocols.
    type Dual = Recv<T, P::Dual>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{Protocol, End, Recv};

    // Define a simple test protocol
    struct TestProtocol;
    struct DualTestProtocol;

    impl Protocol for TestProtocol {
        type Dual = DualTestProtocol;
    }

    impl Protocol for DualTestProtocol {
        type Dual = TestProtocol;
    }

    #[test]
    fn test_send_protocol_implementation() {
        // This test verifies that Send<T, P> implements Protocol
        // and that its Dual is defined as Recv<T, P::Dual>
        
        // We can't directly test the equality of types in Rust,
        // but we can verify that Send<T, P> implements Protocol
        fn assert_implements_protocol<T: Protocol>() {}
        
        // This will compile only if Send<i32, TestProtocol> implements Protocol
        assert_implements_protocol::<Send<i32, TestProtocol>>();
        
        // We can also verify that the associated type Dual is defined correctly
        fn assert_dual_type<T, D>()
        where
            T: Protocol<Dual = D>,
        {}
        
        // This will compile only if Send<i32, TestProtocol>::Dual is Recv<i32, DualTestProtocol>
        assert_dual_type::<Send<i32, TestProtocol>, Recv<i32, DualTestProtocol>>();
    }

    #[test]
    fn test_send_duality_relationship() {
        // This test verifies the duality relationship between Send and Recv
        
        // Define type aliases for clarity
        type SendInt = Send<i32, End>;
        type RecvInt = Recv<i32, End>;
        
        // Check that SendInt::Dual is RecvInt and RecvInt::Dual is SendInt
        fn check_duality<S, R>()
        where
            S: Protocol<Dual = R>,
            R: Protocol<Dual = S>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if the duality relationship is correctly implemented
        check_duality::<SendInt, RecvInt>();
    }

    #[test]
    fn test_complex_protocol_composition() {
        // Test a more complex protocol composition
        
        // A protocol that sends an i32, then a String, then ends
        type ComplexProtocol = Send<i32, Send<String, End>>;
        
        // The dual should be Recv<i32, Recv<String, End>>
        type ExpectedDual = Recv<i32, Recv<String, End>>;
        
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

    #[test]
    fn test_nested_protocol_composition() {
        // Test a nested protocol composition with different types
        
        // A protocol that sends an i32, then receives a bool, then sends a String, then ends
        type NestedProtocol = Send<i32, Recv<bool, Send<String, End>>>;
        
        // The dual should be Recv<i32, Send<bool, Recv<String, End>>>
        type ExpectedNestedDual = Recv<i32, Send<bool, Recv<String, End>>>;
        
        // Check that the dual relationships are correctly established
        fn check_nested_dual<P, D>()
        where
            P: Protocol<Dual = D>,
            D: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if NestedProtocol's dual is correctly derived
        check_nested_dual::<NestedProtocol, ExpectedNestedDual>();
    }
}