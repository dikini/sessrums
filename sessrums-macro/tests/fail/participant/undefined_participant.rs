use sessrums_macro::mpst;

// This test should fail with:
// error: Participant error: Undefined participant 'Database'. All participants must be declared at the beginning of the protocol.
// = help: Add 'participant Database;' to the beginning of the protocol

mpst! {
    protocol QueryProtocol {
        participant Client;
        participant Server;
        
        // Database is not declared as a participant
        Client -> Database: Query;
        Database -> Client: Response;
    }
}

fn main() {}