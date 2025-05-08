use sessrums_macro::mpst;

// This test should fail with:
// error: Syntax error: Invalid message arrow syntax. Expected 'Sender -> Receiver: Type;'
// = help: Use the '->' arrow syntax for message passing

mpst! {
    protocol InvalidMessageArrow {
        participant Client;
        participant Server;
        
        // Invalid arrow syntax (using => instead of ->)
        Client => Server: String;
        Server -> Client: String;
    }
}

fn main() {}