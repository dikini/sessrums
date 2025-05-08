use sessrums_macro::mpst;

// This test should fail with:
// error: Syntax error: Expected ';' after message declaration
// = help: Add a semicolon after the message type

mpst! {
    protocol MissingSemicolon {
        participant Client;
        participant Server;
        
        // Missing semicolon after message type
        Client -> Server: String
        Server -> Client: String;
    }
}

fn main() {}