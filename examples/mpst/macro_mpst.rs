//! Example of using the global protocol macro for multiparty session types (MPST) in sessrums.
//!
//! This example demonstrates how to use the `global_protocol!` macro to define
//! a global protocol in a more intuitive, sequence diagram-like syntax.
//! The protocol involves three roles: Client, Server, and Logger.

use sessrums::chan::Chan;
use sessrums::proto::{Protocol, Role, End};
use sessrums::proto::global::{GlobalProtocol, GSend, GRecv, GChoice, GOffer, GRec, GVar, GEnd};
use sessrums::proto::projection::Project;
use sessrums::proto::global_protocol;
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

// Define the global protocol using the macro
global_protocol! {
    protocol OnlineStore {
        // Client sends a request to Server
        Client -> Server: Request;
        
        // Server makes a choice
        choice at Server {
            // Success branch
            option Success {
                // Server sends a response to Client
                Server -> Client: Response;
                // Server logs the successful transaction
                Server -> Logger: LogMessage;
            }
            // Error branch
            option Error {
                // Server sends an error response to Client
                Server -> Client: Response;
                // Server logs the error
                Server -> Logger: LogMessage;
            }
        }
    }
}

// Project the global protocol to local protocols for each role
type ClientProtocol = <OnlineStore as Project<Client>>::LocalProtocol;
type ServerProtocol = <OnlineStore as Project<Server>>::LocalProtocol;
type LoggerProtocol = <OnlineStore as Project<Logger>>::LocalProtocol;

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

// Implement AsyncSender for bool (for choice/offer)
impl sessrums::io::AsyncSender<bool> for MpscChannel<bool> {
    type Error = std::io::Error;
    type SendFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'a>> where bool: 'a, Self: 'a;

    fn send(&mut self, value: bool) -> Self::SendFuture<'_> {
        let sender = self.sender.clone();
        
        Box::pin(async move {
            sender.send(value).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Channel closed")
            })
        })
    }
}

// Implement AsyncReceiver for bool (for choice/offer)
impl sessrums::io::AsyncReceiver<bool> for MpscChannel<bool> {
    type Error = std::io::Error;
    type RecvFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, Self::Error>> + Send + 'a>> where bool: 'a, Self: 'a;

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
    println!("Starting MPST example using the global protocol macro");
    
    // Create channels for communication between roles
    let (client_server, server_client) = MpscChannel::<Request>::new();
    let (server_client_resp, client_server_resp) = MpscChannel::<Response>::new();
    let (server_logger, logger_server) = MpscChannel::<LogMessage>::new();
    let (server_choice, client_choice) = MpscChannel::<bool>::new();
    
    // Create channels for each role
    let client_chan = Chan::<ClientProtocol, Client, _>::new(client_server);
    let server_chan = Chan::<ServerProtocol, Server, _>::new(server_client);
    let logger_chan = Chan::<LoggerProtocol, Logger, _>::new(logger_server);
    
    // Spawn a thread for the client
    let client_handle = tokio::spawn(async move {
        println!("Client: Sending request to server");
        let client_chan = client_chan.send("Buy product".to_string()).await.unwrap();
        
        // Offer a choice from the server
        let client_chan = client_chan.offer(
            // Success branch handler
            |chan| async move {
                println!("Client: Server chose Success branch");
                let (response, chan) = chan.recv().await.unwrap();
                println!("Client: Received success response: {}", response);
                
                // Close the channel
                chan.close().unwrap();
                println!("Client: Channel closed");
                
                Ok(chan)
            },
            // Error branch handler
            |chan| async move {
                println!("Client: Server chose Error branch");
                let (response, chan) = chan.recv().await.unwrap();
                println!("Client: Received error response: {}", response);
                
                // Close the channel
                chan.close().unwrap();
                println!("Client: Channel closed");
                
                Ok(chan)
            }
        ).await.unwrap();
    });
    
    // Spawn a thread for the server
    let server_handle = tokio::spawn(async move {
        println!("Server: Waiting for request from client");
        let (request, server_chan) = server_chan.recv().await.unwrap();
        println!("Server: Received request: {}", request);
        
        // Simulate processing the request
        let success = request.contains("product");
        
        if success {
            // Choose the Success branch
            println!("Server: Choosing Success branch");
            let server_chan = server_chan.choose_left().await.unwrap();
            
            // Send success response to client
            println!("Server: Sending success response to client");
            let server_chan = server_chan.send("Order confirmed".to_string()).await.unwrap();
            
            // Send log message to logger
            println!("Server: Sending success log to logger");
            let server_chan = server_chan.send("Transaction successful".to_string()).await.unwrap();
            
            // Close the channel
            server_chan.close().unwrap();
            println!("Server: Channel closed");
        } else {
            // Choose the Error branch
            println!("Server: Choosing Error branch");
            let server_chan = server_chan.choose_right().await.unwrap();
            
            // Send error response to client
            println!("Server: Sending error response to client");
            let server_chan = server_chan.send("Order failed".to_string()).await.unwrap();
            
            // Send log message to logger
            println!("Server: Sending error log to logger");
            let server_chan = server_chan.send("Transaction failed".to_string()).await.unwrap();
            
            // Close the channel
            server_chan.close().unwrap();
            println!("Server: Channel closed");
        }
    });
    
    // Spawn a thread for the logger
    let logger_handle = tokio::spawn(async move {
        // Offer a choice from the server
        let logger_chan = logger_chan.offer(
            // Success branch handler
            |chan| async move {
                println!("Logger: Server chose Success branch");
                let (log_message, chan) = chan.recv().await.unwrap();
                println!("Logger: Received success log: {}", log_message);
                
                // Close the channel
                chan.close().unwrap();
                println!("Logger: Channel closed");
                
                Ok(chan)
            },
            // Error branch handler
            |chan| async move {
                println!("Logger: Server chose Error branch");
                let (log_message, chan) = chan.recv().await.unwrap();
                println!("Logger: Received error log: {}", log_message);
                
                // Close the channel
                chan.close().unwrap();
                println!("Logger: Channel closed");
                
                Ok(chan)
            }
        ).await.unwrap();
    });
    
    // Wait for all threads to complete
    client_handle.await?;
    server_handle.await?;
    logger_handle.await?;
    
    println!("MPST example using the global protocol macro completed successfully");
    
    Ok(())
}