use std::marker::PhantomData;
use sessrums_types::roles::{Client, Server};
use sessrums_types::session_types::common::{RoleIdentifier, RecursionLabel};
use sessrums_types::session_types::global::GlobalInteraction;
use sessrums_types::session_types::local::LocalProtocol;
use sessrums_types::projection::project_for_role;

// Define a test message type
#[derive(Clone)]
struct TestMessage;

#[test]
fn test_recursive_protocol_projection() {
    // Create a recursive ping-pong protocol:
    // rec loop {
    //   Client -> Server: Ping;
    //   Server -> Client: Pong;
    //   continue loop;
    // }
    let global = GlobalInteraction::rec(
        "loop",
        GlobalInteraction::message(
            "client",
            "server",
            GlobalInteraction::message(
                "server",
                "client",
                GlobalInteraction::var("loop"),
            ),
        ),
    );
    
    // Verify well-formedness
    assert!(global.check_recursion_well_formedness().is_ok());
    
    // Project for Client role
    let client_local = project_for_role::<Client, TestMessage>(global.clone());
    
    // Verify client gets: Rec { Send, Receive, Var }
    match client_local {
        LocalProtocol::Rec { label, body, .. } => {
            assert_eq!(label.name(), "loop");
            
            match *body {
                LocalProtocol::Send { to, cont, .. } => {
                    assert_eq!(to.name(), "server");
                    
                    match *cont {
                        LocalProtocol::Receive { from, cont, .. } => {
                            assert_eq!(from.name(), "server");
                            
                            match *cont {
                                LocalProtocol::Var { label, .. } => {
                                    assert_eq!(label.name(), "loop");
                                },
                                _ => panic!("Expected Var, got something else"),
                            }
                        },
                        _ => panic!("Expected Receive, got something else"),
                    }
                },
                _ => panic!("Expected Send, got something else"),
            }
        },
        _ => panic!("Expected Rec, got something else"),
    }
    
    // Project for Server role
    let server_local = project_for_role::<Server, TestMessage>(global.clone());
    
    // Verify server gets: Rec { Receive, Send, Var }
    match server_local {
        LocalProtocol::Rec { label, body, .. } => {
            assert_eq!(label.name(), "loop");
            
            match *body {
                LocalProtocol::Receive { from, cont, .. } => {
                    assert_eq!(from.name(), "client");
                    
                    match *cont {
                        LocalProtocol::Send { to, cont, .. } => {
                            assert_eq!(to.name(), "client");
                            
                            match *cont {
                                LocalProtocol::Var { label, .. } => {
                                    assert_eq!(label.name(), "loop");
                                },
                                _ => panic!("Expected Var, got something else"),
                            }
                        },
                        _ => panic!("Expected Send, got something else"),
                    }
                },
                _ => panic!("Expected Receive, got something else"),
            }
        },
        _ => panic!("Expected Rec, got something else"),
    }
}

