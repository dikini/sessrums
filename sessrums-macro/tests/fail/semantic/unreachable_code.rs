use sessrums_macro::mpst;

// This test should fail with:
// error: Semantic error: Unreachable code after 'continue' statement
// = help: Remove code after 'continue' statement or move it before the 'continue'

mpst! {
    protocol UnreachableCode {
        participant Client;
        participant Server;
        
        rec Loop {
            Client -> Server: String;
            
            // Continue statement followed by unreachable code
            continue Loop;
            
            // This code is unreachable
            Server -> Client: String;
        }
    }
}

fn main() {}