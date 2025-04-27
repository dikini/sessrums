//! # async-std Integration Example
//!
//! This example demonstrates how to integrate the session types library with the async-std
//! asynchronous runtime. It shows how to:
//!
//! 1. Create a custom IO implementation using async-std's channels
//! 2. Implement the AsyncSender and AsyncReceiver traits for this IO implementation
//! 3. Use the session types library with async-std's async/await syntax
//! 4. Run a client-server protocol using async-std tasks
//!
//! ## Protocol Diagram
//!
//! ```text
//! Client                                Server
//!   |                                     |
//!   |------- Send(String) - Command ----->|
//!   |                                     |
//!   |<------ Recv(bool) - Status ---------|
//!   |                                     |
//!   |------- Send(Vec<i32>) - Data ------>|
//!   |                                     |
//!   |<------ Recv(Vec<i32>) - Result -----|
//!   |                                     |
//!   |-------------- End ---------------->|
//! ```

use futures_core::future::Future;
use futures_core::task::{Context, Poll};
use sessrums::chan::Chan;
use sessrums::error::Error;
use sessrums::io::{AsyncReceiver, AsyncSender};
use sessrums::proto::{End, Protocol, Recv, Send};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use async_std::channel;
use async_std::task;

// Define the protocol types for client and server
type ClientProtocol = Send<String, Recv<bool, Send<Vec<i32>, Recv<Vec<i32>, End>>>>;
type ServerProtocol = <ClientProtocol as Protocol>::Dual;

/// A custom IO implementation using async-std's channels
struct AsyncStdChannel {
    string_sender: channel::Sender<String>,
    string_receiver: Arc<Mutex<channel::Receiver<String>>>,
    bool_sender: channel::Sender<bool>,
    bool_receiver: Arc<Mutex<channel::Receiver<bool>>>,
    vec_sender: channel::Sender<Vec<i32>>,
    vec_receiver: Arc<Mutex<channel::Receiver<Vec<i32>>>>,
}

// Define futures for AsyncStdChannel
struct StringSendFuture {
    sender: channel::Sender<String>,
    value: Option<String>,
}

struct BoolSendFuture {
    sender: channel::Sender<bool>,
    value: Option<bool>,
}

struct VecSendFuture {
    sender: channel::Sender<Vec<i32>>,
    value: Option<Vec<i32>>,
}

struct StringRecvFuture {
    receiver: Arc<Mutex<channel::Receiver<String>>>,
}

struct BoolRecvFuture {
    receiver: Arc<Mutex<channel::Receiver<bool>>>,
}

struct VecRecvFuture {
    receiver: Arc<Mutex<channel::Receiver<Vec<i32>>>>,
}

// Implement Future for StringSendFuture
impl Future for StringSendFuture {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        if let Some(value) = this.value.take() {
            match this.sender.try_send(value) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(channel::TrySendError::Full(value)) => {
                    // Put the value back and register the waker
                    this.value = Some(value);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(channel::TrySendError::Closed(_)) => {
                    Poll::Ready(Err(Error::ChannelClosed))
                }
            }
        } else {
            Poll::Ready(Err(Error::Protocol("No value to send")))
        }
    }
}

// Implement Future for BoolSendFuture
impl Future for BoolSendFuture {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        if let Some(value) = this.value.take() {
            match this.sender.try_send(value) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(channel::TrySendError::Full(value)) => {
                    // Put the value back and register the waker
                    this.value = Some(value);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(channel::TrySendError::Closed(_)) => {
                    Poll::Ready(Err(Error::ChannelClosed))
                }
            }
        } else {
            Poll::Ready(Err(Error::Protocol("No value to send")))
        }
    }
}

