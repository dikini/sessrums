use sessrums_macro::mpst;
use std::collections::HashMap;

// This test should fail with:
// error: Type error: Unsupported generic type 'HashMap<String, Vec<u32>>'. Complex generic types may not be fully supported.
// = help: Consider using a simpler type or creating a type alias

mpst! {
    protocol UnsupportedGenericType {
        participant Client;
        participant Server;
        
        // Using a complex generic type
        Client -> Server: HashMap<String, Vec<u32>>;
        Server -> Client: String;
    }
}

fn main() {}