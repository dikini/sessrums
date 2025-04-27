//! Recv protocol type for session types.
//!
//! This module defines the `Recv<T, P>` protocol type, which represents a protocol
//! that receives a value of type `T` and then continues with protocol `P`.
//!
//! The `Recv` type is a fundamental building block in session types, allowing
//! for the specification of protocols that receive data from a communication partner.

use std::marker::PhantomData;
use super::Protocol;
use super::send::Send;

/// A protocol that receives a value of type `T` and then continues with protocol `P`.
///
/// # Type Parameters
///
/// * `T` - The type of value to be received.
/// * `P` - The protocol to continue with after receiving the value.
///
/// # Duality
///
/// The dual of `Recv<T, P>` is `Send<T, P::Dual>`, which represents sending a value
/// of type `T` and then continuing with the dual of protocol `P`.
///
/// # Examples
///
/// ```
/// use sessrums::proto::{Protocol, Send, Recv, End};
///
/// // Define a protocol that receives an i32, then a String, then ends
/// type MyProtocol = Recv<i32, Recv<String, End>>;
///
/// // The dual protocol would be:
/// type MyDualProtocol = <MyProtocol as Protocol>::Dual;
/// // Which is equivalent to: Send<i32, Send<String, End>>
/// ```
///
/// # Protocol Composition
///
/// `Recv` can be composed with other protocol types to build complex communication
/// patterns:
///
/// ```
/// use sessrums::proto::{Protocol, Send, Recv, End};
///
/// // A protocol that receives an i32, sends a bool, then receives a String, then ends
/// type ComplexProtocol = Recv<i32, Send<bool, Recv<String, End>>>;
/// ```
pub struct Recv<T, P> {
    _marker: PhantomData<(T, P)>,
}

impl<T, P: Protocol> Protocol for Recv<T, P> {
    /// The dual of `Recv<T, P>` is `Send<T, P::Dual>`.
    ///
    /// This implements the duality principle of session types:
    /// - If one party receives a value of type `T`, the other party must send a value of type `T`.
    /// - After the communication, both parties continue with dual protocols.
    type Dual = Send<T, P::Dual>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{Protocol, End, Send};

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
    fn test_recv_protocol_implementation() {
        // This test verifies that Recv<T, P> implements Protocol
        // and that its Dual is defined as Send<T, P::Dual>
        
        // We can't directly test the equality of types in Rust,
        // but we can verify that Recv<T, P> implements Protocol
        fn assert_implements_protocol<T: Protocol>() {}
        
        // This will compile only if Recv<i32, TestProtocol> implements Protocol
        assert_implements_protocol::<Recv<i32, TestProtocol>>();
        
        // We can also verify that the associated type Dual is defined correctly
        fn assert_dual_type<T, D>()
        where
            T: Protocol<Dual = D>,
        {}
        
        // This will compile only if Recv<i32, TestProtocol>::Dual is Send<i32, DualTestProtocol>
        assert_dual_type::<Recv<i32, TestProtocol>, Send<i32, DualTestProtocol>>();
    }

    #[test]
    fn test_recv_duality_relationship() {
        // This test verifies the duality relationship between Recv and Send
        
        // Define type aliases for clarity
        type RecvInt = Recv<i32, End>;
        type SendInt = Send<i32, End>;
        
        // Check that RecvInt::Dual is SendInt and SendInt::Dual is RecvInt
        fn check_duality<R, S>()
        where
            R: Protocol<Dual = S>,
            S: Protocol<Dual = R>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // This will compile only if the duality relationship is correctly implemented
        check_duality::<RecvInt, SendInt>();
    }

    #[test]
    fn test_complex_protocol_composition() {
        // Test a more complex protocol composition
        
        // A protocol that receives an i32, then a String, then ends
        type ComplexProtocol = Recv<i32, Recv<String, End>>;
        
        // The dual should be Send<i32, Send<String, End>>
        type ExpectedDual = Send<i32, Send<String, End>>;
        
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
        
        // A protocol that receives an i32, then sends a bool, then receives a String, then ends
        type NestedProtocol = Recv<i32, Send<bool, Recv<String, End>>>;
        
        // The dual should be Send<i32, Recv<bool, Send<String, End>>>
        type ExpectedNestedDual = Send<i32, Recv<bool, Send<String, End>>>;
        
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