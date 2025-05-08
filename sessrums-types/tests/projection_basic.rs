use std::marker::PhantomData;
use sessrums_types::roles::{Client, Server};
use sessrums_types::session_types::common::RoleIdentifier;
use sessrums_types::session_types::global::GlobalInteraction;
use sessrums_types::session_types::local::LocalProtocol;
use sessrums_types::projection::{project_for_role, RoleExt};

// Define a test message type
#[derive(Clone)]
struct TestMessage;

#[test]
fn test_message_projection() {
    // Create a simple global protocol: A -> B: Msg; End
    let global = GlobalInteraction::message(
        "client",
        "server",
        GlobalInteraction::end(),
    );
    
    // Project for role Client (sender)
    let client_local = project_for_role::<Client, TestMessage>(global.clone());
    
    // Verify client gets a Send
    match client_local {
        LocalProtocol::Send { to, cont, .. } => {
            assert_eq!(to.name(), "server");
            match *cont {
                LocalProtocol::End { .. } => {},
                _ => panic!("Expected End, got something else"),
            }
        },
        _ => panic!("Expected Send, got something else"),
    }
    
    // Project for role Server (receiver)
    let server_local = project_for_role::<Server, TestMessage>(global.clone());
    
    // Verify server gets a Receive
    match server_local {
        LocalProtocol::Receive { from, cont, .. } => {
            assert_eq!(from.name(), "client");
            match *cont {
                LocalProtocol::End { .. } => {},
                _ => panic!("Expected End, got something else"),
            }
        },
        _ => panic!("Expected Receive, got something else"),
    }
}

// We'll skip the uninvolved role test for now since we can't easily implement
// RoleExt for external types due to Rust's orphan rules

#[test]
fn test_multi_message_projection() {
    // Create a protocol: Client -> Server: Msg1; Server -> Client: Msg2; End
    let global = GlobalInteraction::message(
        "client",
        "server",
        GlobalInteraction::message(
            "server",
            "client",
            GlobalInteraction::end(),
        ),
    );
    
    // Project for Client
    let client_local = project_for_role::<Client, TestMessage>(global.clone());
    
    // Verify Client gets: Send to Server; Receive from Server; End
    match client_local {
        LocalProtocol::Send { to, cont, .. } => {
            assert_eq!(to.name(), "server");
            match *cont {
                LocalProtocol::Receive { from, cont, .. } => {
                    assert_eq!(from.name(), "server");
                    match *cont {
                        LocalProtocol::End { .. } => {},
                        _ => panic!("Expected End, got something else"),
                    }
                },
                _ => panic!("Expected Receive, got something else"),
            }
        },
        _ => panic!("Expected Send, got something else"),
    }
    
    // Project for Server
    let server_local = project_for_role::<Server, TestMessage>(global.clone());
    
    // Verify Server gets: Receive from Client; Send to Client; End
    match server_local {
        LocalProtocol::Receive { from, cont, .. } => {
            assert_eq!(from.name(), "client");
            match *cont {
                LocalProtocol::Send { to, cont, .. } => {
                    assert_eq!(to.name(), "client");
                    match *cont {
                        LocalProtocol::End { .. } => {},
                        _ => panic!("Expected End, got something else"),
                    }
                },
                _ => panic!("Expected Send, got something else"),
            }
        },
        _ => panic!("Expected Receive, got something else"),
    }
}

#[test]
fn test_end_projection() {
    // Create a simple End protocol
    let global = GlobalInteraction::<TestMessage>::end();
    
    // Project for Client
    let client_local = project_for_role::<Client, TestMessage>(global.clone());
    
    // Verify Client gets End
    match client_local {
        LocalProtocol::End { .. } => {},
        _ => panic!("Expected End, got something else"),
    }
    
    // Project for Server
    let server_local = project_for_role::<Server, TestMessage>(global.clone());
    
    // Verify Server gets End
    match server_local {
        LocalProtocol::End { .. } => {},
        _ => panic!("Expected End, got something else"),
    }
}