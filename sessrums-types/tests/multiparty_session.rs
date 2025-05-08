//! Tests for the multiparty session runtime.
//!
//! This module contains tests for the multiparty session runtime, including:
//! - Basic send/receive between multiple roles
//! - Choice/branching with multiple participants
//! - Recursive protocols
//! - Error handling and recovery

use std::sync::Arc;

use sessrums_types::roles::{Client, Server};
use sessrums_types::session_types::multiparty_session::{
    MultipartySession, Send, Receive, Select, Offer, End, OfferResult
};
use sessrums_types::transport::{MockMultipartyBroker, RoleIdentifier};
use sessrums_types::error::SessionError;

/// Type alias for the Storage role
/// 
/// Since we can't implement the sealed Role trait from outside the crate,
/// we'll use the existing Server role as a type parameter for the Storage role.
type Storage = Server;

/// Message types for our protocol
mod messages {
    use serde::{Serialize, Deserialize};
    
    /// Request message sent from Client to Server
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Request {
        pub content: String,
    }

    /// Response message sent from Server to Client
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Response {
        pub content: String,
        pub status: u32,
    }

    /// Log message sent from Server to Storage
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Log {
        pub request_id: String,
        pub timestamp: u64,
        pub details: String,
    }
    
    /// Choice message for selecting options
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Choice {
        pub option: String,
    }
}

/// Test that demonstrates basic multiparty protocol execution
///
/// This test creates a 3-party protocol with Client, Server, and Storage roles:
/// 1. Client sends a Request to Server
/// 2. Server processes the request and sends a Response to Client
/// 3. Server also sends a Log to Storage
/// 4. All parties terminate
///
/// The test verifies that all messages are correctly delivered and processed.
#[test]
fn test_basic_protocol_execution() -> Result<(), SessionError> {
    // Create a broker for message routing
    let broker = Arc::new(MockMultipartyBroker::new());
    
    // Define role identifiers
    let client_id = RoleIdentifier::new("client");
    let server_id = RoleIdentifier::new("server");
    let storage_id = RoleIdentifier::new("storage");
    
    // Register all participants with the broker
    broker.register_participant(&client_id)?;
    broker.register_participant(&server_id)?;
    broker.register_participant(&storage_id)?;
    
    // Create channels for each participant
    let client_channel = broker.create_channel::<Client>(&client_id)?;
    let server_channel = broker.create_channel::<Server>(&server_id)?;
    let storage_channel = broker.create_channel::<Storage>(&storage_id)?;
    
    // Define protocol types for each role
    type ClientProtocol = Send<messages::Request, Receive<messages::Response, End>>;
    type ServerProtocol = Receive<messages::Request, Send<messages::Response, Send<messages::Log, End>>>;
    type StorageProtocol = Receive<messages::Log, End>;
    
    // Create sessions for each participant
    let client_session = MultipartySession::<Client, ClientProtocol, _>::new(client_channel);
    let server_session = MultipartySession::<Server, ServerProtocol, _>::new(server_channel);
    let storage_session = MultipartySession::<Storage, StorageProtocol, _>::new(storage_channel);
    
    // Step 1: Client sends request to Server
    let request = messages::Request {
        content: "Hello from client".to_string(),
    };
    let client_session = client_session.send(&server_id, request)?;
    
    // Step 2: Server receives request from Client
    let (received_request, server_session) = server_session.receive::<messages::Request>(&client_id)?;
    assert_eq!(received_request.content, "Hello from client");
    
    // Step 3: Server sends response to Client
    let response = messages::Response {
        content: format!("Processed: {}", received_request.content),
        status: 200,
    };
    let server_session = server_session.send(&client_id, response.clone())?;
    
    // Step 4: Client receives response from Server
    let (received_response, client_session) = client_session.receive::<messages::Response>(&server_id)?;
    assert_eq!(received_response.content, "Processed: Hello from client");
    assert_eq!(received_response.status, 200);
    
    // Step 5: Server sends log to Storage
    let log = messages::Log {
        request_id: "req-123456".to_string(),
        timestamp: 1620000000,
        details: "Request processed successfully".to_string(),
    };
    let server_session = server_session.send(&storage_id, log.clone())?;
    
    // Step 6: Storage receives log from Server
    let (received_log, storage_session) = storage_session.receive::<messages::Log>(&server_id)?;
    assert_eq!(received_log.details, "Request processed successfully");
    assert_eq!(received_log.request_id, "req-123456");
    
    // Close all sessions
    let _client_channel = client_session.close()?;
    let _server_channel = server_session.close()?;
    let _storage_channel = storage_session.close()?;
    
    Ok(())
}