#[test]
fn test_recursive_protocol_with_choice_projection() {
    // Create a recursive ping-pong protocol with a choice to stop:
    // rec loop {
    //   Client -> Server: Ping;
    //   Server -> Client: Pong;
    //   choice at Client {
    //     Client -> Server: Continue;
    //     continue loop;
    //   } or {
    //     Client -> Server: Stop;
    //     end;
    //   }
    // }
    let global = GlobalInteraction::rec(
        "loop",
        GlobalInteraction::message(
            "client",
            "server",
            GlobalInteraction::message(
                "server",
                "client",
                GlobalInteraction::choice(
                    "client",
                    vec![
                        ("continue".into(), GlobalInteraction::message(
                            "client",
                            "server",
                            GlobalInteraction::var("loop"),
                        )),
                        ("stop".into(), GlobalInteraction::message(
                            "client",
                            "server",
                            GlobalInteraction::end(),
                        )),
                    ],
                ),
            ),
        ),
    );
    
    // Verify well-formedness
    assert!(global.check_recursion_well_formedness().is_ok());
    
    // Project for Client role
    let client_local = project_for_role::<Client, TestMessage>(global.clone());
    
    // Verify client gets: Rec { Send, Receive, Select { Send+Var, Send+End } }
    match client_local {
        LocalProtocol::Rec { label, body, .. } => {
            assert_eq!(label.name(), "loop");
            
            // Check first Send
            match *body {
                LocalProtocol::Send { to, cont, .. } => {
                    assert_eq!(to.name(), "server");
                    
                    // Check Receive
                    match *cont {
                        LocalProtocol::Receive { from, cont, .. } => {
                            assert_eq!(from.name(), "server");
                            
                            // Check Select
                            match *cont {
                                LocalProtocol::Select { branches, .. } => {
                                    assert_eq!(branches.len(), 2);
                                    
                                    // Find continue branch
                                    let continue_branch = branches.iter()
                                        .find(|(label, _)| label.name() == "continue")
                                        .expect("Continue branch not found");
                                    
                                    // Find stop branch
                                    let stop_branch = branches.iter()
                                        .find(|(label, _)| label.name() == "stop")
                                        .expect("Stop branch not found");
                                    
                                    // Check continue branch: Send -> Var
                                    match &*continue_branch.1 {
                                        LocalProtocol::Send { to, cont, .. } => {
                                            assert_eq!(to.name(), "server");
                                            
                                            match &**cont {
                                                LocalProtocol::Var { label, .. } => {
                                                    assert_eq!(label.name(), "loop");
                                                },
                                                _ => panic!("Expected Var in continue branch"),
                                            }
                                        },
                                        _ => panic!("Expected Send in continue branch"),
                                    }
                                    
                                    // Check stop branch: Send -> End
                                    match &*stop_branch.1 {
                                        LocalProtocol::Send { to, cont, .. } => {
                                            assert_eq!(to.name(), "server");
                                            
                                            match &**cont {
                                                LocalProtocol::End { .. } => {},
                                                _ => panic!("Expected End in stop branch"),
                                            }
                                        },
                                        _ => panic!("Expected Send in stop branch"),
                                    }
                                },
                                _ => panic!("Expected Select, got something else"),
                            }
                        },
                        _ => panic!("Expected Receive, got something else"),
                    }
                },
                _ => panic!("Expected Send, got something else"),
            }
        },
        _ => panic!("Expected Rec, got something else"),
    }
    
    // Project for Server role
    let server_local = project_for_role::<Server, TestMessage>(global.clone());
    
    // Verify server gets: Rec { Receive, Send, Offer { Receive+Var, Receive+End } }
    match server_local {
        LocalProtocol::Rec { label, body, .. } => {
            assert_eq!(label.name(), "loop");
            
            // Check first Receive
            match *body {
                LocalProtocol::Receive { from, cont, .. } => {
                    assert_eq!(from.name(), "client");
                    
                    // Check Send
                    match *cont {
                        LocalProtocol::Send { to, cont, .. } => {
                            assert_eq!(to.name(), "client");
                            
                            // Check Offer
                            match *cont {
                                LocalProtocol::Offer { decider, branches, .. } => {
                                    assert_eq!(decider.name(), "client");
                                    assert_eq!(branches.len(), 2);
                                    
                                    // Find continue branch
                                    let continue_branch = branches.iter()
                                        .find(|(label, _)| label.name() == "continue")
                                        .expect("Continue branch not found");
                                    
                                    // Find stop branch
                                    let stop_branch = branches.iter()
                                        .find(|(label, _)| label.name() == "stop")
                                        .expect("Stop branch not found");
                                    
                                    // Check continue branch: Receive -> Var
                                    match &*continue_branch.1 {
                                        LocalProtocol::Receive { from, cont, .. } => {
                                            assert_eq!(from.name(), "client");
                                            
                                            match &**cont {
                                                LocalProtocol::Var { label, .. } => {
                                                    assert_eq!(label.name(), "loop");
                                                },
                                                _ => panic!("Expected Var in continue branch"),
                                            }
                                        },
                                        _ => panic!("Expected Receive in continue branch"),
                                    }
                                    
                                    // Check stop branch: Receive -> End
                                    match &*stop_branch.1 {
                                        LocalProtocol::Receive { from, cont, .. } => {
                                            assert_eq!(from.name(), "client");
                                            
                                            match &**cont {
                                                LocalProtocol::End { .. } => {},
                                                _ => panic!("Expected End in stop branch"),
                                            }
                                        },
                                        _ => panic!("Expected Receive in stop branch"),
                                    }
                                },
                                _ => panic!("Expected Offer, got something else"),
                            }
                        },
                        _ => panic!("Expected Send, got something else"),
                    }
                },
                _ => panic!("Expected Receive, got something else"),
            }
        },
        _ => panic!("Expected Rec, got something else"),
    }
}