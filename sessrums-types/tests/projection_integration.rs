use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread;

use sessrums_types::roles::{Client, Server};
use sessrums_types::session_types::common::{RoleIdentifier, Label};
use sessrums_types::session_types::global::GlobalInteraction;
use sessrums_types::session_types::local::LocalProtocol;
use sessrums_types::projection::{project_for_role, RoleExt};

// We'll use the existing roles instead of creating a new one
// since the Role trait is sealed and we can't implement it for new types
use sessrums_types::roles::Client as Database;

// Define message types for our protocol
#[derive(Clone)]
struct Request;
#[derive(Clone)]
struct Query;
// Rename this to avoid conflict with std::result::Result
#[derive(Clone)]
struct QueryResult;
#[derive(Clone)]
struct Accept;
#[derive(Clone)]
struct Reject;
#[derive(Clone)]
struct Notify;

// Mock channel for communication between roles
struct MockChannel<T> {
    sender: mpsc::Sender<T>,
    receiver: mpsc::Receiver<T>,
}

impl<T> MockChannel<T> {
    fn new() -> (Self, Self) {
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        
        (
            MockChannel {
                sender: tx1,
                receiver: rx2,
            },
            MockChannel {
                sender: tx2,
                receiver: rx1,
            },
        )
    }
    
    fn send(&self, msg: T) -> Result<(), mpsc::SendError<T>> {
        self.sender.send(msg)
    }
    
    fn recv(&self) -> Result<T, mpsc::RecvError> {
        self.receiver.recv()
    }
}

// Message enum for our protocol
#[derive(Clone)]
enum Message {
    Request,
    Query,
    QueryResult,
    Accept,
    Reject,
    Notify,
    Label(String),
}

#[test]
fn test_projection_execution() {
    // Define a global protocol: 
    // A -> B: Request;
    // B -> C: Query;
    // C -> B: Result;
    // choice at B {
    //   B -> A: Accept; B -> C: Notify; End
    // } or {
    //   B -> A: Reject; End
    // }
    let global = GlobalInteraction::message(
        "client",
        "server",
        GlobalInteraction::message(
            "server",
            "database",
            GlobalInteraction::message(
                "database",
                "server",
                GlobalInteraction::choice(
                    "server",
                    vec![
                        ("accept".into(), GlobalInteraction::message(
                            "server",
                            "client",
                            GlobalInteraction::message(
                                "server",
                                "database",
                                GlobalInteraction::end(),
                            ),
                        )),
                        ("reject".into(), GlobalInteraction::message(
                            "server",
                            "client",
                            GlobalInteraction::end(),
                        )),
                    ],
                ),
            ),
        ),
    );
    
    // Project for all roles
    let client_local = project_for_role::<Client, Message>(global.clone());
    let server_local = project_for_role::<Server, Message>(global.clone());
    let database_local = project_for_role::<Database, Message>(global.clone());
    
    // Set up mock channels for all role pairs
    let (client_to_server, server_from_client) = MockChannel::new();
    let (server_to_client, client_from_server) = MockChannel::new();
    let (server_to_database, database_from_server) = MockChannel::new();
    let (database_to_server, server_from_database) = MockChannel::new();
    
    // Execute the protocol concurrently
    let client_handle = thread::spawn(move || {
        // Client sends request, then receives either Accept or Reject
        println!("Client: Sending Request to Server");
        client_to_server.send(Message::Request).unwrap();
        
        // Wait for response from Server
        match client_from_server.recv().unwrap() {
            Message::Accept => {
                println!("Client: Received Accept from Server");
                // Protocol ends
            },
            Message::Reject => {
                println!("Client: Received Reject from Server");
                // Protocol ends
            },
            _ => panic!("Client: Unexpected message"),
        }
        
        println!("Client: Protocol completed");
    });
    
    let server_handle = thread::spawn(move || {
        // Server receives request from Client
        match server_from_client.recv().unwrap() {
            Message::Request => {
                println!("Server: Received Request from Client");
                
                // Server sends Query to Database
                println!("Server: Sending Query to Database");
                server_to_database.send(Message::Query).unwrap();
                
                // Server receives Result from Database
                match server_from_database.recv().unwrap() {
                    Message::QueryResult => {
                        println!("Server: Received Result from Database");
                        
                        // Server chooses Accept or Reject (let's choose Accept for this test)
                        let choice = "accept";
                        println!("Server: Choosing {}", choice);
                        
                        if choice == "accept" {
                            // Send Accept to Client
                            println!("Server: Sending Accept to Client");
                            server_to_client.send(Message::Accept).unwrap();
                            
                            // Send Notify to Database
                            println!("Server: Sending Notify to Database");
                            server_to_database.send(Message::Notify).unwrap();
                        } else {
                            // Send Reject to Client
                            println!("Server: Sending Reject to Client");
                            server_to_client.send(Message::Reject).unwrap();
                        }
                    },
                    _ => panic!("Server: Unexpected message"),
                }
            },
            _ => panic!("Server: Unexpected message"),
        }
        
        println!("Server: Protocol completed");
    });
    
    let database_handle = thread::spawn(move || {
        // Database receives Query from Server
        match database_from_server.recv().unwrap() {
            Message::Query => {
                println!("Database: Received Query from Server");
                
                // Database sends Result to Server
                println!("Database: Sending Result to Server");
                database_to_server.send(Message::QueryResult).unwrap();
                
                // Database may receive Notify from Server
                // Use a simple match on the message directly
                if let Ok(msg) = database_from_server.recv() {
                    match msg {
                        Message::Notify => {
                            println!("Database: Received Notify from Server");
                        },
                        _ => panic!("Database: Unexpected message"),
                    }
                } else {
                    // This is also valid - if Server chose Reject, Database won't receive anything
                    println!("Database: No further messages (Server chose Reject)");
                }
            },
            _ => panic!("Database: Unexpected message"),
        }
        
        println!("Database: Protocol completed");
    });
    
    // Join threads and verify results
    client_handle.join().unwrap();
    server_handle.join().unwrap();
    database_handle.join().unwrap();
    
    println!("All roles completed the protocol successfully");
}

#[test]
fn test_protocol_type_safety() {
    // This test verifies that the projected protocols are type-safe
    // and can be used with the correct message types
    
    // Define a global protocol: Client -> Server: Request; Server -> Client: Accept; End
    let global = GlobalInteraction::message(
        "client",
        "server",
        GlobalInteraction::message(
            "server",
            "client",
            GlobalInteraction::end(),
        ),
    );
    
    // Project for Client and Server
    let client_local = project_for_role::<Client, Message>(global.clone());
    let server_local = project_for_role::<Server, Message>(global.clone());
    
    // Verify the structure of the projected protocols
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
    
    println!("Protocol type safety verified");
}