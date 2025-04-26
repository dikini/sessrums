//! Core Protocol trait for session types.
//!
//! This module defines the fundamental `Protocol` trait that serves as the
//! foundation for the session type system. Session types allow for compile-time
//! verification of communication protocol adherence between communicating parties.

/// The `Protocol` trait represents a communication protocol type.
///
/// In session type theory, protocols describe the communication behavior between
/// two parties. Each protocol has a dual counterpart that represents the behavior
/// of the other party in the communication.
///
/// # Duality
///
/// Duality is a fundamental concept in session types. For every protocol `P`,
/// there exists a dual protocol `P::Dual` that represents the complementary
/// behavior. For example:
///
/// - If `P` sends a message of type `T`, then `P::Dual` receives a message of type `T`.
/// - If `P` offers a choice between protocols, then `P::Dual` makes a choice between the dual protocols.
///
/// This duality ensures that when two parties follow dual protocols, their
/// communication is guaranteed to be compatible and free from communication errors
/// like deadlocks or protocol violations.
///
/// # Examples
///
/// ```
/// use sez::proto::{Protocol, Send, Recv, End};
///
/// // A protocol that sends an i32, then receives a String, then ends
/// type MyProtocol = Send<i32, Recv<String, End>>;
///
/// // The dual protocol receives an i32, then sends a String, then ends
/// // (This will be implemented in future tasks)
/// // type MyDualProtocol = <MyProtocol as Protocol>::Dual;
/// // This would be equivalent to: Recv<i32, Send<String, End>>
/// ```
///
/// # Type Safety
///
/// The `Protocol` trait and its implementations leverage Rust's type system to
/// ensure protocol adherence at compile time. This means that protocol violations
/// are caught as type errors during compilation, preventing runtime communication
/// errors.
pub trait Protocol: Sized {
    /// The dual protocol type.
    ///
    /// For any protocol `P`, `P::Dual` represents the complementary protocol
    /// that can communicate with `P` without errors.
    ///
    /// The duality relationship is symmetric, meaning that for any protocol `P`:
    /// `P::Dual::Dual == P`
    type Dual: Protocol;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define some simple protocol types for testing
    struct TestProtocol;
    struct DualTestProtocol;

    // Implement Protocol for TestProtocol
    impl Protocol for TestProtocol {
        type Dual = DualTestProtocol;
    }

    // Implement Protocol for DualTestProtocol
    impl Protocol for DualTestProtocol {
        type Dual = TestProtocol;
    }

    #[test]
    fn test_protocol_duality_symmetry() {
        // This test verifies the symmetry of the duality relationship
        // by checking that the dual of the dual is the original type.
        
        // We can't directly compare types in Rust, but we can use type inference
        // to verify that the types match.
        
        // TestProtocol::Dual is DualTestProtocol
        // DualTestProtocol::Dual is TestProtocol
        // So TestProtocol::Dual::Dual is TestProtocol, which confirms the symmetry property.
        
        // This is a compile-time check - if these types match up correctly,
        // the code will compile successfully.
        fn check_dual_symmetry<P, Q>()
        where
            P: Protocol<Dual = Q>,
            Q: Protocol<Dual = P>,
        {
            // Empty function body - we're just checking type relationships
        }
        
        // Verify that TestProtocol and DualTestProtocol are duals of each other
        check_dual_symmetry::<TestProtocol, DualTestProtocol>();
    }
}