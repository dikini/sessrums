//! # Asynchronous Protocol Example
//!
//! This example demonstrates the use of asynchronous session types for
//! implementing a bidirectional communication protocol between two parties.
//! It showcases the use of `AsyncSender` and `AsyncReceiver` traits along with
//! the `send`, `recv`, `offer`, and `choose` methods for asynchronous communication.
//!
//! ## Protocol Diagram
//!
//! ```text
//! Client                                Server
//!   |                                     |
//!   |------- Send(String) - Request ----->|
//!   |                                     |
//!   |                                     |  Server processes request
//!   |                                     |  and decides response type
//!   |                                     |
//!   |<------ Choose ----------------------|
//!   |         /      \                    |
//!   |        /        \                   |
//!   |       /          \                  |
//!   |  Success          Error             |
//!   |     |              |                |
//!   |<--- Recv(i32) -----|                |
//!   |                    |                |
//!   |<--- Recv(String) --|                |
//!   |                                     |
//!   |-------------- End ---------------->|
//! ```
//!
//! This protocol represents a client-server interaction where:
//! 1. The client sends a request (String) to the server
//! 2. The server processes the request and chooses between two response types:
//!    - Success: Returns a numeric result (i32)
//!    - Error: Returns an error message (String)
//! 3. The communication ends

use futures_core::future::Future;
use futures_core::task::{Context, Poll};
use sez::chan::Chan;
use sez::error::Error;
use sez::io::{AsyncReceiver, AsyncSender};
use sez::proto::{End, Offer, Protocol, Recv, Send as ProtoSend};
use std::marker::{PhantomData, Send};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc as tokio_mpsc;

// Define the protocol types for client and server

// Client protocol: Send a request, then offer a choice between receiving a success value or an error message
type ClientProtocol = ProtoSend<String, Offer<Recv<i32, End>, Recv<String, End>>>;

// Server protocol: Receive a request, then choose between sending a success value or an error message
// This is the dual of the client protocol
type ServerProtocol = <ClientProtocol as Protocol>::Dual;

// Alternatively, we could explicitly define the server protocol as:
// type ServerProtocol = Recv<String, Choose<Send<i32, End>, Send<String, End>>>;

/// A bidirectional channel implementation using tokio's mpsc channels
/// This allows for asynchronous communication between the client and server
struct AsyncChannel<T: Clone + std::marker::Unpin> {
    sender: tokio_mpsc::Sender<T>,
    receiver: Arc<Mutex<tokio_mpsc::Receiver<T>>>,
}

// Define futures for AsyncChannel
struct AsyncChannelSendFuture<T> {
    sender: tokio_mpsc::Sender<T>,
    value: Option<T>,
}

struct AsyncChannelRecvFuture<T> {
    receiver: Arc<Mutex<tokio_mpsc::Receiver<T>>>,
    _marker: PhantomData<T>,
}

// Implement Future for AsyncChannelSendFuture
impl<T: std::marker::Unpin> Future for AsyncChannelSendFuture<T> {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        if let Some(value) = this.value.take() {
            match this.sender.try_send(value) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(tokio_mpsc::error::TrySendError::Full(value)) => {
                    // Put the value back and register the waker
                    this.value = Some(value);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(tokio_mpsc::error::TrySendError::Closed(_)) => {
                    Poll::Ready(Err("Channel closed".to_string()))
                }
            }
        } else {
            Poll::Ready(Err("No value to send".to_string()))
        }
    }
}

