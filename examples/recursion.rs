//! # Recursive Protocol Example
//!
//! This example demonstrates the concept of recursive protocols using session types.
//! It implements a client-server interaction where the client can repeatedly request
//! data from the server until it decides to stop.
//!
//! While this example simulates recursion using loops rather than using the actual
//! `Rec<P>` and `Var<const N: usize>` types (due to current library limitations),
//! it illustrates the key concepts of recursive protocols:
//!
//! 1. A protocol that can loop back to a previous state
//! 2. A bounded recursion with a clear termination condition
//! 3. A practical use case for recursion in session types
//!
//! ## Protocol Diagram
//!
//! ```text
//! Client                                Server
//!   |                                     |
//!   |                                     |
//!   |<------ Loop start ----------------->|
//!   |                                     |
//!   |------- Choose ---------------------->|
//!   |         /      \                    |
//!   |        /        \                   |
//!   |       /          \                  |
//!   | Continue          Stop              |
//!   |     |              |                |
//!   |     |              |                |
//!   |-----|-- Send(String) - Query ------>|
//!   |     |                               |
//!   |<----|-- Recv(String) - Response ----|
//!   |     |                               |
//!   |-----|-- Loop back ---------------->|
//!   |     |                               |
//!   |     |                               |
//!   |<----|---------------------------|   |
//!   |                                 |   |
//!   |-------------- End ---------------->|
//! ```
//!
//! This protocol represents a client-server interaction where:
//! 1. The client can choose to either continue or stop the interaction
//! 2. If the client chooses to continue:
//!    a. The client sends a query (String) to the server
//!    b. The server responds with an answer (String)
//!    c. The protocol loops back to the beginning
//! 3. If the client chooses to stop, the communication ends
//!
//! ## Recursive Protocol Definition
//!
//! In a fully implemented recursive protocol, we would define it as:
//!
//! ```rust
//! // Define the client's protocol using recursion
//! type ClientProtocol = Rec<Choose<Send<String, Recv<String, Var<0>>>, End>>;
//! ```
//!
//! Where:
//! - `Rec<P>` defines a recursive protocol
//! - `Var<0>` refers back to the enclosing `Rec<P>`
//! - `Choose<A, B>` allows the client to choose between two continuations
//! - `Send<T, P>` sends a value of type T and continues with protocol P
//! - `Recv<T, P>` receives a value of type T and continues with protocol P
//! - `End` terminates the protocol

use sessrums::chan::Chan;
use sessrums::proto::{End, Protocol, Recv, Send};
use sessrums::error::Error;
use std::sync::mpsc;
use std::thread;
use futures_core::future::Future;
use std::pin::Pin;
use futures_core::task::{Context, Poll};
use std::marker::PhantomData;

// Define a simple protocol for a single request-response interaction
type SimpleProtocol = Send<String, Recv<String, End>>;
type ServerProtocol = <SimpleProtocol as Protocol>::Dual;

// A bidirectional channel that can both send and receive values
struct BiChannel<T> {
    sender: mpsc::Sender<T>,
    receiver: mpsc::Receiver<T>,
}

// Define futures for BiChannel
struct BiChannelSendFuture<T: Default + Unpin> {
    sender: mpsc::Sender<T>,
    value: Option<T>,
}

struct BiChannelRecvFuture<T: Unpin> {
    receiver: *mut mpsc::Receiver<T>,
    _marker: PhantomData<T>,
}

// Implement Future for BiChannelSendFuture
impl<T: Default + Unpin> Future for BiChannelSendFuture<T> {
    type Output = Result<(), mpsc::SendError<T>>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Some(value) = this.value.take() {
            match this.sender.send(value) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(e) => Poll::Ready(Err(e)),
            }
        } else {
            Poll::Ready(Err(mpsc::SendError(Default::default())))
        }
    }
}

