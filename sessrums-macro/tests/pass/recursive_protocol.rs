//! Test for a protocol with recursion.
//!
//! This test verifies that the macro can correctly parse and process a protocol
//! definition with recursion using the `rec` and `continue` keywords.
//! 
//! The protocol demonstrates a simple client-server interaction where:
//! 1. The client can repeatedly send requests to the server
//! 2. The server responds to each request
//! 3. The client can choose to continue the interaction or end it

use sessrums_macro::mpst;

// Define a simple recursive protocol where:
// - Client sends a request to Server
// - Server responds to Client
// - Client can choose to continue the interaction or end it
mpst! {
    protocol RecursiveProtocol {
        // Define the participants
        participant Client;
        participant Server;

        // Define the recursive interaction
        rec Loop {
            // Client sends a request to Server
            Client -> Server: String;
            // Server responds to Client
            Server -> Client: String;
            // Continue the recursion (loop back)
            continue Loop;
        }
    }
}

fn main() {
    // This is just a compile-time test, so we don't need to do anything here
    // The test passes if the macro expansion compiles successfully
    let _protocol = RecursiveProtocol::new();
}