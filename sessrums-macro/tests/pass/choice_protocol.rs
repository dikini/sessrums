//! Test for a protocol with choice blocks.
//!
//! This test verifies that the macro can correctly parse and process a protocol
//! definition with choice blocks using the `choice at Role { option Label { ... } ... }` syntax.
//! 
//! The protocol demonstrates a client-server interaction where:
//! 1. The client can choose between different types of requests
//! 2. The server responds differently based on the client's choice

use sessrums_macro::mpst;

// Define custom types for the protocol
struct QueryRequest {
    query: String,
}

struct QueryResponse {
    results: Vec<String>,
}

struct UpdateRequest {
    id: u32,
    data: String,
}

struct UpdateResponse {
    success: bool,
    message: String,
}

// Define a protocol with choice blocks where:
// - Client can choose between sending a query or an update request
// - Server responds with the appropriate response type based on the client's choice
mpst! {
    protocol ChoiceProtocol {
        // Define the participants
        participant Client;
        participant Server;

        // Define the choice interaction
        choice at Client {
            // First option: Client sends a query request
            option Query {
                Client -> Server: QueryRequest;
                Server -> Client: QueryResponse;
            }
            
            // Second option: Client sends an update request
            option Update {
                Client -> Server: UpdateRequest;
                Server -> Client: UpdateResponse;
            }
        }
    }
}

fn main() {
    // This is just a compile-time test, so we don't need to do anything here
    // The test passes if the macro expansion compiles successfully
    let _protocol = ChoiceProtocol::new();
}