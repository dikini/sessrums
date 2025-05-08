//! Concurrent protocol execution test for multiparty session types.
//!
//! This test demonstrates how to execute a multiparty protocol concurrently
//! with each participant running in its own thread. It uses the same protocol
//! from the manual projection example (Client -> Server -> Storage) and verifies
//! that the protocol executes correctly with all messages delivered to the
//! right participants.
//!
//! The test showcases:
//! 1. Setting up a shared broker for message routing
//! 2. Creating local protocols for each participant
//! 3. Converting local protocols to executable multiparty sessions
//! 4. Spawning threads for concurrent execution
//! 5. Error handling and resource cleanup
//! 6. Verification of correct message exchange

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use sessrums_examples::examples::manual_projection::{
    messages::{Request, Response, Log},
    project_to_client, project_to_server, project_to_storage,
    Storage,
};
use sessrums_types::roles::{Client, Server};
use sessrums_types::session_types::{
    Send, Receive, End,
    MultipartySession,
};
use sessrums_types::transport::{MockMultipartyBroker, RoleIdentifier, ParticipantChannel};
use sessrums_types::error::SessionError;

/// Converts a local protocol Send action to a MultipartySession Send state
fn to_multiparty_send<R, M, Next>(
    channel: ParticipantChannel<R>,
) -> MultipartySession<R, Send<M, Next>, ParticipantChannel<R>>
where
    R: sessrums_types::roles::Role,
{
    MultipartySession::new(channel)
}

/// Converts a local protocol Receive action to a MultipartySession Receive state
fn to_multiparty_receive<R, M, Next>(
    channel: ParticipantChannel<R>,
) -> MultipartySession<R, Receive<M, Next>, ParticipantChannel<R>>
where
    R: sessrums_types::roles::Role,
{
    MultipartySession::new(channel)
}

/// Converts a local protocol End action to a MultipartySession End state
fn to_multiparty_end<R>(
    channel: ParticipantChannel<R>,
) -> MultipartySession<R, End, ParticipantChannel<R>>
where
    R: sessrums_types::roles::Role,
{
    MultipartySession::new(channel)
}

/// Shared state to verify correct message content was exchanged
#[derive(Debug, Default)]
struct TestState {
    client_received_response: Option<Response>,
    storage_received_log: Option<Log>,
}

/// Test that demonstrates concurrent execution of a 3-party protocol
#[test]
fn test_concurrent_protocol_execution() {
    // Create a shared broker for message routing
    let broker = Arc::new(MockMultipartyBroker::new());
    
    // Define role identifiers
    let client_id = RoleIdentifier::new("client");
    let server_id = RoleIdentifier::new("server");
    let storage_id = RoleIdentifier::new("storage");
    
    // Register all participants with the broker
    broker.register_participant(&client_id).expect("Failed to register client");
    broker.register_participant(&server_id).expect("Failed to register server");
    broker.register_participant(&storage_id).expect("Failed to register storage");
    
    // Create a shared state to verify message content
    let test_state = Arc::new(Mutex::new(TestState::default()));
    
    // Create channels for each participant
    let client_channel = broker.create_channel::<Client>(&client_id).expect("Failed to create client channel");
    let server_channel = broker.create_channel::<Server>(&server_id).expect("Failed to create server channel");
    let storage_channel = broker.create_channel::<Storage>(&storage_id).expect("Failed to create storage channel");
    
    // Spawn thread for client
    let client_broker = Arc::clone(&broker);
    let client_state = Arc::clone(&test_state);
    let client_handle = thread::spawn(move || {
        // Execute client protocol
        let result = execute_client_protocol(client_channel, &client_state);
        
        // Return result for error handling in main thread
        result
    });
    
    // Spawn thread for server
    let server_broker = Arc::clone(&broker);
    let server_handle = thread::spawn(move || {
        // Execute server protocol
        let result = execute_server_protocol(server_channel);
        
        // Return result for error handling in main thread
        result
    });
    
    // Spawn thread for storage
    let storage_broker = Arc::clone(&broker);
    let storage_state = Arc::clone(&test_state);
    let storage_handle = thread::spawn(move || {
        // Execute storage protocol
        let result = execute_storage_protocol(storage_channel, &storage_state);
        
        // Return result for error handling in main thread
        result
    });
    
    // Join all threads and handle errors
    let client_result = client_handle.join().expect("Client thread panicked");
    let server_result = server_handle.join().expect("Server thread panicked");
    let storage_result = storage_handle.join().expect("Storage thread panicked");
    
    // Check for errors in thread execution
    client_result.expect("Client protocol execution failed");
    server_result.expect("Server protocol execution failed");
    storage_result.expect("Storage protocol execution failed");
    
    // Verify message content
    let state = test_state.lock().expect("Failed to lock test state");
    
    // Verify client received the expected response
    let response = state.client_received_response.as_ref().expect("Client did not receive a response");
    assert_eq!(response.content, "Processed: Hello from client");
    assert_eq!(response.status, 200);
    
    // Verify storage received the expected log
    let log = state.storage_received_log.as_ref().expect("Storage did not receive a log");
    assert_eq!(log.details, "Request processed successfully");
    assert!(log.request_id.len() > 0);
}

