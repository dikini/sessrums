//! Test for a simple protocol with two participants and basic message passing.
//!
//! This test verifies that the macro can correctly parse and process a simple
//! protocol definition with two participants and basic message passing.

use sessrums_macro::mpst;

// Define a simple protocol with two participants: Client and Server
// The protocol consists of a simple message exchange where:
// 1. Client sends a String message to Server
// 2. Server sends a String message back to Client
mpst! {
    protocol PingPong {
        // Define the participants
        participant Client;
        participant Server;

        // Define the message exchange
        Client -> Server: String;
        Server -> Client: String;
    }
}

fn main() {
    // This is just a compile-time test, so we don't need to do anything here
    // The test passes if the macro expansion compiles successfully
    let _protocol = PingPong::new();
}