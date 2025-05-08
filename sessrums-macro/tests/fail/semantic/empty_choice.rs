use sessrums_macro::mpst;

// This test should fail with:
// error: Semantic error: Empty choice block. A choice must have at least one option.
// = help: Add at least one option to the choice block

mpst! {
    protocol EmptyChoice {
        participant Client;
        participant Server;
        
        Client -> Server: String;
        
        // Empty choice block with no options
        choice at Client {
            // No options defined
        }
        
        Server -> Client: String;
    }
}

fn main() {}