use sessrums_macro::mpst;

// This test should fail with:
// error: Semantic error: Role 'Server' cannot make a choice in a branch where 'Client' is the deciding role.
// = help: In a choice block, the first message in each branch must be sent by the deciding role

mpst! {
    protocol InvalidChoiceRole {
        participant Client;
        participant Server;
        
        // Choice block where Client is the deciding role
        choice at Client {
            option Option1 {
                // First message in this branch is sent by Server, not Client
                Server -> Client: String;
                Client -> Server: String;
            }
            option Option2 {
                Client -> Server: String;
                Server -> Client: String;
            }
        }
    }
}

fn main() {}