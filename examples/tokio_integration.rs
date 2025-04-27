//! # Tokio Integration Example
//!
//! This example demonstrates how to integrate the session types library with the Tokio
//! asynchronous runtime. It shows how to:
//!
//! 1. Create a custom IO implementation using Tokio's channels
//! 2. Implement the AsyncSender and AsyncReceiver traits for this IO implementation
//! 3. Use the session types library with Tokio's async/await syntax
//! 4. Run a client-server protocol using Tokio tasks
//!
//! ## Protocol Diagram
//!
//! ```text
//! Client                                Server
//!   |                                     |
//!   |------- Send(String) - Request ----->|
//!   |                                     |
//!   |<------ Recv(String) - Response -----|
//!   |                                     |
//!   |------- Send(i32) - Value ---------->|
//!   |                                     |
//!   |<------ Recv(i32) - Result ----------|
//!   |                                     |
//!   |-------------- End ---------------->|
//! ```

use futures_core::future::Future;
use futures_core::task::{Context, Poll};
use sessrums::chan::Chan;
use sessrums::error::Error;
use sessrums::io::{AsyncReceiver, AsyncSender};
use sessrums::proto::{End, Protocol, Recv, Send};
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Define the protocol types for client and server
type ClientProtocol = Send<String, Recv<String, Send<i32, Recv<i32, End>>>>;
type ServerProtocol = <ClientProtocol as Protocol>::Dual;

/// A custom IO implementation using Tokio's mpsc channels
struct TokioChannel {
    string_sender: mpsc::Sender<String>,
    string_receiver: Arc<Mutex<mpsc::Receiver<String>>>,
    int_sender: mpsc::Sender<i32>,
    int_receiver: Arc<Mutex<mpsc::Receiver<i32>>>,
}

// Define futures for TokioChannel
struct StringSendFuture {
    sender: mpsc::Sender<String>,
    value: Option<String>,
}

struct IntSendFuture {
    sender: mpsc::Sender<i32>,
    value: Option<i32>,
}

struct StringRecvFuture {
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
}

struct IntRecvFuture {
    receiver: Arc<Mutex<mpsc::Receiver<i32>>>,
}

// Implement Future for StringSendFuture
impl Future for StringSendFuture {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        if let Some(value) = this.value.take() {
            match this.sender.try_send(value) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(mpsc::error::TrySendError::Full(value)) => {
                    // Put the value back and register the waker
                    this.value = Some(value);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    Poll::Ready(Err(Error::ChannelClosed))
                }
            }
        } else {
            Poll::Ready(Err(Error::Protocol("No value to send")))
        }
    }
}

// Implement Future for IntSendFuture
impl Future for IntSendFuture {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        if let Some(value) = this.value.take() {
            match this.sender.try_send(value) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(mpsc::error::TrySendError::Full(value)) => {
                    // Put the value back and register the waker
                    this.value = Some(value);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    Poll::Ready(Err(Error::ChannelClosed))
                }
            }
        } else {
            Poll::Ready(Err(Error::Protocol("No value to send")))
        }
    }
}

