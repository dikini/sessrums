use sessrums_macro::mpst;

// This test should fail with:
// error: Recursion error: Undefined recursion label 'Loop'. Labels must be defined with 'rec Loop' before they can be referenced with 'continue Loop'.
// = help: Define the recursion label with 'rec Loop { ... }' before using 'continue Loop'

mpst! {
    protocol UndefinedRecursionLabel {
        participant Client;
        participant Server;
        
        Client -> Server: String;
        Server -> Client: String;
        
        // Continue to an undefined recursion label
        continue Loop;
    }
}

fn main() {}