//! Protocol 1: Simple Send/Recv Ping-Pong
//!
//! # Protocol Description
//!
//! This example demonstrates a simple ping-pong protocol where:
//! - Client sends an i32 value
//! - Server receives the i32 value
//! - Server sends a String response
//! - Client receives the String response
//! - Both sides close the connection
//!
//! # Session Type Safety
//!
//! This protocol demonstrates several key aspects of session type safety:
//!
//! 1. **Type-level Protocol Definition**: The protocol is defined at the type level using
//!    `Send<T, P>`, `Recv<T, P>`, and `End` types, ensuring that the communication
//!    sequence is enforced by the Rust type system.
//!
//! 2. **Duality**: The client and server protocols are duals of each other, ensuring
//!    that they can communicate without deadlocks or protocol violations. When the client
//!    sends, the server receives; when the server sends, the client receives.
//!
//! 3. **Type Safety**: The protocol ensures that the correct types are sent and received
//!    at each step. The client must send an i32, and the server must respond with a String.
//!
//! 4. **Protocol Completion**: Both sides must follow the protocol to completion, ending
//!    with the `End` type, which ensures that no communication is left hanging.
//!
//! # Visual Diagram
//!
//! ```text
//!                   PingPongClient                 PingPongServer
//!                   --------------                 --------------
//!                         |                              |
//!                         |        Send(i32)            |
//!                         | ---------------------------> |
//!                         |                              |
//!                         |        Recv(String)          |
//!                         | <--------------------------- |
//!                         |                              |
//!                         |           End                |
//!                         | - - - - - - - - - - - - - - -|
//!                         |                              |
//! ```
//!
//! # Type-Level Representation
//!
//! ```
//! Client: Send<i32, Recv<String, End>>
//! Server: Recv<i32, Send<String, End>>
//! ```
//!
//! This demonstrates how the types mirror the communication flow and ensure
//! protocol adherence at compile time.

use sez::proto::{Send, Recv, End};
use sez::chan::Chan;

// Import helper functions from the integration test module
use crate::integration::{assert_protocol, assert_dual, mock_channel};

// Define the protocol types
// Client: Send an i32, then receive a String, then end
type PingPongClient = Send<i32, Recv<String, End>>;
// Server: Receive an i32, then send a String, then end
type PingPongServer = Recv<i32, Send<String, End>>;

/// This test verifies the type-level properties of the ping-pong protocol.
///
/// While the actual send/recv methods will be implemented in Phase 3, this test
/// focuses on ensuring that the protocol types are correctly defined and that
/// the duality relationship between client and server protocols is maintained.
#[tokio::test]
async fn test_ping_pong_protocol() {
    // This is a placeholder that will be implemented in Phase 3
    // after the send/recv methods are implemented
    
    // The implementation will:
    // 1. Create a pair of channels with the PingPongClient and PingPongServer types
    // 2. Client sends an i32 value
    // 3. Server receives the i32 value
    // 4. Server sends a String response
    // 5. Client receives the String response
    // 6. Both sides close the connection
    
    // Verify that PingPongClient and PingPongServer implement the Protocol trait
    assert_protocol::<PingPongClient>();
    assert_protocol::<PingPongServer>();
    
    // Verify that PingPongServer is the dual of PingPongClient
    // This ensures that the two protocols can communicate with each other
    // without deadlocks or protocol violations
    assert_dual::<PingPongClient, PingPongServer>();
    
    // Create mock channels for type checking
    // These channels don't perform actual IO operations but allow us to verify
    // that the protocol types can be used with the Chan type
    let _client_chan: Chan<PingPongClient, ()> = mock_channel();
    let _server_chan: Chan<PingPongServer, ()> = mock_channel();
    
    // In Phase 3, we'll add actual communication code here to demonstrate
    // the runtime behavior of the protocol
}

/// This test demonstrates how the type system prevents protocol violations.
///
/// If we were to try to use the wrong protocol type for a channel, the code
/// would fail to compile, demonstrating the type safety provided by session types.
#[test]
fn test_ping_pong_type_safety() {
    // The following line would fail to compile if uncommented because
    // we're trying to use a server protocol for a client channel:
    //
    // let _invalid_chan: Chan<PingPongClient, ()> = mock_channel::<PingPongServer, ()>();
    //
    // Similarly, trying to send a String when the protocol expects an i32
    // would fail to compile in the actual implementation.
    
    // Instead, we verify that the correct types work
    let _client_chan: Chan<PingPongClient, ()> = mock_channel::<PingPongClient, ()>();
    let _server_chan: Chan<PingPongServer, ()> = mock_channel::<PingPongServer, ()>();
}