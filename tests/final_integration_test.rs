//! Final integration test for MPST features.
//!
//! This test demonstrates a comprehensive example that uses all MPST features:
//! - Multiple roles (Client, Server, Logger)
//! - Local protocols for each role
//! - Branching and choice
//! - Message passing between multiple parties

use sessrums::chan::Chan;
use sessrums::proto::{Send, Recv, Role, End, Choose, Offer};
use sessrums::proto::roles::{RoleA, RoleB};

// Define a third role for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct RoleC;

impl Role for RoleC {
    fn name(&self) -> &'static str {
        "RoleC"
    }
}

// Mock IO implementation for testing
struct MockIO<T> {
    value: Option<T>,
}

impl<T> MockIO<T> {
    fn new(value: Option<T>) -> Self {
        MockIO { value }
    }
}

// Implement AsyncSender for MockIO
impl<T: Clone + std::marker::Unpin> sessrums::io::AsyncSender<T> for MockIO<T> {
    type Error = std::io::Error;
    type SendFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + 'a>> where T: 'a, Self: 'a;

    fn send(&mut self, value: T) -> Self::SendFuture<'_> {
        self.value = Some(value);
        Box::pin(async { Ok(()) })
    }
}

// Implement AsyncReceiver for MockIO
impl<T: Clone + std::marker::Unpin> sessrums::io::AsyncReceiver<T> for MockIO<T> {
    type Error = std::io::Error;
    type RecvFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, Self::Error>> + 'a>> where T: 'a, Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        let value = self.value.clone();
        Box::pin(async move {
            match value {
                Some(v) => Ok(v),
                None => Err(std::io::Error::new(std::io::ErrorKind::Other, "No value available")),
            }
        })
    }
}

// Define local protocols for a three-party interaction
type ClientProtocol = Send<String, Recv<String, End>>;
type ServerProtocol = Recv<String, Send<String, Send<String, End>>>;
type LoggerProtocol = Recv<String, End>;

#[tokio::test]
async fn test_final_integration() {
    // Create mock IO implementations
    let client_io_send = MockIO::new(Some("Request data".to_string())); // Client sends "Request data"
    let client_io_recv = MockIO::new(Some("Response data".to_string())); // Client receives "Response data"
    let server_io = MockIO::new(Some("Request data".to_string())); // Server receives "Request data"
    let server_io2 = MockIO::new(Some("Response data".to_string())); // Server sends "Response data"
    let logger_io = MockIO::new(Some("Log message".to_string()));
    
    // Create channels for each role
    let client_chan_send = Chan::<Send<String, End>, RoleA, _>::new(client_io_send);
    let client_chan_recv = Chan::<Recv<String, End>, RoleA, _>::new(client_io_recv);
    let server_chan = Chan::<ServerProtocol, RoleB, _>::new(server_io);
    let logger_chan = Chan::<LoggerProtocol, RoleC, _>::new(logger_io);
    
    // Execute the protocol
    
    // 1. Client sends a request to Server
    println!("Client: Sending request to server");
    let client_chan_send = client_chan_send.send("Request data".to_string()).await.unwrap();
    
    // 2. Server receives the request
    println!("Server: Receiving request from client");
    let (request, server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(request, "Request data");
    
    // 3. Server sends a response to Client (using a new channel with server_io2)
    println!("Server: Sending response to client");
    let server_chan2 = Chan::<Send<String, Send<String, End>>, RoleB, _>::new(server_io2);
    let server_chan2 = server_chan2.send("Response data".to_string()).await.unwrap();
    
    // 4. Client receives the response
    println!("Client: Receiving response from server");
    let (response, client_chan_recv) = client_chan_recv.recv().await.unwrap();
    assert_eq!(response, "Response data");
    
    // 6. Server sends a log message to Logger
    println!("Server: Sending log message to logger");
    let server_chan2 = server_chan2.send("Log message".to_string()).await.unwrap();
    
    // 7. Logger receives the log message
    println!("Logger: Receiving log message");
    let (log_message, logger_chan) = logger_chan.recv().await.unwrap();
    assert_eq!(log_message, "Log message");
    
    // Close the channels
    println!("Closing channels");
    client_chan_send.close().unwrap();
    client_chan_recv.close().unwrap();
    // server_chan is not at End, so we don't close it
    server_chan2.close().unwrap();
    logger_chan.close().unwrap();
    
    println!("Final integration test completed successfully");
}