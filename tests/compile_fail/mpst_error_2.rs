//! Test that a global protocol with an unknown role in projection fails to compile.

use sessrums::proto::global::{GSend, GEnd};
use sessrums::proto::roles::{Role, RoleA, RoleB};
use sessrums::proto::projection::Project;

// Define roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Server;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Logger;

impl Role for Client {
    fn name(&self) -> &'static str {
        "Client"
    }
}

impl Role for Server {
    fn name(&self) -> &'static str {
        "Server"
    }
}

impl Role for Logger {
    fn name(&self) -> &'static str {
        "Logger"
    }
}

fn main() {
    // Define a global protocol: Client sends a String to Server, then ends
    type GlobalProtocol = GSend<String, Client, Server, GEnd>;
    
    // This should fail to compile because Logger is not involved in the protocol
    let _local_protocol = <GlobalProtocol as Project<Logger>>::LocalProtocol;
}