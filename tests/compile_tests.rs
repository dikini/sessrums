//! Compile-time tests for the sessrums library.
//!
//! This file contains tests that verify the compile-time behavior of the session type system.
//! These tests ensure that the type system correctly enforces protocol adherence and rejects
//! invalid protocols at compile time.
//!
//! The tests in this file are designed to compile successfully, demonstrating correct usage
//! of the session type system. For tests that should fail to compile, see the `compile_fail`
//! directory.

use sessrums::proto::{Send, Recv, Choose, Offer, End, Protocol};
use sessrums::chan::Chan;

/// Test that verifies the basic protocol type definitions compile correctly.
#[test]
fn test_protocol_type_definitions() {
    // Define simple protocol types
    type SimpleClient = Send<i32, End>;
    type SimpleServer = Recv<i32, End>;
    
    // Define more complex protocol types
    type ComplexClient = Send<i32, Recv<String, End>>;
    type ComplexServer = Recv<i32, Send<String, End>>;
    
    // Define protocol types with choice
    type ChoiceClient = Send<i32, Choose<Send<String, End>, Send<bool, End>>>;
    type OfferServer = Recv<i32, Offer<Recv<String, End>, Recv<bool, End>>>;
    
    // Verify that these types implement the Protocol trait
    fn assert_protocol<P: Protocol>() {}
    
    assert_protocol::<SimpleClient>();
    assert_protocol::<SimpleServer>();
    assert_protocol::<ComplexClient>();
    assert_protocol::<ComplexServer>();
    assert_protocol::<ChoiceClient>();
    assert_protocol::<OfferServer>();
}

/// Test that verifies the duality relationship between protocol types.
#[test]
fn test_protocol_duality() {
    // Define protocol types
    type ClientProto = Send<i32, Recv<String, End>>;
    type ServerProto = Recv<i32, Send<String, End>>;
    
    // Verify duality
    fn assert_dual<P: Protocol, Q: Protocol>()
    where
        P::Dual: Protocol,
        Q: Protocol<Dual = P>,
        P: Protocol<Dual = Q>,
    {}
    
    assert_dual::<ClientProto, ServerProto>();
    
    // Verify self-duality of End
    fn assert_self_dual<P: Protocol>()
    where
        P::Dual: Protocol<Dual = P>,
    {}
    
    assert_self_dual::<End>();
}

/// Test that verifies the type-level properties of the Choose and Offer types.
#[test]
fn test_choose_offer_types() {
    // Define protocol types with choice
    type ChoiceProto = Choose<Send<i32, End>, Send<String, End>>;
    type OfferProto = Offer<Recv<i32, End>, Recv<String, End>>;
    
    // Verify duality
    fn assert_dual<P: Protocol, Q: Protocol>()
    where
        P::Dual: Protocol,
        Q: Protocol<Dual = P>,
        P: Protocol<Dual = Q>,
    {}
    
    assert_dual::<ChoiceProto, OfferProto>();
}

/// Test that verifies the type-level properties of nested protocol types.
#[test]
fn test_nested_protocol_types() {
    // Define nested protocol types
    type NestedClient = Send<i32, Choose<
        Send<String, Recv<bool, End>>,
        Send<f64, Recv<char, End>>
    >>;
    
    type NestedServer = Recv<i32, Offer<
        Recv<String, Send<bool, End>>,
        Recv<f64, Send<char, End>>
    >>;
    
    // Verify duality
    fn assert_dual<P: Protocol, Q: Protocol>()
    where
        P::Dual: Protocol,
        Q: Protocol<Dual = P>,
        P: Protocol<Dual = Q>,
    {}
    
    assert_dual::<NestedClient, NestedServer>();
}

/// Test that verifies the type-level properties of channel creation.
#[test]
fn test_channel_creation() {
    // Define protocol types
    type ClientProto = Send<i32, End>;
    type ServerProto = Recv<i32, End>;
    
    // Create channels
    let _client_chan = Chan::<ClientProto, sessrums::proto::RoleA, ()>::new(()); // Added RoleA
    let _server_chan = Chan::<ServerProto, sessrums::proto::RoleB, ()>::new(()); // Added RoleB
    
    // The fact that this compiles verifies that the channel creation works correctly
}

/// Test that verifies the type-level properties of the protocol combinators.
#[test]
fn test_protocol_combinators() {
    // Define protocol types using type aliases for clarity
    type SendInt = Send<i32, End>;
    type RecvInt = Recv<i32, End>;
    type SendString = Send<String, End>;
    type RecvString = Recv<String, End>;
    
    // Combine protocol types
    type CombinedClient = Send<i32, Recv<String, End>>;
    type CombinedServer = Recv<i32, Send<String, End>>;
    
    // Verify duality
    fn assert_dual<P: Protocol, Q: Protocol>()
    where
        P::Dual: Protocol,
        Q: Protocol<Dual = P>,
        P: Protocol<Dual = Q>,
    {}
    
    assert_dual::<CombinedClient, CombinedServer>();
    
    // Verify that the combined types implement the Protocol trait
    fn assert_protocol<P: Protocol>() {}
    
    assert_protocol::<CombinedClient>();
    assert_protocol::<CombinedServer>();
}