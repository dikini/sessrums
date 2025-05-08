use sessrums_macro::mpst;

// This test should fail with:
// error: Participant error: Duplicate participant 'Client'. Each participant must be declared exactly once.
// = help: Remove the duplicate participant declaration

mpst! {
    protocol DuplicateParticipant {
        participant Client;
        participant Server;
        // Duplicate participant declaration
        participant Client;
        
        Client -> Server: String;
        Server -> Client: String;
    }
}

fn main() {}