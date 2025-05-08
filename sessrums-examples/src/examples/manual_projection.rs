//! Manual Protocol Definition and Projection Example
//!
//! This example demonstrates how to manually define a global protocol and project it
//! to local protocols for each role. It includes:
//!
//! 1. A simple 3-party global protocol definition
//! 2. Manual projections to local protocols for each role
//! 3. Message type definitions
//! 4. Verification functions to check projection correctness
//!
//! The protocol models a simple client-server-storage interaction where:
//! - Client sends a Request to Server
//! - Server processes the request and sends a Response to Client
//! - Server also sends a Log to Storage
//! - All parties terminate

use sessrums_types::roles::{Client, Server};
use sessrums_types::session_types::{
    GlobalInteraction,
    LocalProtocol,
};

/// Type alias for the Storage role
///
/// Since we can't implement the sealed Role trait from outside the crate,
/// we'll use the existing Server role as a type parameter for the Storage role.
pub type Storage = Server;

/// Message types for our protocol
pub mod messages {
    /// Request message sent from Client to Server
    #[derive(Debug, Clone)]
    pub struct Request {
        pub content: String,
    }

    /// Response message sent from Server to Client
    #[derive(Debug, Clone)]
    pub struct Response {
        pub content: String,
        pub status: u32,
    }

    /// Log message sent from Server to Storage
    #[derive(Debug, Clone)]
    pub struct Log {
        pub request_id: String,
        pub timestamp: u64,
        pub details: String,
    }
}

/// Defines the global protocol for our 3-party interaction
///
/// The protocol flow is:
/// 1. Client sends Request to Server
/// 2. Server sends Response to Client
/// 3. Server sends Log to Storage
/// 4. All parties terminate
pub fn define_global_protocol() -> GlobalInteraction<messages::Request> {
    GlobalInteraction::message(
        "client",
        "server",
        GlobalInteraction::message(
            "server",
            "client",
            GlobalInteraction::message(
                "server",
                "storage",
                GlobalInteraction::end(),
            ),
        ),
    )
}

/// Manually projects the global protocol to the Client's local protocol
///
/// From the Client's perspective:
/// 1. Send Request to Server
/// 2. Receive Response from Server
/// 3. End
pub fn project_to_client() -> LocalProtocol<Client, messages::Request> {
    LocalProtocol::<Client, messages::Request>::send(
        "server",
        LocalProtocol::<Client, messages::Request>::receive(
            "server",
            LocalProtocol::<Client, messages::Request>::end(),
        ),
    )
}

/// Manually projects the global protocol to the Server's local protocol
///
/// From the Server's perspective:
/// 1. Receive Request from Client
/// 2. Send Response to Client
/// 3. Send Log to Storage
/// 4. End
pub fn project_to_server() -> LocalProtocol<Server, messages::Request> {
    LocalProtocol::<Server, messages::Request>::receive(
        "client",
        LocalProtocol::<Server, messages::Request>::send(
            "client",
            LocalProtocol::<Server, messages::Request>::send(
                "storage",
                LocalProtocol::<Server, messages::Request>::end(),
            ),
        ),
    )
}

/// Manually projects the global protocol to the Storage's local protocol
///
/// From the Storage's perspective:
/// 1. Receive Log from Server
/// 2. End
pub fn project_to_storage() -> LocalProtocol<Storage, messages::Request> {
    LocalProtocol::<Storage, messages::Request>::receive(
        "server",
        LocalProtocol::<Storage, messages::Request>::end(),
    )
}

/// Verifies that the manual projections correctly reflect the global protocol
///
/// This function checks that each local protocol correctly represents the
/// global protocol from that role's perspective by:
/// 1. Ensuring all expected interactions are present
/// 2. Verifying the order of interactions
/// 3. Confirming that each role only sees interactions it participates in
pub fn verify_projections() -> bool {
    // Get the local protocols
    let client_local = project_to_client();
    let server_local = project_to_server();
    let storage_local = project_to_storage();
    
    // Verify Client projection
    match client_local {
        LocalProtocol::Send { to, cont, .. } => {
            // First action should be sending to server
            if to.name() != "server" {
                return false;
            }
            
            // Second action should be receiving from server
            match *cont {
                LocalProtocol::Receive { from, cont, .. } => {
                    if from.name() != "server" {
                        return false;
                    }
                    
                    // Should end after receiving
                    match *cont {
                        LocalProtocol::End { .. } => {},
                        _ => return false,
                    }
                },
                _ => return false,
            }
        },
        _ => return false,
    }
    
    // Verify Server projection
    match server_local {
        LocalProtocol::Receive { from, cont, .. } => {
            // First action should be receiving from client
            if from.name() != "client" {
                return false;
            }
            
            // Second action should be sending to client
            match *cont {
                LocalProtocol::Send { to, cont, .. } => {
                    if to.name() != "client" {
                        return false;
                    }
                    
                    // Third action should be sending to storage
                    match *cont {
                        LocalProtocol::Send { to, cont, .. } => {
                            if to.name() != "storage" {
                                return false;
                            }
                            
                            // Should end after sending to storage
                            match *cont {
                                LocalProtocol::End { .. } => {},
                                _ => return false,
                            }
                        },
                        _ => return false,
                    }
                },
                _ => return false,
            }
        },
        _ => return false,
    }
    
    // Verify Storage projection
    match storage_local {
        LocalProtocol::Receive { from, cont, .. } => {
            // First action should be receiving from server
            if from.name() != "server" {
                return false;
            }
            
            // Should end after receiving
            match *cont {
                LocalProtocol::End { .. } => {},
                _ => return false,
            }
        },
        _ => return false,
    }
    
    // All projections verified successfully
    true
}

/// Verifies that the projections preserve causal dependencies between messages
///
/// This function checks that the causal ordering of messages in the global protocol
/// is preserved in the local projections. Specifically:
/// 1. Server can only send Response to Client after receiving Request
/// 2. Server can only send Log to Storage after receiving Request
pub fn verify_causal_dependencies() -> bool {
    // Verify Server's local protocol preserves causal dependencies
    match project_to_server() {
        LocalProtocol::Receive { from: client, cont, .. } => {
            if client.name() != "client" {
                return false;
            }
            
            // After receiving from client, server should send to client first
            match *cont {
                LocalProtocol::Send { to: client_resp, cont, .. } => {
                    if client_resp.name() != "client" {
                        return false;
                    }
                    
                    // Then send to storage
                    match *cont {
                        LocalProtocol::Send { to: storage, .. } => {
                            if storage.name() != "storage" {
                                return false;
                            }
                            // Causal dependencies preserved
                            true
                        },
                        _ => false,
                    }
                },
                _ => false,
            }
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_projections() {
        assert!(verify_projections());
    }

    #[test]
    fn test_verify_causal_dependencies() {
        assert!(verify_causal_dependencies());
    }
}