// Implement Future for VecSendFuture
impl Future for VecSendFuture {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        if let Some(value) = this.value.take() {
            match this.sender.try_send(value) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(channel::TrySendError::Full(value)) => {
                    // Put the value back and register the waker
                    this.value = Some(value);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(channel::TrySendError::Closed(_)) => {
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
            Ok(receiver) => {
                // Try to receive a value
                match receiver.try_recv() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(channel::TryRecvError::Empty) => {
                        // Register the waker and return Pending
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(channel::TryRecvError::Closed) => {
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

// Implement Future for BoolRecvFuture
impl Future for BoolRecvFuture {
    type Output = Result<bool, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        // Try to lock the mutex
        match this.receiver.try_lock() {
            Ok(receiver) => {
                // Try to receive a value
                match receiver.try_recv() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(channel::TryRecvError::Empty) => {
                        // Register the waker and return Pending
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(channel::TryRecvError::Closed) => {
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

// Implement Future for VecRecvFuture
impl Future for VecRecvFuture {
    type Output = Result<Vec<i32>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        // Try to lock the mutex
        match this.receiver.try_lock() {
            Ok(receiver) => {
                // Try to receive a value
                match receiver.try_recv() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(channel::TryRecvError::Empty) => {
                        // Register the waker and return Pending
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(channel::TryRecvError::Closed) => {
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

// Implement AsyncSender<String> for AsyncStdChannel
impl AsyncSender<String> for AsyncStdChannel {
    type Error = Error;
    type SendFuture<'a> = StringSendFuture where Self: 'a;

    fn send(&mut self, value: String) -> Self::SendFuture<'_> {
        StringSendFuture {
            sender: self.string_sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver<String> for AsyncStdChannel
impl AsyncReceiver<String> for AsyncStdChannel {
    type Error = Error;
    type RecvFuture<'a> = StringRecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        StringRecvFuture {
            receiver: self.string_receiver.clone(),
        }
    }
}

// Implement AsyncSender<bool> for AsyncStdChannel
impl AsyncSender<bool> for AsyncStdChannel {
    type Error = Error;
    type SendFuture<'a> = BoolSendFuture where Self: 'a;

    fn send(&mut self, value: bool) -> Self::SendFuture<'_> {
        BoolSendFuture {
            sender: self.bool_sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver<bool> for AsyncStdChannel
impl AsyncReceiver<bool> for AsyncStdChannel {
    type Error = Error;
    type RecvFuture<'a> = BoolRecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        BoolRecvFuture {
            receiver: self.bool_receiver.clone(),
        }
    }
}

// Implement AsyncSender<Vec<i32>> for AsyncStdChannel
impl AsyncSender<Vec<i32>> for AsyncStdChannel {
    type Error = Error;
    type SendFuture<'a> = VecSendFuture where Self: 'a;

    fn send(&mut self, value: Vec<i32>) -> Self::SendFuture<'_> {
        VecSendFuture {
            sender: self.vec_sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver<Vec<i32>> for AsyncStdChannel
impl AsyncReceiver<Vec<i32>> for AsyncStdChannel {
    type Error = Error;
    type RecvFuture<'a> = VecRecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        VecRecvFuture {
            receiver: self.vec_receiver.clone(),
        }
    }
}

/// Creates a pair of AsyncStdChannel instances for bidirectional communication
fn create_async_std_channel_pair() -> (AsyncStdChannel, AsyncStdChannel) {
    // Create channels for client -> server and server -> client communication
    let (client_string_tx, server_string_rx) = channel::bounded(10);
    let (server_string_tx, client_string_rx) = channel::bounded(10);
    
    let (client_bool_tx, server_bool_rx) = channel::bounded(10);
    let (server_bool_tx, client_bool_rx) = channel::bounded(10);
    
    let (client_vec_tx, server_vec_rx) = channel::bounded(10);
    let (server_vec_tx, client_vec_rx) = channel::bounded(10);
    
    let client_channel = AsyncStdChannel {
        string_sender: client_string_tx,
        string_receiver: Arc::new(Mutex::new(client_string_rx)),
        bool_sender: client_bool_tx,
        bool_receiver: Arc::new(Mutex::new(client_bool_rx)),
        vec_sender: client_vec_tx,
        vec_receiver: Arc::new(Mutex::new(client_vec_rx)),
    };
    
    let server_channel = AsyncStdChannel {
        string_sender: server_string_tx,
        string_receiver: Arc::new(Mutex::new(server_string_rx)),
        bool_sender: server_bool_tx,
        bool_receiver: Arc::new(Mutex::new(server_bool_rx)),
        vec_sender: server_vec_tx,
        vec_receiver: Arc::new(Mutex::new(server_vec_rx)),
    };
    
    (client_channel, server_channel)
}

/// Implements the client side of the protocol
async fn run_client(chan: Chan<ClientProtocol, AsyncStdChannel>) -> Result<(), Error> {
    println!("Client: Starting protocol");
    
    // Step 1: Send a command to the server
    println!("Client: Sending command");
    let command = "process".to_string();
    let chan = chan.send(command).await?;
    println!("Client: Command sent successfully");
    
    // Step 2: Receive status from the server
    println!("Client: Waiting for status");
    let (status, chan) = chan.recv().await?;
    println!("Client: Received status: {}", status);
    
    if !status {
        println!("Client: Server rejected the command");
        // We can't close the channel here because it's not at the End protocol state
        // Instead, we need to continue with the protocol or return an error
        return Err(Error::Protocol("Server rejected the command"));
    }
    
    // Step 3: Send data to the server
    println!("Client: Sending data");
    let data = vec![1, 2, 3, 4, 5];
    let chan = chan.send(data).await?;
    println!("Client: Data sent successfully");
    
    // Step 4: Receive the processed result from the server
    println!("Client: Waiting for result");
    let (result, chan) = chan.recv().await?;
    println!("Client: Received result: {:?}", result);
    
    // Step 5: Close the channel
    chan.close()?;
    println!("Client: Protocol completed successfully");
    
    Ok(())
}

/// Implements the server side of the protocol
async fn run_server(chan: Chan<ServerProtocol, AsyncStdChannel>) -> Result<(), Error> {
    println!("Server: Starting protocol");
    
    // Step 1: Receive a command from the client
    println!("Server: Waiting for command");
    let (command, chan) = chan.recv().await?;
    println!("Server: Received command: {}", command);
    
    // Step 2: Check if the command is valid and send status
    println!("Server: Validating command");
    let is_valid = command == "process";
    let chan = chan.send(is_valid).await?;
    println!("Server: Status sent successfully: {}", is_valid);
    
    if !is_valid {
        println!("Server: Invalid command, ending protocol");
        // We can't close the channel here because it's not at the End protocol state
        // Instead, we need to continue with the protocol or return an error
        return Err(Error::Protocol("Invalid command"));
    }
    
    // Step 3: Receive data from the client
    println!("Server: Waiting for data");
    let (data, chan) = chan.recv().await?;
    println!("Server: Received data: {:?}", data);
    
    // Step 4: Process the data and send the result
    println!("Server: Processing data");
    let result: Vec<i32> = data.iter().map(|x| x * x).collect(); // Square each element
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
    let (client_channel, server_channel) = create_async_std_channel_pair();
    
    // Create client and server channels with their respective protocols
    let client_chan = Chan::<ClientProtocol, _>::new(client_channel);
    let server_chan = Chan::<ServerProtocol, _>::new(server_channel);
    
    // Spawn the server in a separate task
    // This requires the futures to implement Send
    let server_handle = task::spawn(async move {
        run_server(server_chan).await
    });
    
    // Run the client in the main task
    run_client(client_chan).await?;
    
    // Wait for the server to complete
    server_handle.await?;
    
    println!("Send trait demonstration completed successfully");
    Ok(())
}

#[async_std::main]
async fn main() -> Result<(), Error> {
    println!("=== async-std Integration Example ===\n");
    
    // Demonstrate the Send trait implementation
    demonstrate_send_trait().await?;
    
    println!("\n=== Example completed successfully ===");
    Ok(())
}