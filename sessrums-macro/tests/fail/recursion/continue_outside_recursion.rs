use sessrums_macro::mpst;

// This test should fail with:
// error: Recursion error: Continue statement refers to label 'Loop' which is not active in this scope
// = help: Make sure the continue statement is within the scope of the referenced recursion label

mpst! {
    protocol ContinueOutsideRecursion {
        participant Client;
        participant Server;
        
        // Define a recursion block
        rec Loop {
            Client -> Server: String;
            Server -> Client: String;
        }
        
        // Continue statement outside the recursion block
        continue Loop;
    }
}

fn main() {}