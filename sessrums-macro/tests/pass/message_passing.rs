//! Test for a protocol with various message types.
//!
//! This test verifies that the macro can correctly parse and process a protocol
//! definition with various message types, including primitive types, standard
//! library types, and custom types.

use sessrums_macro::mpst;

// Define some custom types for the protocol
struct Request {
    id: u32,
    content: String,
}

struct Response {
    id: u32,
    result: Result<String, String>,
}

enum Status {
    Ok,
    Error,
    Pending,
}

// Define a protocol with various message types
// The protocol consists of:
// 1. Client sends a Request to Server
// 2. Server sends a Response back to Client
// 3. Client sends a boolean acknowledgment to Server
// 4. Server sends a Status enum to Client
// 5. Client sends a tuple with mixed types to Server
mpst! {
    protocol ComplexMessages {
        // Define the participants
        participant Client;
        participant Server;

        // Define the message exchange with various types
        Client -> Server: Request;
        Server -> Client: Response;
        Client -> Server: bool;
        Server -> Client: Status;
        Client -> Server: (u32, String, bool);
    }
}

fn main() {
    // This is just a compile-time test, so we don't need to do anything here
    // The test passes if the macro expansion compiles successfully
    let _protocol = ComplexMessages::new();
}