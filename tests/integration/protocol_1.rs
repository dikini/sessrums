//! Protocol 1: Simple Send/Recv Ping-Pong
//!
//! This example demonstrates a simple ping-pong protocol where:
//! - Client sends an i32 value
//! - Server receives the i32 value
//! - Server sends a String response
//! - Client receives the String response
//! - Both sides close the connection

use std::marker::PhantomData;
use sez::proto::{Protocol, Send, Recv, End};
use sez::chan::Chan;

// Define the protocol types
type PingPongClient = Send<i32, Recv<String, End>>;
type PingPongServer = Recv<i32, Send<String, End>>;

// This test will be implemented once the send/recv methods are available
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
    
    // For now, we just verify that the types are correctly defined
    fn assert_protocol<P: Protocol>() {}
    assert_protocol::<PingPongClient>();
    assert_protocol::<PingPongServer>();
    
    // Also verify that PingPongServer is the dual of PingPongClient
    fn assert_dual<P: Protocol, Q: Protocol>()
    where
        P::Dual: Protocol,
        Q: PartialEq<P::Dual>,
    {}
    assert_dual::<PingPongClient, PingPongServer>();
}