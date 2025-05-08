use sessrums_macro::mpst;

// This test should fail with:
// error: Type error: Invalid message type 'NonExistentType'. Message types must be valid Rust types.
// = help: Use a fully qualified path or import the type with 'use'

mpst! {
    protocol InvalidMessageType {
        participant Client;
        participant Server;
        
        // Using a non-existent type
        Client -> Server: NonExistentType;
        Server -> Client: String;
    }
}

fn main() {}