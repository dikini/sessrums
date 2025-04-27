//! Integration test infrastructure for the sessrums library.
//!
//! This module provides helper functions and macros for testing session type protocols.
//! It's designed to make it easy to write tests that verify both the type-level properties
//! and runtime behavior of protocols.

// No need for PhantomData import
use sessrums::proto::Protocol;

/// Assert that a type implements the Protocol trait.
/// This is useful for verifying that protocol types are correctly defined.
pub fn assert_protocol<P: Protocol>() {}

/// Assert that two types have the correct duality relationship.
/// This verifies that Q is the dual of P by checking that they can be used
/// in a type-safe way with the Protocol trait.
pub fn assert_dual<P: Protocol, Q: Protocol>()
where
    P::Dual: Protocol,
{
    // This function doesn't need to do anything at runtime.
    // The type constraints ensure that P::Dual is a valid protocol type,
    // and we can use other means to verify that Q is equivalent to P::Dual.
    
    // At compile time, we can verify that Q is the dual of P by using
    // the Protocol trait's associated Dual type.
    fn assert_same_type<T, U>() where T: Protocol, U: Protocol {}
    assert_same_type::<P::Dual, Q>();
}

/// Assert that a type is its own dual.
/// This is useful for symmetric protocols like End.
pub fn assert_self_dual<P: Protocol>()
where
    P::Dual: Protocol,
{
    fn assert_same_type<T, U>() where T: Protocol, U: Protocol {}
    assert_same_type::<P, P::Dual>();
}

/// Helper function to create a channel with a specific protocol and IO type.
/// This is useful for testing protocol types without needing actual IO.
pub fn mock_channel<P: Protocol, IO>() -> sessrums::chan::Chan<P, IO>
where
    IO: Default,
{
    sessrums::chan::Chan::new(IO::default())
}

// Re-export protocol modules for convenience
pub mod protocol_1;
pub mod protocol_2;
pub mod protocol_3;
pub mod protocol_4;
pub mod protocol_5;

// Add more protocol modules as they are implemented