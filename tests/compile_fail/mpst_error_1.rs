//! Test that a global protocol with the same role as sender and receiver fails to compile.

use sessrums::proto::global::{GSend, GEnd};
use sessrums::proto::roles::{Role, RoleA};

// Define a role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Client;

impl Role for Client {
    fn name(&self) -> &'static str {
        "Client"
    }
}

fn main() {
    // This should fail to compile because the sender and receiver are the same role
    let protocol = GSend::<String, Client, Client, GEnd>::new();
    protocol.validate().unwrap(); // This should fail at compile time
}