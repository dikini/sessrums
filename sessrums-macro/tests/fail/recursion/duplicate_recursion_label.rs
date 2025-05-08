use sessrums_macro::mpst;

// This test should fail with:
// error: Recursion error: Duplicate recursion label 'Loop'. Each recursion label must be unique within its scope.
// = help: Choose a different label name for this recursion block

mpst! {
    protocol DuplicateRecursionLabel {
        participant Client;
        participant Server;
        
        // First recursion block with label 'Loop'
        rec Loop {
            Client -> Server: String;
            Server -> Client: String;
            
            // Second recursion block with the same label 'Loop'
            rec Loop {
                Client -> Server: String;
                Server -> Client: String;
                continue Loop;
            }
        }
    }
}

fn main() {}