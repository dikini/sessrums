//! Protocol 5: Data Query with Options
//!
//! # Protocol Description
//!
//! This example demonstrates a data query protocol with options where:
//! - Client sends a query string
//! - Server receives the query string
//! - Server chooses between two options:
//!   - Option 1: Server sends binary data (Vec<u8>) and ends
//!   - Option 2: Server sends an error code (i16) and ends
//! - Client offers these two options:
//!   - Option 1: Client receives binary data (Vec<u8>) and ends
//!   - Option 2: Client receives an error code (i16) and ends
//!
//! # Session Type Safety
//!
//! This protocol demonstrates several key aspects of session type safety:
//!
//! 1. **Type-level Protocol Definition**: The protocol is defined at the type level using
//!    `Send<T, P>`, `Recv<T, P>`, `Choose<L, R>`, `Offer<L, R>`, and `End` types, ensuring
//!    that the communication sequence is enforced by the Rust type system.
//!
//! 2. **Duality**: The client and server protocols are duals of each other, ensuring
//!    that they can communicate without deadlocks or protocol violations. When the client
//!    sends, the server must be ready to receive; when the server chooses, the client must
//!    be ready to offer the corresponding options.
//!
//! 3. **Type Safety**: The protocol ensures that the correct types are sent and received
//!    at each step. The client must send a query string (String), and then be ready to
//!    receive either binary data (Vec<u8>) or an error code (i16). The server must receive
//!    the query string, and then choose to send either binary data or an error code.
//!
//! 4. **Protocol Completion**: Both sides must follow the protocol to completion, ending
//!    with the `End` type, which ensures that no communication is left hanging.
//!
//! 5. **Branching Communication**: This protocol demonstrates how session types can handle
//!    branching communication patterns using `Choose` and `Offer` types. The server can
//!    choose which branch to take based on runtime conditions (e.g., whether the query
//!    was successful), and the client must be prepared to handle either branch.
//!
//! # Visual Diagram
//!
//! ```text
//!                   QueryClient                      QueryServer
//!                   -----------                      -----------
//!                        |                               |
//!                        |                               |
//!                        |        Send(query)            |
//!                        | ----------------------------> |
//!                        |                               |
//!                        |        Recv(query)            |
//!                        | <---------------------------- |
//!                        |                               |
//!                        |                               | Server chooses:
//!                        |                               |
//!                        |                               |--------+
//!                        |                               |        |
//!                        |                               |        |
//!                        |                               V        V
//!                        |                          Option 1   Option 2
//!                        |                               |        |
//!                        |        Recv(data)             |        |
//!                        | <---------------------------- |        |
//!                        |                               |        |
//!                        |                               |        |
//!                        |        Recv(error)            |        |
//!                        | <------------------------------------ |
//!                        |                               |        |
//!                        |                               |        |
//!                        V                               V        V
//!                       End                             End      End
//! ```
//!
//! # Type-Level Representation
//!
//! ```
//! Client: Send<String, Offer<Recv<Vec<u8>, End>, Recv<i16, End>>>
//! Server: Recv<String, Choose<Send<Vec<u8>, End>, Send<i16, End>>>
//! ```
//!
//! This demonstrates how the types mirror the communication flow and ensure
//! protocol adherence at compile time. The client sends a query string, then
//! offers to receive either binary data or an error code. The server receives
//! the query string, then chooses to send either binary data or an error code.

use sez::proto::{Send, Recv, Choose, Offer, End};
use sez::chan::Chan;

// Import helper functions from the integration test module
use crate::integration::{assert_protocol, assert_dual, mock_channel};

// Define the protocol types
// Client: Send query, offer to receive data or error, then end
type QueryClient = Send<String, Offer<Recv<Vec<u8>, End>, Recv<i16, End>>>;
// Server: Receive query, choose to send data or error, then end
type QueryServer = Recv<String, Choose<Send<Vec<u8>, End>, Send<i16, End>>>;

/// This test verifies the type-level properties of the data query protocol with options.
///
/// While the actual send/recv/choose/offer methods will be implemented in Phase 3, this test
/// focuses on ensuring that the protocol types are correctly defined and that
/// the duality relationship between client and server protocols is maintained.
#[tokio::test]
async fn test_data_query_with_options_protocol() {
    // This is a placeholder that will be implemented in Phase 3
    // after the send/recv/choose/offer methods are implemented
    
    // The implementation will:
    // 1. Create a pair of channels with the QueryClient and QueryServer types
    // 2. Client sends query string
    // 3. Server receives query string
    // 4. Server chooses to send either data or error code
    // 5. Client receives either data or error code based on server's choice
    // 6. Both sides close the connection
    
    // Verify that QueryClient and QueryServer implement the Protocol trait
    assert_protocol::<QueryClient>();
    assert_protocol::<QueryServer>();
    
    // Verify that QueryServer is the dual of QueryClient
    // This ensures that the two protocols can communicate with each other
    // without deadlocks or protocol violations
    assert_dual::<QueryClient, QueryServer>();
    
    // Create mock channels for type checking
    // These channels don't perform actual IO operations but allow us to verify
    // that the protocol types can be used with the Chan type
    let _client_chan: Chan<QueryClient, ()> = mock_channel();
    let _server_chan: Chan<QueryServer, ()> = mock_channel();
    
    // In Phase 3, we'll add actual communication code here to demonstrate
    // the runtime behavior of the protocol
}

/// This test demonstrates how the type system prevents protocol violations.
///
/// If we were to try to use the wrong protocol type for a channel, the code
/// would fail to compile, demonstrating the type safety provided by session types.
#[test]
fn test_data_query_with_options_type_safety() {
    // The following line would fail to compile if uncommented because
    // we're trying to use a server protocol for a client channel:
    //
    // let _invalid_chan: Chan<QueryClient, ()> = mock_channel::<QueryServer, ()>();
    //
    // Similarly, trying to send a different type than expected (e.g., u32 instead of String)
    // would fail to compile in the actual implementation.
    
    // Instead, we verify that the correct types work
    let _client_chan: Chan<QueryClient, ()> = mock_channel::<QueryClient, ()>();
    let _server_chan: Chan<QueryServer, ()> = mock_channel::<QueryServer, ()>();
}