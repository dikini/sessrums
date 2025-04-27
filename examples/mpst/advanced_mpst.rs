//! Advanced example of multiparty session types (MPST) in sessrums.
//!
//! This example demonstrates a more complex protocol with branching and recursion:
//! 1. Client sends a command to Server
//! 2. Server makes a choice based on the command:
//!    a. If the command is "QUERY", Server sends data to Client and logs to Logger
//!    b. If the command is "EXIT", Server sends a goodbye message and ends
//! 3. The protocol loops back to step 1 unless the EXIT branch is taken
//!
//! The protocol is defined using the global protocol types and then projected
//! to local protocols for each role.

use sessrums::chan::Chan;
use sessrums::proto::{Protocol, Role, End};
use sessrums::proto::global::{GlobalProtocol, GSend, GRecv, GChoice, GOffer, GRec, GVar, GEnd};
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
type Command = String;
type Data = String;
type LogMessage = String;
type Goodbye = String;

// Define a label for recursion
struct LoopLabel;

// Define the global protocol with recursion and choice
// 1. Client sends a Command to Server
// 2. Server makes a choice:
//    a. QUERY branch: Server sends Data to Client and LogMessage to Logger, then loops
//    b. EXIT branch: Server sends Goodbye to Client and ends
type GlobalProtocol = GRec<LoopLabel, 
    GSend<Command, Client, Server,
        GChoice<Server, (
            // QUERY branch
            GSend<Data, Server, Client,
                GSend<LogMessage, Server, Logger,
                    GVar<LoopLabel>
                >
            >,
            // EXIT branch
            GSend<Goodbye, Server, Client,
                GEnd
            >
        )>
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
    println!("Starting advanced MPST example with recursion and choice");
    
    // Create channels for communication between roles
    let (client_server_cmd, server_client_cmd) = MpscChannel::<Command>::new();
    let (server_client_data, client_server_data) = MpscChannel::<Data>::new();
    let (server_client_bye, client_server_bye) = MpscChannel::<Goodbye>::new();
    let (server_logger, logger_server) = MpscChannel::<LogMessage>::new();
    let (server_choice, client_choice) = MpscChannel::<bool>::new();
    
    // Create channels for each role
    let client_chan = Chan::<ClientProtocol, Client, _>::new(client_server_cmd);
    let server_chan = Chan::<ServerProtocol, Server, _>::new(server_client_cmd);
    let logger_chan = Chan::<LoggerProtocol, Logger, _>::new(logger_server);
    
    // Spawn a thread for the client
    let client_handle = tokio::spawn(async move {
        // Enter the recursive protocol
        let mut client_chan = client_chan.enter();
        
        // First iteration: Send QUERY command
        println!("Client: Sending QUERY command");
        let client_chan = client_chan.send("QUERY".to_string()).await.unwrap();
        
        // Offer a choice from the server
        let client_chan = client_chan.offer(
            // QUERY branch handler
            |chan| async move {
                println!("Client: Server chose QUERY branch");
                let (data, chan) = chan.recv().await.unwrap();
                println!("Client: Received data: {}", data);
                
                // Return to the start of the recursion
                Ok(chan.zero())
            },
            // EXIT branch handler
            |chan| async move {
                println!("Client: Server chose EXIT branch");
                let (goodbye, chan) = chan.recv().await.unwrap();
                println!("Client: Received goodbye: {}", goodbye);
                
                // Close the channel
                chan.close().unwrap();
                println!("Client: Channel closed");
                
                Ok(chan)
            }
        ).await.unwrap();
        
        // Second iteration: Send EXIT command
        let client_chan = client_chan.send("EXIT".to_string()).await.unwrap();
        
        // Offer a choice from the server again
        client_chan.offer(
            // QUERY branch handler
            |chan| async move {
                println!("Client: Server chose QUERY branch");
                let (data, chan) = chan.recv().await.unwrap();
                println!("Client: Received data: {}", data);
                
                // Return to the start of the recursion
                Ok(chan.zero())
            },
            // EXIT branch handler
            |chan| async move {
                println!("Client: Server chose EXIT branch");
                let (goodbye, chan) = chan.recv().await.unwrap();
                println!("Client: Received goodbye: {}", goodbye);
                
                // Close the channel
                chan.close().unwrap();
                println!("Client: Channel closed");
                
                Ok(chan)
            }
        ).await.unwrap();
    });
    
    // Spawn a thread for the server
    let server_handle = tokio::spawn(async move {
        // Enter the recursive protocol
        let mut server_chan = server_chan.enter();
        
        // First iteration
        println!("Server: Waiting for command from client");
        let (command, server_chan) = server_chan.recv().await.unwrap();
        println!("Server: Received command: {}", command);
        
        if command == "QUERY" {
            // Choose the QUERY branch
            println!("Server: Choosing QUERY branch");
            let server_chan = server_chan.choose_left().await.unwrap();
            
            // Send data to client
            println!("Server: Sending data to client");
            let server_chan = server_chan.send("Some data for your query".to_string()).await.unwrap();
            
            // Send log message to logger
            println!("Server: Sending log message to logger");
            let server_chan = server_chan.send("Query processed".to_string()).await.unwrap();
            
            // Return to the start of the recursion
            server_chan = server_chan.zero();
            
            // Second iteration
            println!("Server: Waiting for next command from client");
            let (command, server_chan) = server_chan.recv().await.unwrap();
            println!("Server: Received command: {}", command);
            
            // Choose the EXIT branch this time
            println!("Server: Choosing EXIT branch");
            let server_chan = server_chan.choose_right().await.unwrap();
            
            // Send goodbye to client
            println!("Server: Sending goodbye to client");
            let server_chan = server_chan.send("Goodbye!".to_string()).await.unwrap();
            
            // Close the channel
            server_chan.close().unwrap();
            println!("Server: Channel closed");
        } else {
            // Choose the EXIT branch
            println!("Server: Choosing EXIT branch");
            let server_chan = server_chan.choose_right().await.unwrap();
            
            // Send goodbye to client
            println!("Server: Sending goodbye to client");
            let server_chan = server_chan.send("Goodbye!".to_string()).await.unwrap();
            
            // Close the channel
            server_chan.close().unwrap();
            println!("Server: Channel closed");
        }
    });
    
    // Spawn a thread for the logger
    let logger_handle = tokio::spawn(async move {
        // Enter the recursive protocol
        let mut logger_chan = logger_chan.enter();
        
        // Wait for log message from server
        println!("Logger: Waiting for log message from server");
        let (log_message, logger_chan) = logger_chan.recv().await.unwrap();
        println!("Logger: Received log message: {}", log_message);
        
        // Return to the start of the recursion
        logger_chan = logger_chan.zero();
        
        // The logger doesn't receive any more messages in this example
        // because the server chooses the EXIT branch in the second iteration
        
        // Close the channel
        logger_chan.close().unwrap();
        println!("Logger: Channel closed");
    });
    
    // Wait for all threads to complete
    client_handle.await?;
    server_handle.await?;
    logger_handle.await?;
    
    println!("Advanced MPST example completed successfully");
    
    Ok(())
}