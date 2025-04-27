//! Protocol 3: Simple Choice
//!
//! # Protocol Description
//!
//! This example demonstrates a simple choice protocol where:
//! - Client chooses between two options:
//!   - Option 1: Client sends a u64 value and ends
//!   - Option 2: Client receives an f32 value and ends
//! - Server offers these two options:
//!   - Option 1: Server receives a u64 value and ends
//!   - Option 2: Server sends an f32 value and ends
//!
//! # Session Type Safety
//!
//! This protocol demonstrates several key aspects of session type safety:
//!
//! 1. **Type-level Protocol Definition**: The protocol is defined at the type level using
//!    `Choose<L, R>`, `Offer<L, R>`, `Send<T, P>`, `Recv<T, P>`, and `End` types, ensuring
//!    that the communication sequence is enforced by the Rust type system.
//!
//! 2. **Duality**: The client and server protocols are duals of each other, ensuring
//!    that they can communicate without deadlocks or protocol violations. When the client
//!    chooses to send, the server must be ready to receive; when the client chooses to
//!    receive, the server must be ready to send.
//!
//! 3. **Type Safety**: The protocol ensures that the correct types are sent and received
//!    at each step. The client must send a u64 or receive an f32, and the server must
//!    receive a u64 or send an f32, depending on the client's choice.
//!
//! 4. **Protocol Completion**: Both sides must follow the protocol to completion, ending
//!    with the `End` type, which ensures that no communication is left hanging.
//!
//! # Visual Diagram
//!
//! ```text
//!                   ChoiceClient                    ChoiceServer
//!                   ------------                    ------------
//!                         |                              |
//!                         |        Choose                |
//!                         | ---------------------------> |
//!                         |                              |
//!                         |        Offer                 |
//!                         | <--------------------------- |
//!                         |                              |
//!                  +------+------+              +--------+-------+
//!                  |             |              |                |
//!                  |             |              |                |
//!            Option 1       Option 2      Option 1         Option 2
//!                  |             |              |                |
//!                  |             |              |                |
//!                  v             v              v                v
//!               Send(u64)     Recv(f32)     Recv(u64)        Send(f32)
//!                  |             |              |                |
//!                  v             v              v                v
//!                 End           End            End              End
//! ```
//!
//! # Type-Level Representation
//!
//! ```
//! Client: Choose<Send<u64, End>, Recv<f32, End>>
//! Server: Offer<Recv<u64, End>, Send<f32, End>>
//! ```
//!
//! This demonstrates how the types mirror the communication flow and ensure
//! protocol adherence at compile time. The client chooses between sending a u64
//! or receiving an f32, while the server offers to either receive a u64 or send
//! an f32, depending on the client's choice.

use sessrums::proto::{Choose, Offer, Send, Recv, End};
use sessrums::chan::Chan;

// Import helper functions from the integration test module
use crate::integration::{assert_protocol, assert_dual, mock_channel};

// Define the protocol types
// Client: Choose between sending a u64 or receiving an f32, then end
type ChoiceClient = Choose<Send<u64, End>, Recv<f32, End>>;
// Server: Offer to either receive a u64 or send an f32, then end
type ChoiceServer = Offer<Recv<u64, End>, Send<f32, End>>;

/// This test verifies the type-level properties of the simple choice protocol.
///
/// While the actual choose/offer methods will be implemented in Phase 4, this test
/// focuses on ensuring that the protocol types are correctly defined and that
/// the duality relationship between client and server protocols is maintained.
#[tokio::test]
async fn test_simple_choice_protocol() {
    // This is a placeholder that will be implemented in Phase 4
    // after the choose/offer methods are implemented
    
    // The implementation will:
    // 1. Create a pair of channels with the ChoiceClient and ChoiceServer types
    // 2. Client chooses either:
    //    a. Send a u64 value and end, or
    //    b. Receive an f32 value and end
    // 3. Server offers either:
    //    a. Receive a u64 value and end, or
    //    b. Send an f32 value and end
    // 4. Both sides close the connection
    
    // Verify that ChoiceClient and ChoiceServer implement the Protocol trait
    assert_protocol::<ChoiceClient>();
    assert_protocol::<ChoiceServer>();
    
    // Verify that ChoiceServer is the dual of ChoiceClient
    // This ensures that the two protocols can communicate with each other
    // without deadlocks or protocol violations
    assert_dual::<ChoiceClient, ChoiceServer>();
    
    // Create mock channels for type checking
    // These channels don't perform actual IO operations but allow us to verify
    // that the protocol types can be used with the Chan type
    let _client_chan: Chan<ChoiceClient, ()> = mock_channel();
    let _server_chan: Chan<ChoiceServer, ()> = mock_channel();
    
    // In Phase 4, we'll add actual communication code here to demonstrate
    // the runtime behavior of the protocol
}

/// This test demonstrates how the type system prevents protocol violations.
///
/// If we were to try to use the wrong protocol type for a channel, the code
/// would fail to compile, demonstrating the type safety provided by session types.
#[test]
fn test_simple_choice_type_safety() {
    // The following line would fail to compile if uncommented because
    // we're trying to use a server protocol for a client channel:
    //
    // let _invalid_chan: Chan<ChoiceClient, ()> = mock_channel::<ChoiceServer, ()>();
    //
    // Similarly, trying to choose a branch that doesn't exist in the protocol
    // would fail to compile in the actual implementation.
    
    // Instead, we verify that the correct types work
    let _client_chan: Chan<ChoiceClient, ()> = mock_channel::<ChoiceClient, ()>();
    let _server_chan: Chan<ChoiceServer, ()> = mock_channel::<ChoiceServer, ()>();
}