/// Execute the client protocol
fn execute_client_protocol(
    channel: ParticipantChannel<Client>,
    state: &Arc<Mutex<TestState>>,
) -> Result<(), SessionError> {
    // Create a session for the client
    let session = to_multiparty_send::<Client, Request, Receive<Response, End>>(channel);
    
    // Create the request message
    let request = Request {
        content: "Hello from client".to_string(),
    };
    
    // Send the request to the server
    let server_id = RoleIdentifier::new("server");
    let session = session.send(&server_id, request)?;
    
    // Receive the response from the server
    let (response, session) = session.receive::<Response>(&server_id)?;
    
    // Store the response in the shared state
    let mut test_state = state.lock()?;
    test_state.client_received_response = Some(response);
    
    // Close the session
    let _channel = session.close()?;
    
    Ok(())
}

/// Execute the server protocol
fn execute_server_protocol(
    channel: ParticipantChannel<Server>,
) -> Result<(), SessionError> {
    // Create a session for the server
    let session = to_multiparty_receive::<Server, Request, Send<Response, Send<Log, End>>>(channel);
    
    // Receive the request from the client
    let client_id = RoleIdentifier::new("client");
    let (request, session) = session.receive::<Request>(&client_id)?;
    
    // Process the request
    let response = Response {
        content: format!("Processed: {}", request.content),
        status: 200,
    };
    
    // Send the response to the client
    let session = session.send(&client_id, response)?;
    
    // Create a log message
    let log = Log {
        request_id: format!("req-{}", std::time::SystemTime::now().elapsed().unwrap().as_micros()),
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        details: "Request processed successfully".to_string(),
    };
    
    // Send the log to storage
    let storage_id = RoleIdentifier::new("storage");
    let session = session.send(&storage_id, log)?;
    
    // Close the session
    let _channel = session.close()?;
    
    Ok(())
}

/// Execute the storage protocol
fn execute_storage_protocol(
    channel: ParticipantChannel<Storage>,
    state: &Arc<Mutex<TestState>>,
) -> Result<(), SessionError> {
    // Create a session for the storage
    let session = to_multiparty_receive::<Storage, Log, End>(channel);
    
    // Receive the log from the server
    let server_id = RoleIdentifier::new("server");
    let (log, session) = session.receive::<Log>(&server_id)?;
    
    // Store the log in the shared state
    let mut test_state = state.lock()?;
    test_state.storage_received_log = Some(log);
    
    // Close the session
    let _channel = session.close()?;
    
    Ok(())
}

/// Test that demonstrates error handling in concurrent protocol execution
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
    
    // Execute client protocol which should fail when trying to send to non-existent server
    let session = to_multiparty_send::<Client, Request, Receive<Response, End>>(client_channel);
    
    // Create the request message
    let request = Request {
        content: "Hello from client".to_string(),
    };
    
    // Send the request to the server (should fail)
    let server_id = RoleIdentifier::new("server");
    let result = session.send(&server_id, request);
    
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

/// Test that demonstrates handling thread panics
#[test]
fn test_thread_panic_handling() {
    // Create a shared broker for message routing
    let broker = Arc::new(MockMultipartyBroker::new());
    
    // Define role identifiers
    let client_id = RoleIdentifier::new("client");
    let server_id = RoleIdentifier::new("server");
    
    // Register participants
    broker.register_participant(&client_id).expect("Failed to register client");
    broker.register_participant(&server_id).expect("Failed to register server");
    
    // Create channels
    let client_channel = broker.create_channel::<Client>(&client_id).expect("Failed to create client channel");
    
    // Spawn a thread that will panic
    let panic_handle = thread::spawn(move || {
        // Simulate some work
        thread::sleep(Duration::from_millis(10));
        
        // Deliberately panic
        panic!("Simulated panic in thread");
    });
    
    // Join the thread and expect a panic
    let join_result = panic_handle.join();
    assert!(join_result.is_err());
    
    // Verify we can recover from the panic and continue
    // Create a new channel to verify the broker is still usable
    let new_client_channel = broker.create_channel::<Client>(&client_id);
    assert!(new_client_channel.is_ok());
}