/// Test that demonstrates choice/branching in multiparty protocols
///
/// This test creates a protocol with choice:
/// 1. Client selects between "login" and "register" options
/// 2. Server offers the corresponding branches
/// 3. Based on the choice, different message exchanges occur
#[test]
fn test_choice_protocol() -> Result<(), SessionError> {
    // Create a broker for message routing
    let broker = Arc::new(MockMultipartyBroker::new());
    
    // Define role identifiers
    let client_id = RoleIdentifier::new("client");
    let server_id = RoleIdentifier::new("server");
    
    // Register participants with the broker
    broker.register_participant(&client_id)?;
    broker.register_participant(&server_id)?;
    
    // Create channels for each participant
    let client_channel = broker.create_channel::<Client>(&client_id)?;
    let server_channel = broker.create_channel::<Server>(&server_id)?;
    
    // Define protocol types for login branch
    type LoginRequest = Send<messages::Request, Receive<messages::Response, End>>;
    type LoginResponse = Receive<messages::Request, Send<messages::Response, End>>;
    
    // Define protocol types for register branch
    type RegisterRequest = Send<messages::Choice, End>;
    type RegisterResponse = Receive<messages::Choice, End>;
    
    // Define protocol types with choice
    type ClientProtocol = Select<LoginRequest, RegisterRequest>;
    type ServerProtocol = Offer<LoginResponse, RegisterResponse>;
    
    // Create sessions for each participant
    let client_session = MultipartySession::<Client, ClientProtocol, _>::new(client_channel);
    let server_session = MultipartySession::<Server, ServerProtocol, _>::new(server_channel);
    
    // Client selects the login branch
    let client_session = client_session.select_left()?;
    
    // Server offers the choice and follows the client's selection
    let server_session = match server_session.offer()? {
        OfferResult::Left(session) => session,
        OfferResult::Right(_) => panic!("Server should have selected the left branch"),
    };
    
    // Client sends login request
    let request = messages::Request {
        content: "Login request".to_string(),
    };
    let client_session = client_session.send(&server_id, request)?;
    
    // Server receives login request
    let (received_request, server_session) = server_session.receive::<messages::Request>(&client_id)?;
    assert_eq!(received_request.content, "Login request");
    
    // Server sends login response
    let response = messages::Response {
        content: "Login successful".to_string(),
        status: 200,
    };
    let server_session = server_session.send(&client_id, response)?;
    
    // Client receives login response
    let (received_response, client_session) = client_session.receive::<messages::Response>(&server_id)?;
    assert_eq!(received_response.content, "Login successful");
    
    // Close all sessions
    let _client_channel = client_session.close()?;
    let _server_channel = server_session.close()?;
    
    Ok(())
}