// Implement Future for AsyncChannelRecvFuture
impl<T: std::marker::Unpin> Future for AsyncChannelRecvFuture<T> {
    type Output = Result<T, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        // Try to lock the mutex
        match this.receiver.try_lock() {
            Ok(mut receiver) => {
                // Try to receive a value
                match receiver.try_recv() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(tokio_mpsc::error::TryRecvError::Empty) => {
                        // Register the waker and return Pending
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(tokio_mpsc::error::TryRecvError::Disconnected) => {
                        Poll::Ready(Err("Channel disconnected".to_string()))
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

// Implement AsyncSender for AsyncChannel
impl<T: Clone + std::marker::Unpin> AsyncSender<T> for AsyncChannel<T> {
    type Error = String;
    type SendFuture<'a> = AsyncChannelSendFuture<T> where T: 'a, Self: 'a;

    fn send(&mut self, value: T) -> Self::SendFuture<'_> {
        AsyncChannelSendFuture {
            sender: self.sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver for AsyncChannel
impl<T: Clone + std::marker::Unpin> AsyncReceiver<T> for AsyncChannel<T> {
    type Error = String;
    type RecvFuture<'a> = AsyncChannelRecvFuture<T> where T: 'a, Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        AsyncChannelRecvFuture {
            receiver: self.receiver.clone(),
            _marker: PhantomData,
        }
    }
}

// Create a specialized AsyncChannel for our protocol
struct ProtocolChannel {
    string_sender: tokio_mpsc::Sender<String>,
    string_receiver: Arc<Mutex<tokio_mpsc::Receiver<String>>>,
    int_sender: tokio_mpsc::Sender<i32>,
    int_receiver: Arc<Mutex<tokio_mpsc::Receiver<i32>>>,
    bool_sender: tokio_mpsc::Sender<bool>,
    bool_receiver: Arc<Mutex<tokio_mpsc::Receiver<bool>>>,
}

// Define futures for ProtocolChannel
struct StringSendFuture {
    sender: tokio_mpsc::Sender<String>,
    value: Option<String>,
}

struct IntSendFuture {
    sender: tokio_mpsc::Sender<i32>,
    value: Option<i32>,
}

struct BoolSendFuture {
    sender: tokio_mpsc::Sender<bool>,
    value: Option<bool>,
}

struct StringRecvFuture {
    receiver: Arc<Mutex<tokio_mpsc::Receiver<String>>>,
}

struct IntRecvFuture {
    receiver: Arc<Mutex<tokio_mpsc::Receiver<i32>>>,
}

struct BoolRecvFuture {
    receiver: Arc<Mutex<tokio_mpsc::Receiver<bool>>>,
}

// Implement Future for StringSendFuture
impl Future for StringSendFuture {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        if let Some(value) = this.value.take() {
            match this.sender.try_send(value) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(tokio_mpsc::error::TrySendError::Full(value)) => {
                    // Put the value back and register the waker
                    this.value = Some(value);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(tokio_mpsc::error::TrySendError::Closed(_)) => {
                    Poll::Ready(Err("Channel closed".to_string()))
                }
            }
        } else {
            Poll::Ready(Err("No value to send".to_string()))
        }
    }
}

// Implement Future for IntSendFuture
impl Future for IntSendFuture {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        if let Some(value) = this.value.take() {
            match this.sender.try_send(value) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(tokio_mpsc::error::TrySendError::Full(value)) => {
                    // Put the value back and register the waker
                    this.value = Some(value);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(tokio_mpsc::error::TrySendError::Closed(_)) => {
                    Poll::Ready(Err("Channel closed".to_string()))
                }
            }
        } else {
            Poll::Ready(Err("No value to send".to_string()))
        }
    }
}

// Implement Future for BoolSendFuture
impl Future for BoolSendFuture {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        if let Some(value) = this.value.take() {
            match this.sender.try_send(value) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(tokio_mpsc::error::TrySendError::Full(value)) => {
                    // Put the value back and register the waker
                    this.value = Some(value);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(tokio_mpsc::error::TrySendError::Closed(_)) => {
                    Poll::Ready(Err("Channel closed".to_string()))
                }
            }
        } else {
            Poll::Ready(Err("No value to send".to_string()))
        }
    }
}

// Implement Future for StringRecvFuture
impl Future for StringRecvFuture {
    type Output = Result<String, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        // Try to lock the mutex
        match this.receiver.try_lock() {
            Ok(mut receiver) => {
                // Try to receive a value
                match receiver.try_recv() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(tokio_mpsc::error::TryRecvError::Empty) => {
                        // Register the waker and return Pending
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(tokio_mpsc::error::TryRecvError::Disconnected) => {
                        Poll::Ready(Err("Channel disconnected".to_string()))
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
    type Output = Result<i32, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        // Try to lock the mutex
        match this.receiver.try_lock() {
            Ok(mut receiver) => {
                // Try to receive a value
                match receiver.try_recv() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(tokio_mpsc::error::TryRecvError::Empty) => {
                        // Register the waker and return Pending
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(tokio_mpsc::error::TryRecvError::Disconnected) => {
                        Poll::Ready(Err("Channel disconnected".to_string()))
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
    type Output = Result<bool, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        // Try to lock the mutex
        match this.receiver.try_lock() {
            Ok(mut receiver) => {
                // Try to receive a value
                match receiver.try_recv() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(tokio_mpsc::error::TryRecvError::Empty) => {
                        // Register the waker and return Pending
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(tokio_mpsc::error::TryRecvError::Disconnected) => {
                        Poll::Ready(Err("Channel disconnected".to_string()))
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

// Implement AsyncSender<String> for ProtocolChannel
impl AsyncSender<String> for ProtocolChannel {
    type Error = String;
    type SendFuture<'a> = StringSendFuture where Self: 'a;

    fn send(&mut self, value: String) -> Self::SendFuture<'_> {
        StringSendFuture {
            sender: self.string_sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver<String> for ProtocolChannel
impl AsyncReceiver<String> for ProtocolChannel {
    type Error = String;
    type RecvFuture<'a> = StringRecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        StringRecvFuture {
            receiver: self.string_receiver.clone(),
        }
    }
}

// Implement AsyncSender<i32> for ProtocolChannel
impl AsyncSender<i32> for ProtocolChannel {
    type Error = String;
    type SendFuture<'a> = IntSendFuture where Self: 'a;

    fn send(&mut self, value: i32) -> Self::SendFuture<'_> {
        IntSendFuture {
            sender: self.int_sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver<i32> for ProtocolChannel
impl AsyncReceiver<i32> for ProtocolChannel {
    type Error = String;
    type RecvFuture<'a> = IntRecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        IntRecvFuture {
            receiver: self.int_receiver.clone(),
        }
    }
}

// Implement AsyncSender<bool> for ProtocolChannel
impl AsyncSender<bool> for ProtocolChannel {
    type Error = String;
    type SendFuture<'a> = BoolSendFuture where Self: 'a;

    fn send(&mut self, value: bool) -> Self::SendFuture<'_> {
        BoolSendFuture {
            sender: self.bool_sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver<bool> for ProtocolChannel
impl AsyncReceiver<bool> for ProtocolChannel {
    type Error = String;
    type RecvFuture<'a> = BoolRecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        BoolRecvFuture {
            receiver: self.bool_receiver.clone(),
        }
    }
}

/// Creates a pair of AsyncChannel instances for bidirectional communication
#[allow(dead_code)]
fn create_async_channel_pair<T: Clone + std::marker::Unpin>(buffer_size: usize) -> (AsyncChannel<T>, AsyncChannel<T>) {
    // Create channels for client -> server and server -> client communication
    let (client_tx, server_rx) = tokio_mpsc::channel(buffer_size);
    let (server_tx, client_rx) = tokio_mpsc::channel(buffer_size);
    
    let client_channel = AsyncChannel {
        sender: client_tx,
        receiver: Arc::new(Mutex::new(client_rx)),
    };
    
    let server_channel = AsyncChannel {
        sender: server_tx,
        receiver: Arc::new(Mutex::new(server_rx)),
    };
    
    (client_channel, server_channel)
}

/// Creates a pair of ProtocolChannel instances for bidirectional communication
fn create_protocol_channel_pair() -> (ProtocolChannel, ProtocolChannel) {
    // Create channels for client -> server and server -> client communication
    let (client_string_tx, server_string_rx) = tokio_mpsc::channel(10);
    let (server_string_tx, client_string_rx) = tokio_mpsc::channel(10);
    
    let (client_int_tx, server_int_rx) = tokio_mpsc::channel(10);
    let (server_int_tx, client_int_rx) = tokio_mpsc::channel(10);
    
    let (client_bool_tx, server_bool_rx) = tokio_mpsc::channel(10);
    let (server_bool_tx, client_bool_rx) = tokio_mpsc::channel(10);
    
    let client_channel = ProtocolChannel {
        string_sender: client_string_tx,
        string_receiver: Arc::new(Mutex::new(client_string_rx)),
        int_sender: client_int_tx,
        int_receiver: Arc::new(Mutex::new(client_int_rx)),
        bool_sender: client_bool_tx,
        bool_receiver: Arc::new(Mutex::new(client_bool_rx)),
    };
    
    let server_channel = ProtocolChannel {
        string_sender: server_string_tx,
        string_receiver: Arc::new(Mutex::new(server_string_rx)),
        int_sender: server_int_tx,
        int_receiver: Arc::new(Mutex::new(server_int_rx)),
        bool_sender: server_bool_tx,
        bool_receiver: Arc::new(Mutex::new(server_bool_rx)),
    };
    
    (client_channel, server_channel)
}

/// Implements the client side of the protocol
async fn run_client(chan: Chan<ClientProtocol, ProtocolChannel>) -> Result<(), Error> {
    println!("Client: Starting protocol");
    
    // Step 1: Send a request to the server
    println!("Client: Sending request");
    let request = "calculate:fibonacci:10".to_string();
    let chan = chan.send(request).await?;
    println!("Client: Request sent successfully");
    
    // Step 2: Offer a choice to the server and handle the chosen branch
    println!("Client: Waiting for server's choice");
    // Use the offer method with a different approach
    let _result = chan.offer(
        |chan| {
            // Create a function that returns a boxed future
            fn handle_success(chan: Chan<Recv<i32, End>, ProtocolChannel>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>> {
                Box::pin(async move {
                    println!("Client: Server chose success branch");
                    let (result, chan) = chan.recv().await?;
                    println!("Client: Received result: {}", result);
                    
                    // Close the channel
                    chan.close()?;
                    println!("Client: Protocol completed successfully with result: {}", result);
                    Ok(())
                })
            }
            
            Ok(handle_success(chan))
        },
        |chan| {
            // Create a function that returns a boxed future
            fn handle_error(chan: Chan<Recv<String, End>, ProtocolChannel>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>> {
                Box::pin(async move {
                    println!("Client: Server chose error branch");
                    let (error_msg, chan) = chan.recv().await?;
                    println!("Client: Received error: {}", error_msg);
                    
                    // Close the channel
                    chan.close()?;
                    println!("Client: Protocol completed with error: {}", error_msg);
                    Ok(())
                })
            }
            
            Ok(handle_error(chan))
        }
    ).await?;
    
    Ok(())
}

/// Implements the server side of the protocol
async fn run_server(chan: Chan<ServerProtocol, ProtocolChannel>) -> Result<(), Error> {
    println!("Server: Starting protocol");
    
    // Step 1: Receive a request from the client
    println!("Server: Waiting for request");
    let (request, chan) = chan.recv().await?;
    println!("Server: Received request: {}", request);
    
    // Step 2: Process the request and choose a response type
    println!("Server: Processing request");
    
    // Parse the request (format: "command:type:value")
    let parts: Vec<&str> = request.split(':').collect();
    
    if parts.len() != 3 {
        // Invalid request format, choose the error branch
        println!("Server: Invalid request format, sending error");
        let chan = chan.choose_right().await?;
        
        // Send an error message
        let error_msg = "Invalid request format. Expected 'command:type:value'".to_string();
        let chan = chan.send(error_msg).await?;
        
        // Close the channel
        chan.close()?;
        println!("Server: Protocol completed with error response");
        return Ok(());
    }
    
    let command = parts[0];
    let calc_type = parts[1];
    let value_str = parts[2];
    
    // Parse the value
    let value = match value_str.parse::<i32>() {
        Ok(v) => v,
        Err(_) => {
            // Invalid value, choose the error branch
            println!("Server: Invalid value, sending error");
            let chan = chan.choose_right().await?;
            
            // Send an error message
            let error_msg = format!("Invalid value: '{}'. Expected an integer.", value_str);
            let chan = chan.send(error_msg).await?;
            
            // Close the channel
            chan.close()?;
            println!("Server: Protocol completed with error response");
            return Ok(());
        }
    };
    
    // Process the command
    if command != "calculate" {
        // Unknown command, choose the error branch
        println!("Server: Unknown command, sending error");
        let chan = chan.choose_right().await?;
        
        // Send an error message
        let error_msg = format!("Unknown command: '{}'. Expected 'calculate'.", command);
        let chan = chan.send(error_msg).await?;
        
        // Close the channel
        chan.close()?;
        println!("Server: Protocol completed with error response");
        return Ok(());
    }
    
    // Calculate the result based on the calculation type
    match calc_type {
        "fibonacci" => {
            // Calculate the fibonacci number
            println!("Server: Calculating fibonacci({})...", value);
            let result = fibonacci(value);
            
            // Choose the success branch
            println!("Server: Calculation successful, sending result");
            let chan = chan.choose_left().await?;
            
            // Send the result
            let chan = chan.send(result).await?;
            
            // Close the channel
            chan.close()?;
            println!("Server: Protocol completed successfully");
            Ok(())
        }
        "factorial" => {
            // Calculate the factorial
            println!("Server: Calculating factorial({})...", value);
            let result = factorial(value);
            
            // Choose the success branch
            println!("Server: Calculation successful, sending result");
            let chan = chan.choose_left().await?;
            
            // Send the result
            let chan = chan.send(result).await?;
            
            // Close the channel
            chan.close()?;
            println!("Server: Protocol completed successfully");
            Ok(())
        }
        _ => {
            // Unknown calculation type, choose the error branch
            println!("Server: Unknown calculation type, sending error");
            let chan = chan.choose_right().await?;
            
            // Send an error message
            let error_msg = format!("Unknown calculation type: '{}'. Expected 'fibonacci' or 'factorial'.", calc_type);
            let chan = chan.send(error_msg).await?;
            
            // Close the channel
            chan.close()?;
            println!("Server: Protocol completed with error response");
            Ok(())
        }
    }
}

/// Calculate the nth fibonacci number
fn fibonacci(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    } else if n == 1 {
        return 1;
    }
    
    let mut a = 0;
    let mut b = 1;
    
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    
    b
}

/// Calculate the factorial of n
fn factorial(n: i32) -> i32 {
    if n <= 0 {
        return 1;
    }
    
    let mut result = 1;
    for i in 1..=n {
        result *= i;
    }
    
    result
}

/// Demonstrates error handling with a custom IO implementation that fails on recv
async fn demonstrate_error_handling() -> Result<(), Error> {
    println!("\nDemonstrating error handling:");
    
    // Create a custom IO implementation that fails on recv
    struct FailingIO;
    
    // Define futures for FailingIO
    struct FailingIOSendFuture<T> {
        success: bool,
        _marker: PhantomData<T>,
    }

    struct FailingIORecvFuture<T> {
        success: bool,
        _marker: PhantomData<T>,
    }

    // Implement Future for FailingIOSendFuture
    impl<T> Future for FailingIOSendFuture<T> {
        type Output = Result<(), String>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.success {
                println!("Send operation succeeded");
                Poll::Ready(Ok(()))
            } else {
                println!("Send operation failed");
                Poll::Ready(Err("Send failed".to_string()))
            }
        }
    }

    // Implement Future for FailingIORecvFuture
    impl<T> Future for FailingIORecvFuture<T> {
        type Output = Result<T, String>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.success {
                println!("Receive operation succeeded");
                // We can't actually return a T here, so we'll always fail
                Poll::Ready(Err("Cannot create value of type T".to_string()))
            } else {
                println!("Receive operation failed");
                Poll::Ready(Err("Receive failed".to_string()))
            }
        }
    }

    // Implement AsyncSender<String> for FailingIO
    impl AsyncSender<String> for FailingIO {
        type Error = String;
        type SendFuture<'a> = FailingIOSendFuture<String> where Self: 'a;

        fn send(&mut self, _value: String) -> Self::SendFuture<'_> {
            FailingIOSendFuture {
                success: true,
                _marker: PhantomData,
            }
        }
    }
    
    // Implement AsyncReceiver<String> for FailingIO
    impl AsyncReceiver<String> for FailingIO {
        type Error = String;
        type RecvFuture<'a> = FailingIORecvFuture<String> where Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            FailingIORecvFuture {
                success: false,
                _marker: PhantomData,
            }
        }
    }
    
    // Implement AsyncSender<bool> for FailingIO
    impl AsyncSender<bool> for FailingIO {
        type Error = String;
        type SendFuture<'a> = FailingIOSendFuture<bool> where Self: 'a;

        fn send(&mut self, _value: bool) -> Self::SendFuture<'_> {
            FailingIOSendFuture {
                success: true,
                _marker: PhantomData,
            }
        }
    }
    
    // Implement AsyncReceiver<bool> for FailingIO
    impl AsyncReceiver<bool> for FailingIO {
        type Error = String;
        type RecvFuture<'a> = FailingIORecvFuture<bool> where Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            FailingIORecvFuture {
                success: false,
                _marker: PhantomData,
            }
        }
    }
    
    // Implement AsyncSender<i32> for FailingIO
    impl AsyncSender<i32> for FailingIO {
        type Error = String;
        type SendFuture<'a> = FailingIOSendFuture<i32> where Self: 'a;

        fn send(&mut self, _value: i32) -> Self::SendFuture<'_> {
            FailingIOSendFuture {
                success: true,
                _marker: PhantomData,
            }
        }
    }
    
    // Implement AsyncReceiver<i32> for FailingIO
    impl AsyncReceiver<i32> for FailingIO {
        type Error = String;
        type RecvFuture<'a> = FailingIORecvFuture<i32> where Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            FailingIORecvFuture {
                success: false,
                _marker: PhantomData,
            }
        }
    }
    
    
    // Create a client channel with our failing IO
    let chan = Chan::<ClientProtocol, _>::new(FailingIO);
    
    // Send should work
    println!("Attempting to send a message...");
    let chan = chan.send("Hello".to_string()).await.map_err(|_| {
        Error::Protocol("Unexpected send error")
    })?;
    
    // Offer should fail because it tries to receive
    println!("Attempting to offer (should fail)...");
    match chan.offer(
        |_| {
            // Create a function that returns a boxed future
            fn dummy_handler() -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>> {
                Box::pin(async { Ok(()) })
            }
            
            Ok(dummy_handler())
        },
        |_| {
            // Create a function that returns a boxed future
            fn dummy_handler() -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>> {
                Box::pin(async { Ok(()) })
            }
            
            Ok(dummy_handler())
        }
    ).await {
        Ok(_) => println!("Unexpected success!"),
        Err(e) => {
            println!("Received expected error: {}", e);
            return Ok(());
        }
    }
    
    // We should never reach here
    Err(Error::Protocol("Expected an error but got success"))
}

/// Demonstrates type safety through the type system
mod type_safety_examples {
    use super::*;
    
    // Define a protocol: Send a String, then offer a choice between receiving an i32 or a String
    type MyProtocol = ProtoSend<String, Offer<Recv<i32, End>, Recv<String, End>>>;
    
    // The following function compiles because it follows the protocol correctly
    #[allow(dead_code)]
    async fn correct_protocol_usage(chan: Chan<MyProtocol, ProtocolChannel>) -> Result<(), Error> {
        // First send a String as required by the protocol
        let chan = chan.send("Hello".to_string()).await?;
        
        // Then offer a choice as required by the protocol
        let _result = chan.offer(
            |chan| {
                // Create a function that returns a boxed future
                fn handle_int(chan: Chan<Recv<i32, End>, ProtocolChannel>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>> {
                    Box::pin(async move {
                        let (value, chan) = chan.recv().await?;
                        println!("Received i32: {}", value);
                        chan.close()?;
                        Ok(())
                    })
                }
                
                Ok(handle_int(chan))
            },
            |chan| {
                // Create a function that returns a boxed future
                fn handle_string(chan: Chan<Recv<String, End>, ProtocolChannel>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>> {
                    Box::pin(async move {
                        let (value, chan) = chan.recv().await?;
                        println!("Received String: {}", value);
                        chan.close()?;
                        Ok(())
                    })
                }
                
                Ok(handle_string(chan))
            }
        ).await?;
        
        Ok(())
    }
    
    // The following function would not compile because it violates the protocol
    // by trying to send after offering
    /*
    #[allow(dead_code)]
    async fn incorrect_protocol_usage(chan: Chan<MyProtocol, AsyncChannel<String>>) -> Result<(), Error> {
        // First send a String as required by the protocol
        let chan = chan.send("Hello".to_string()).await?;
        
        // Error: The protocol requires offering a choice next, but we're trying to send again
        let chan = chan.send("Another message".to_string()).await?;
        
        // Then offer a choice
        let result = chan.offer(
            |chan| async move { Ok(()) },
            |chan| async move { Ok(()) }
        ).await?;
        
        Ok(result)
    }
    */
    
    // The following function would not compile because it violates the protocol
    // by trying to offer before sending
    /*
    #[allow(dead_code)]
    async fn incorrect_order_usage(chan: Chan<MyProtocol, AsyncChannel<String>>) -> Result<(), Error> {
        // Error: The protocol requires sending a String first, but we're trying to offer a choice
        let result = chan.offer(
            |chan| async move { Ok(()) },
            |chan| async move { Ok(()) }
        ).await?;
        
        // Then send a String
        let chan = chan.send("Hello".to_string()).await?;
        
        Ok(result)
    }
    */
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("=== Asynchronous Protocol Example ===\n");
    
    // Create a channel pair for communication
    let (client_channel, server_channel) = create_protocol_channel_pair();
    
    // Create client and server channels with their respective protocols
    let client_chan = Chan::<ClientProtocol, _>::new(client_channel);
    let server_chan = Chan::<ServerProtocol, _>::new(server_channel);
    
    // Run the server in a separate task
    let server_handle = tokio::spawn(async move {
        run_server(server_chan).await
    });
    
    // Run the client in the main task
    run_client(client_chan).await?;
    
    // Wait for the server to complete
    server_handle.await.unwrap()?;
    
    // Demonstrate error handling
    demonstrate_error_handling().await?;
    
    println!("\n=== Example completed successfully ===");
    Ok(())
}