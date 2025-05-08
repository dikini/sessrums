use sessrums_types::roles::{Client, Server};
use sessrums_types::session_types::common::Label;
use sessrums_types::session_types::global::GlobalInteraction;
use sessrums_types::session_types::local::LocalProtocol;
use sessrums_types::projection::project_for_role;

// Define test message types
#[derive(Clone)]
struct Msg1;
#[derive(Clone)]
struct Msg2;

#[test]
fn test_choice_projection() {
    // Create a global protocol with choice: 
    // choice at Client { Client -> Server: Msg1; End } or { Client -> Server: Msg2; End }
    let global = GlobalInteraction::choice(
        "client",
        vec![
            ("option1".into(), GlobalInteraction::message(
                "client",
                "server",
                GlobalInteraction::end(),
            )),
            ("option2".into(), GlobalInteraction::message(
                "client",
                "server",
                GlobalInteraction::end(),
            )),
        ],
    );
    
    // Project for role Client (decider)
    let client_local = project_for_role::<Client, Msg1>(global.clone());
    
    // Verify client gets a Select with the correct branches
    match client_local {
        LocalProtocol::Select { branches, .. } => {
            assert_eq!(branches.len(), 2);
            
            // Check first branch
            let (label1, cont1) = &branches[0];
            assert_eq!(label1.name(), "option1");
            match &**cont1 {
                LocalProtocol::Send { to, cont, .. } => {
                    assert_eq!(to.name(), "server");
                    match &**cont {
                        LocalProtocol::End { .. } => {},
                        _ => panic!("Expected End, got something else"),
                    }
                },
                _ => panic!("Expected Send, got something else"),
            }
            
            // Check second branch
            let (label2, cont2) = &branches[1];
            assert_eq!(label2.name(), "option2");
            match &**cont2 {
                LocalProtocol::Send { to, cont, .. } => {
                    assert_eq!(to.name(), "server");
                    match &**cont {
                        LocalProtocol::End { .. } => {},
                        _ => panic!("Expected End, got something else"),
                    }
                },
                _ => panic!("Expected Send, got something else"),
            }
        },
        _ => panic!("Expected Select, got something else"),
    }
    
    // Project for role Server (participant)
    let server_local = project_for_role::<Server, Msg1>(global.clone());
    
    // Verify server gets an Offer with the correct branches
    match server_local {
        LocalProtocol::Offer { decider, branches, .. } => {
            assert_eq!(decider.name(), "client");
            assert_eq!(branches.len(), 2);
            
            // Check first branch
            let (label1, cont1) = &branches[0];
            assert_eq!(label1.name(), "option1");
            match &**cont1 {
                LocalProtocol::Receive { from, cont, .. } => {
                    assert_eq!(from.name(), "client");
                    match &**cont {
                        LocalProtocol::End { .. } => {},
                        _ => panic!("Expected End, got something else"),
                    }
                },
                _ => panic!("Expected Receive, got something else"),
            }
            
            // Check second branch
            let (label2, cont2) = &branches[1];
            assert_eq!(label2.name(), "option2");
            match &**cont2 {
                LocalProtocol::Receive { from, cont, .. } => {
                    assert_eq!(from.name(), "client");
                    match &**cont {
                        LocalProtocol::End { .. } => {},
                        _ => panic!("Expected End, got something else"),
                    }
                },
                _ => panic!("Expected Receive, got something else"),
            }
        },
        _ => panic!("Expected Offer, got something else"),
    }
}

