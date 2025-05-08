use sessrums_macro::mpst;

// This test should fail with:
// error: Syntax error: Invalid choice syntax. Expected 'choice at Role { option Label { ... } ... }'.
// = help: Use 'choice at Role { ... }' syntax

mpst! {
    protocol InvalidChoiceSyntax {
        participant Client;
        participant Server;
        
        // Invalid choice syntax - missing 'at' keyword
        choice Client {
            option Option1 {
                Client -> Server: String;
            }
        }
    }
}

fn main() {}