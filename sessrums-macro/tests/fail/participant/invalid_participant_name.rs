use sessrums_macro::mpst;

// This test should fail with:
// error: Participant error: Invalid participant name '123Client'. Participant names must be valid Rust identifiers.
// = help: Use a valid Rust identifier for the participant name (must start with a letter or underscore)

mpst! {
    protocol InvalidParticipantName {
        // Invalid participant name (starts with a number)
        participant 123Client;
        participant Server;
        
        123Client -> Server: String;
        Server -> 123Client: String;
    }
}

fn main() {}