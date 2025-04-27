//! Basic example of multiparty session types (MPST) in sessrums.
//!
//! This example demonstrates a simple three-party protocol where:
//! 1. Client sends a request to Server
//! 2. Server processes the request and sends a response to Client
//! 3. Server also sends a log message to Logger
//!
//! The protocol is defined using the global protocol types and then projected
//! to local protocols for each role.

use sessrums::chan::Chan;
use sessrums::proto::{Protocol, Role, End};
use sessrums::proto::global::{GlobalProtocol, GSend, GEnd};
use sessrums::proto::projection::Project;
use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread;

// Define the roles in our protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Server;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Logger;

// Implement the Role trait for each role
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

// Define the message types
type Request = String;
type Response = String;
type LogMessage = String;

// Define the global protocol
// Client sends a Request to Server
// Server sends a Response to Client
// Server sends a LogMessage to Logger
type GlobalProtocol = GSend<Request, Client, Server, 
    GSend<Response, Server, Client, 
        GSend<LogMessage, Server, Logger, GEnd>
    >
>;

// Project the global protocol to local protocols for each role
type ClientProtocol = <GlobalProtocol as Project<Client>>::LocalProtocol;
type ServerProtocol = <GlobalProtocol as Project<Server>>::LocalProtocol;
type LoggerProtocol = <GlobalProtocol as Project<Logger>>::LocalProtocol;

// Create a simple channel implementation using mpsc
struct MpscChannel<T> {
    sender: mpsc::Sender<T>,
    receiver: Option<mpsc::Receiver<T>>,
}

impl<T> MpscChannel<T> {
    fn new() -> (Self, Self) {
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        
        let endpoint1 = MpscChannel {
            sender: tx1,
            receiver: Some(rx2),
        };
        
        let endpoint2 = MpscChannel {
            sender: tx2,
            receiver: Some(rx1),
        };
        
        (endpoint1, endpoint2)
    }
}

// Implement AsyncSender for MpscChannel
impl<T: Clone + Send + 'static> sessrums::io::AsyncSender<T> for MpscChannel<T> {
    type Error = std::io::Error;
    type SendFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'a>> where T: 'a, Self: 'a;

    fn send(&mut self, value: T) -> Self::SendFuture<'_> {
        let sender = self.sender.clone();
        let value = value.clone();
        
        Box::pin(async move {
            sender.send(value).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Channel closed")
            })
        })
    }
}

// Implement AsyncReceiver for MpscChannel
impl<T: Clone + Send + 'static> sessrums::io::AsyncReceiver<T> for MpscChannel<T> {
    type Error = std::io::Error;
    type RecvFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, Self::Error>> + Send + 'a>> where T: 'a, Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        let receiver = self.receiver.take();
        
        Box::pin(async move {
            if let Some(rx) = receiver {
                rx.recv().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Channel closed")
                })
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Receiver already consumed"))
            }
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting MPST example with three roles: Client, Server, and Logger");
    
    // Create channels for communication between roles
    let (client_server_1, server_client_1) = MpscChannel::<Request>::new();
    let (server_client_2, client_server_2) = MpscChannel::<Response>::new();
    let (server_logger, logger_server) = MpscChannel::<LogMessage>::new();
    
    // Create channels for each role
    let client_chan = Chan::<ClientProtocol, Client, _>::new(client_server_1);
    let server_chan = Chan::<ServerProtocol, Server, _>::new(server_client_1);
    let logger_chan = Chan::<LoggerProtocol, Logger, _>::new(logger_server);
    
    // Spawn a thread for the client
    let client_handle = tokio::spawn(async move {
        println!("Client: Sending request to server");
        let client_chan = client_chan.send("Hello, Server!".to_string()).await.unwrap();
        
        println!("Client: Waiting for response from server");
        let (response, client_chan) = client_chan.recv().await.unwrap();
        println!("Client: Received response: {}", response);
        
        // Close the channel
        client_chan.close().unwrap();
        println!("Client: Channel closed");
    });
    
    // Spawn a thread for the server
    let server_handle = tokio::spawn(async move {
        println!("Server: Waiting for request from client");
        let (request, server_chan) = server_chan.recv().await.unwrap();
        println!("Server: Received request: {}", request);
        
        // Process the request
        let response = format!("Response to: {}", request);
        println!("Server: Sending response to client");
        let server_chan = server_chan.send(response).await.unwrap();
        
        // Send log message to logger
        let log_message = format!("Processed request: {}", request);
        println!("Server: Sending log message to logger");
        let server_chan = server_chan.send(log_message).await.unwrap();
        
        // Close the channel
        server_chan.close().unwrap();
        println!("Server: Channel closed");
    });
    
    // Spawn a thread for the logger
    let logger_handle = tokio::spawn(async move {
        println!("Logger: Waiting for log message from server");
        let (log_message, logger_chan) = logger_chan.recv().await.unwrap();
        println!("Logger: Received log message: {}", log_message);
        
        // Close the channel
        logger_chan.close().unwrap();
        println!("Logger: Channel closed");
    });
    
    // Wait for all threads to complete
    client_handle.await?;
    server_handle.await?;
    logger_handle.await?;
    
    println!("MPST example completed successfully");
    
    Ok(())
}