// Implement Future for BiChannelRecvFuture
impl<T: Unpin> Future for BiChannelRecvFuture<T> {
    type Output = Result<T, mpsc::RecvError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        // Safety: We know the pointer is valid for the lifetime of the future
        let receiver = unsafe { &mut *this.receiver };
        match receiver.recv() {
            Ok(value) => Poll::Ready(Ok(value)),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

// Implement AsyncSender for BiChannel
impl<T: Default + Unpin> sessrums::io::AsyncSender<T> for BiChannel<T> {
    type Error = mpsc::SendError<T>;
    type SendFuture<'a> = BiChannelSendFuture<T> where T: 'a, Self: 'a;

    fn send(&mut self, value: T) -> Self::SendFuture<'_> {
        BiChannelSendFuture {
            sender: self.sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver for BiChannel
impl<T: Unpin> sessrums::io::AsyncReceiver<T> for BiChannel<T> {
    type Error = mpsc::RecvError;
    type RecvFuture<'a> = BiChannelRecvFuture<T> where T: 'a, Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        BiChannelRecvFuture {
            receiver: &mut self.receiver as *mut mpsc::Receiver<T>,
            _marker: PhantomData,
        }
    }
}

/// Creates a pair of bidirectional channels for communication
fn create_channel_pair<T>() -> (BiChannel<T>, BiChannel<T>) {
    let (client_tx, server_rx) = mpsc::channel();
    let (server_tx, client_rx) = mpsc::channel();
    
    let client_channel = BiChannel {
        sender: client_tx,
        receiver: client_rx,
    };
    
    let server_channel = BiChannel {
        sender: server_tx,
        receiver: server_rx,
    };
    
    (client_channel, server_channel)
}

/// Implements the client side of the protocol
async fn run_client(max_iterations: usize) -> Result<(), Error> {
    println!("Client: Starting protocol");
    
    // Keep track of the number of iterations
    let mut iterations = 0;
    
    // Loop until we reach the maximum number of iterations
    while iterations < max_iterations {
        println!("Client: Iteration {}/{}", iterations + 1, max_iterations);
        
        // Create a new channel for this iteration
        let (client_channel, server_channel) = create_channel_pair();
        
        // Create client and server channels with their respective protocols
        let client_chan = Chan::<SimpleProtocol, _>::new(client_channel);
        
        // Start the server for this iteration
        let server_handle = thread::spawn(move || {
            let server_future = run_server_iteration(Chan::<ServerProtocol, _>::new(server_channel));
            
            // Create a runtime for the server thread
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(server_future)
        });
        
        // Send a query to the server
        println!("Client: Sending query");
        let query = format!("What is the {}th Fibonacci number?", iterations + 1);
        let chan_after_send = client_chan.send(query).await?;
        
        // Receive the response from the server
        println!("Client: Waiting for response");
        let (response, chan_after_recv) = chan_after_send.recv().await?;
        println!("Client: Received response: {}", response);
        
        // Close the channel
        println!("Client: Closing channel for this iteration");
        chan_after_recv.close()?;
        
        // Wait for the server to complete
        server_handle.join().unwrap()?;
        
        iterations += 1;
    }
    
    println!("Client: All iterations completed successfully");
    Ok(())
}

/// Implements the server side of the protocol for a single iteration
async fn run_server_iteration(chan: Chan<ServerProtocol, BiChannel<String>>) -> Result<(), Error> {
    println!("Server: Starting iteration");
    
    // Receive the query from the client
    println!("Server: Waiting for query");
    let (query, chan_after_recv) = chan.recv().await?;
    let query_string = query.to_string();
    println!("Server: Received query: {}", query_string);
    
    // Process the query and send a response
    println!("Server: Processing query");
    let response = process_query(&query_string);
    println!("Server: Sending response");
    let chan_after_send = chan_after_recv.send(response).await?;
    
    // Close the channel
    println!("Server: Closing channel for this iteration");
    chan_after_send.close()?;
    
    println!("Server: Iteration completed successfully");
    Ok(())
}

/// Process a query and return a response
fn process_query(query: &str) -> String {
    // Extract the Fibonacci number index from the query
    if let Some(index_str) = query.strip_prefix("What is the ").and_then(|s| s.strip_suffix("th Fibonacci number?")) {
        if let Ok(index) = index_str.parse::<usize>() {
            let result = fibonacci(index);
            return format!("The {}th Fibonacci number is {}", index, result);
        }
    }
    
    "I don't understand that question.".to_string()
}

/// Calculate the nth Fibonacci number
fn fibonacci(n: usize) -> usize {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0;
            let mut b = 1;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("=== Recursive Protocol Example ===\n");
    println!("This example demonstrates the concept of recursive protocols using session types.");
    println!("It simulates a recursive protocol where a client can repeatedly request data");
    println!("from a server until it decides to stop.\n");
    println!("Note: Due to current library limitations, this example simulates recursion");
    println!("using loops rather than using the actual Rec<P> and Var<N> types.\n");
    
    // Set the maximum number of iterations
    let max_iterations = 5;
    
    // Run the client
    run_client(max_iterations).await?;
    
    println!("\n=== Example completed successfully ===");
    Ok(())
}