#[test]
fn test_nested_choice_projection() {
    // Create a protocol with nested choice:
    // choice at Client {
    //   Client -> Server: Msg1;
    //   choice at Server {
    //     Server -> Client: Msg1; End
    //   } or {
    //     Server -> Client: Msg2; End
    //   }
    // } or {
    //   Client -> Server: Msg2; End
    // }
    let global = GlobalInteraction::choice(
        "client",
        vec![
            ("option1".into(), GlobalInteraction::message(
                "client",
                "server",
                GlobalInteraction::choice(
                    "server",
                    vec![
                        ("nested1".into(), GlobalInteraction::message(
                            "server",
                            "client",
                            GlobalInteraction::end(),
                        )),
                        ("nested2".into(), GlobalInteraction::message(
                            "server",
                            "client",
                            GlobalInteraction::end(),
                        )),
                    ],
                ),
            )),
            ("option2".into(), GlobalInteraction::message(
                "client",
                "server",
                GlobalInteraction::end(),
            )),
        ],
    );
    
    // Project for role Client
    let client_local = project_for_role::<Client, Msg1>(global.clone());
    
    // Verify client gets a Select with the correct structure
    match client_local {
        LocalProtocol::Select { branches, .. } => {
            assert_eq!(branches.len(), 2);
            
            // Check first branch (with nested choice)
            let (label1, cont1) = &branches[0];
            assert_eq!(label1.name(), "option1");
            match &**cont1 {
                LocalProtocol::Send { to, cont, .. } => {
                    assert_eq!(to.name(), "server");
                    match &**cont {
                        LocalProtocol::Offer { decider, branches, .. } => {
                            assert_eq!(decider.name(), "server");
                            assert_eq!(branches.len(), 2);
                            // Check nested branches
                            for (_, nested_cont) in branches {
                                match &**nested_cont {
                                    LocalProtocol::Receive { from, .. } => {
                                        assert_eq!(from.name(), "server");
                                    },
                                    _ => panic!("Expected Receive, got something else"),
                                }
                            }
                        },
                        _ => panic!("Expected Offer, got something else"),
                    }
                },
                _ => panic!("Expected Send, got something else"),
            }
            
            // Check second branch
            let (label2, cont2) = &branches[1];
            assert_eq!(label2.name(), "option2");
            match &**cont2 {
                LocalProtocol::Send { to, cont, .. } => {
                    assert_eq!(to.name(), "server");
                    match &**cont {
                        LocalProtocol::End { .. } => {},
                        _ => panic!("Expected End, got something else"),
                    }
                },
                _ => panic!("Expected Send, got something else"),
            }
        },
        _ => panic!("Expected Select, got something else"),
    }
    
    // Project for role Server
    let server_local = project_for_role::<Server, Msg1>(global.clone());
    
    // Verify server gets an Offer with the correct structure
    match server_local {
        LocalProtocol::Offer { decider, branches, .. } => {
            assert_eq!(decider.name(), "client");
            assert_eq!(branches.len(), 2);
            
            // Check first branch (with nested choice)
            let (label1, cont1) = &branches[0];
            assert_eq!(label1.name(), "option1");
            match &**cont1 {
                LocalProtocol::Receive { from, cont, .. } => {
                    assert_eq!(from.name(), "client");
                    match &**cont {
                        LocalProtocol::Select { branches, .. } => {
                            assert_eq!(branches.len(), 2);
                            // Check nested branches
                            for (_, nested_cont) in branches {
                                match &**nested_cont {
                                    LocalProtocol::Send { to, .. } => {
                                        assert_eq!(to.name(), "client");
                                    },
                                    _ => panic!("Expected Send, got something else"),
                                }
                            }
                        },
                        _ => panic!("Expected Select, got something else"),
                    }
                },
                _ => panic!("Expected Receive, got something else"),
            }
            
            // Check second branch
            let (label2, cont2) = &branches[1];
            assert_eq!(label2.name(), "option2");
            match &**cont2 {
                LocalProtocol::Receive { from, cont, .. } => {
                    assert_eq!(from.name(), "client");
                    match &**cont {
                        LocalProtocol::End { .. } => {},
                        _ => panic!("Expected End, got something else"),
                    }
                },
                _ => panic!("Expected Receive, got something else"),
            }
        },
        _ => panic!("Expected Offer, got something else"),
    }
}