//! Test for a protocol with participants that have aliases.
//!
//! This test verifies that the macro can correctly parse and process a protocol
//! definition where participants have aliases, which can be useful for more
//! descriptive role names in complex protocols.

use sessrums_macro::mpst;

// Define a protocol with participants that have aliases
// The protocol consists of:
// 1. A Client (aliased as "C") sends a request to a Server (aliased as "S")
// 2. The Server responds back to the Client
mpst! {
    protocol RequestResponse {
        // Define the participants with aliases
        participant Client as C;
        participant Server as S;

        // Define the message exchange using the aliases
        C -> S: String;
        S -> C: String;
    }
}

fn main() {
    // This is just a compile-time test, so we don't need to do anything here
    // The test passes if the macro expansion compiles successfully
    let _protocol = RequestResponse::new();
}