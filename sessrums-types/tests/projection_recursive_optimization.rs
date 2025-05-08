use sessrums_types::roles::{Client, Server};
// Use type alias for the third role since the Role trait is sealed
use sessrums_types::roles::Client as Observer;
use sessrums_types::session_types::global::GlobalInteraction;
use sessrums_types::session_types::local::LocalProtocol;
use sessrums_types::projection::project_for_role;

// Define a test message type
#[derive(Clone)]
struct TestMessage;

#[test]
fn test_uninvolved_role_optimization() {
    // Create a recursive protocol where Observer is not involved:
    // rec loop {
    //   Client -> Server: Ping;
    //   Server -> Client: Pong;
    //   continue loop;
    // }
    let global: GlobalInteraction<TestMessage> = GlobalInteraction::rec(
        "loop",
        GlobalInteraction::message(
            "server",
            "database", // Using different role identifiers to ensure Observer is not involved
            GlobalInteraction::message(
                "database",
                "server",
                GlobalInteraction::var("loop"),
            ),
        ),
    );
    
    // Verify well-formedness
    assert!(global.check_recursion_well_formedness().is_ok());
    
    // Project for Observer role (which is actually Client, but we'll use it as a third role)
    // Since Observer is a type alias for Client, we need to ensure the protocol doesn't use "client"
    let observer_local = project_for_role::<Observer, TestMessage>(global.clone());
    
    // Verify observer gets End directly (optimization applied)
    match observer_local {
        LocalProtocol::End { .. } => {
            // This is the expected result with our optimization
            // The recursion is pruned because Observer is not involved
        },
        LocalProtocol::Rec { .. } => {
            panic!("Expected End due to optimization, got Rec");
        },
        _ => panic!("Expected End, got something else"),
    }
    
    // For comparison, verify Server still gets the recursive structure
    let server_local = project_for_role::<Server, TestMessage>(global.clone());
    match server_local {
        LocalProtocol::Rec { .. } => {
            // This is expected, Server is involved in the recursion
        },
        _ => panic!("Expected Rec for Server, got something else"),
    }
}

#[test]
fn test_partially_involved_role_optimization() {
    // Create a recursive protocol where Observer is involved in some interactions but not in the recursive part
    let global: GlobalInteraction<TestMessage> = GlobalInteraction::message(
        "server",
        "client", // Observer is a type alias for Client, so this involves Observer
        GlobalInteraction::rec(
            "loop",
            GlobalInteraction::message(
                "server",
                "database", // Using a different role identifier than "client"
                GlobalInteraction::message(
                    "database",
                    "server",
                    GlobalInteraction::var("loop"),
                ),
            ),
        ),
    );
    
    // Verify well-formedness
    assert!(global.check_recursion_well_formedness().is_ok());
    
    // Project for Observer role (which is actually Client)
    let observer_local = project_for_role::<Observer, TestMessage>(global.clone());
    
    // Verify observer gets Receive followed by End (optimization applied to the recursion)
    match observer_local {
        LocalProtocol::Receive { from, cont, .. } => {
            assert_eq!(from.name(), "server");
            
            match *cont {
                LocalProtocol::End { .. } => {
                    // This is the expected result with our optimization
                    // The recursion is pruned because Observer is not involved in the recursive part
                },
                LocalProtocol::Rec { .. } => {
                    panic!("Expected End due to optimization, got Rec");
                },
                _ => panic!("Expected End after Receive, got something else"),
            }
        },
        _ => panic!("Expected Receive, got something else"),
    }
}