// Implement Future for StringRecvFuture
impl Future for StringRecvFuture {
    type Output = Result<String, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        // Try to lock the mutex
        match this.receiver.try_lock() {
            Ok(mut receiver) => {
                // Try to receive a value
                match receiver.try_recv() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(mpsc::error::TryRecvError::Empty) => {
                        // Register the waker and return Pending
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        Poll::Ready(Err(Error::ChannelClosed))
                    }
                }
            }
            Err(_) => {
                // Mutex is locked, register the waker and return Pending
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

// Implement Future for IntRecvFuture
impl Future for IntRecvFuture {
    type Output = Result<i32, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        // Try to lock the mutex
        match this.receiver.try_lock() {
            Ok(mut receiver) => {
                // Try to receive a value
                match receiver.try_recv() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(mpsc::error::TryRecvError::Empty) => {
                        // Register the waker and return Pending
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        Poll::Ready(Err(Error::ChannelClosed))
                    }
                }
            }
            Err(_) => {
                // Mutex is locked, register the waker and return Pending
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

// Implement AsyncSender<String> for TokioChannel
impl AsyncSender<String> for TokioChannel {
    type Error = Error;
    type SendFuture<'a> = StringSendFuture where Self: 'a;

    fn send(&mut self, value: String) -> Self::SendFuture<'_> {
        StringSendFuture {
            sender: self.string_sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver<String> for TokioChannel
impl AsyncReceiver<String> for TokioChannel {
    type Error = Error;
    type RecvFuture<'a> = StringRecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        StringRecvFuture {
            receiver: self.string_receiver.clone(),
        }
    }
}

// Implement AsyncSender<i32> for TokioChannel
impl AsyncSender<i32> for TokioChannel {
    type Error = Error;
    type SendFuture<'a> = IntSendFuture where Self: 'a;

    fn send(&mut self, value: i32) -> Self::SendFuture<'_> {
        IntSendFuture {
            sender: self.int_sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver<i32> for TokioChannel
impl AsyncReceiver<i32> for TokioChannel {
    type Error = Error;
    type RecvFuture<'a> = IntRecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        IntRecvFuture {
            receiver: self.int_receiver.clone(),
        }
    }
}

/// Creates a pair of TokioChannel instances for bidirectional communication
fn create_tokio_channel_pair() -> (TokioChannel, TokioChannel) {
    // Create channels for client -> server and server -> client communication
    let (client_string_tx, server_string_rx) = mpsc::channel(10);
    let (server_string_tx, client_string_rx) = mpsc::channel(10);
    
    let (client_int_tx, server_int_rx) = mpsc::channel(10);
    let (server_int_tx, client_int_rx) = mpsc::channel(10);
    
    let client_channel = TokioChannel {
        string_sender: client_string_tx,
        string_receiver: Arc::new(Mutex::new(client_string_rx)),
        int_sender: client_int_tx,
        int_receiver: Arc::new(Mutex::new(client_int_rx)),
    };
    
    let server_channel = TokioChannel {
        string_sender: server_string_tx,
        string_receiver: Arc::new(Mutex::new(server_string_rx)),
        int_sender: server_int_tx,
        int_receiver: Arc::new(Mutex::new(server_int_rx)),
    };
    
    (client_channel, server_channel)
}

/// Implements the client side of the protocol
async fn run_client(chan: Chan<ClientProtocol, TokioChannel>) -> Result<(), Error> {
    println!("Client: Starting protocol");
    
    // Step 1: Send a request to the server
    println!("Client: Sending request");
    let request = "calculate:square".to_string();
    let chan = chan.send(request).await?;
    println!("Client: Request sent successfully");
    
    // Step 2: Receive a response from the server
    println!("Client: Waiting for server response");
    let (response, chan) = chan.recv().await?;
    println!("Client: Received response: {}", response);
    
    // Step 3: Send a value to the server
    println!("Client: Sending value");
    let value = 7;
    let chan = chan.send(value).await?;
    println!("Client: Value sent successfully");
    
    // Step 4: Receive the result from the server
    println!("Client: Waiting for result");
    let (result, chan) = chan.recv().await?;
    println!("Client: Received result: {}", result);
    
    // Step 5: Close the channel
    chan.close()?;
    println!("Client: Protocol completed successfully");
    
    Ok(())
}

/// Implements the server side of the protocol
async fn run_server(chan: Chan<ServerProtocol, TokioChannel>) -> Result<(), Error> {
    println!("Server: Starting protocol");
    
    // Step 1: Receive a request from the client
    println!("Server: Waiting for request");
    let (request, chan) = chan.recv().await?;
    println!("Server: Received request: {}", request);
    
    // Step 2: Send a response to the client
    println!("Server: Sending response");
    let response = "ready".to_string();
    let chan = chan.send(response).await?;
    println!("Server: Response sent successfully");
    
    // Step 3: Receive a value from the client
    println!("Server: Waiting for value");
    let (value, chan) = chan.recv().await?;
    println!("Server: Received value: {}", value);
    
    // Step 4: Process the value and send the result
    println!("Server: Processing value");
    let result = value * value; // Square the value
    println!("Server: Sending result");
    let chan = chan.send(result).await?;
    println!("Server: Result sent successfully");
    
    // Step 5: Close the channel
    chan.close()?;
    println!("Server: Protocol completed successfully");
    
    Ok(())
}

/// Demonstrates how to use the Send trait with futures
async fn demonstrate_send_trait() -> Result<(), Error> {
    println!("\nDemonstrating Send trait implementation:");
    
    // Create a channel pair
    let (client_channel, server_channel) = create_tokio_channel_pair();
    
    // Create client and server channels with their respective protocols
    let client_chan = Chan::<ClientProtocol, _>::new(client_channel);
    let server_chan = Chan::<ServerProtocol, _>::new(server_channel);
    
    // Spawn the server in a separate task
    // This requires the futures to implement Send
    let server_handle = tokio::spawn(async move {
        run_server(server_chan).await
    });
    
    // Run the client in the main task
    run_client(client_chan).await?;
    
    // Wait for the server to complete
    server_handle.await.unwrap()?;
    
    println!("Send trait demonstration completed successfully");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("=== Tokio Integration Example ===\n");
    
    // Demonstrate the Send trait implementation
    demonstrate_send_trait().await?;
    
    println!("\n=== Example completed successfully ===");
    Ok(())
}