/// Test that demonstrates error handling in protocol execution
///
/// This test deliberately creates an error condition by:
/// 1. Registering only the Client role with the broker
/// 2. Attempting to send a message to a non-existent Server role
/// 3. Verifying that the expected error is returned
#[test]
fn test_protocol_error_handling() {
    // Create a shared broker for message routing
    let broker = Arc::new(MockMultipartyBroker::new());
    
    // Define role identifiers
    let client_id = RoleIdentifier::new("client");
    let server_id = RoleIdentifier::new("server");
    
    // Only register client (deliberately omit server to cause an error)
    broker.register_participant(&client_id).expect("Failed to register client");
    
    // Create a channel for the client
    let client_channel = broker.create_channel::<Client>(&client_id).expect("Failed to create client channel");
    
    // Define a simple protocol for the client
    type ClientProtocol = Send<messages::Request, End>;
    
    // Create a session for the client
    let client_session = MultipartySession::<Client, ClientProtocol, _>::new(client_channel);
    
    // Create the request message
    let request = messages::Request {
        content: "Hello from client".to_string(),
    };
    
    // Send the request to the server (should fail)
    let result = client_session.send(&server_id, request);
    
    // Verify that the expected error occurred
    assert!(result.is_err());
    if let Err(err) = result {
        match err {
            SessionError::ProtocolViolation(msg) => {
                assert!(msg.contains("not found"));
            },
            _ => panic!("Expected ProtocolViolation error, got: {:?}", err),
        }
    }
}

/// Test that demonstrates a recursive protocol
///
/// This test creates a recursive protocol where:
/// 1. Client sends a series of messages to Server
/// 2. Server responds to each message
/// 3. The protocol repeats until a termination condition is met
#[test]
fn test_recursive_protocol() -> Result<(), SessionError> {
    // Create a broker for message routing
    let broker = Arc::new(MockMultipartyBroker::new());
    
    // Define role identifiers
    let client_id = RoleIdentifier::new("client");
    let server_id = RoleIdentifier::new("server");
    
    // Register participants with the broker
    broker.register_participant(&client_id)?;
    broker.register_participant(&server_id)?;
    
    // Create channels for each participant
    let client_channel = broker.create_channel::<Client>(&client_id)?;
    let server_channel = broker.create_channel::<Server>(&server_id)?;
    
    // Define recursive protocol types
    // For simplicity, we'll use a fixed number of iterations instead of true recursion
    type ClientProtocol = Send<messages::Request, Receive<messages::Response, 
                         Send<messages::Request, Receive<messages::Response, End>>>>;
    
    type ServerProtocol = Receive<messages::Request, Send<messages::Response, 
                         Receive<messages::Request, Send<messages::Response, End>>>>;
    
    // Create sessions for each participant
    let client_session = MultipartySession::<Client, ClientProtocol, _>::new(client_channel);
    let server_session = MultipartySession::<Server, ServerProtocol, _>::new(server_channel);
    
    // First iteration
    let request1 = messages::Request {
        content: "Request 1".to_string(),
    };
    let client_session = client_session.send(&server_id, request1)?;
    
    let (received_request1, server_session) = server_session.receive::<messages::Request>(&client_id)?;
    assert_eq!(received_request1.content, "Request 1");
    
    let response1 = messages::Response {
        content: "Response 1".to_string(),
        status: 200,
    };
    let server_session = server_session.send(&client_id, response1)?;
    
    let (received_response1, client_session) = client_session.receive::<messages::Response>(&server_id)?;
    assert_eq!(received_response1.content, "Response 1");
    
    // Second iteration
    let request2 = messages::Request {
        content: "Request 2".to_string(),
    };
    let client_session = client_session.send(&server_id, request2)?;
    
    let (received_request2, server_session) = server_session.receive::<messages::Request>(&client_id)?;
    assert_eq!(received_request2.content, "Request 2");
    
    let response2 = messages::Response {
        content: "Response 2".to_string(),
        status: 200,
    };
    let server_session = server_session.send(&client_id, response2)?;
    
    let (received_response2, client_session) = client_session.receive::<messages::Response>(&server_id)?;
    assert_eq!(received_response2.content, "Response 2");
    
    // Close all sessions
    let _client_channel = client_session.close()?;
    let _server_channel = server_session.close()?;
    
    Ok(())
}