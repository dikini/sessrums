//! # Request-Response Protocol Example
//!
//! This example demonstrates a simple request-response protocol implementation using session types.
//! It shows how to define protocols, create channels, and use the `send`, `recv`,
//! and `close` methods to communicate according to the protocol.
//!
//! ## Protocol Description
//!
//! The protocol implemented here is a simple request-response pattern:
//!
//! ```text
//! Client                                Server
//!   |                                     |
//!   |------- Send(Request) ------------->|
//!   |                                     |
//!   |<------ Recv(Response) -------------|
//!   |                                     |
//!   |-------------- End ---------------->|
//! ```
//!
//! This is a common pattern in client-server interactions where:
//! 1. The client sends a request
//! 2. The server processes the request and sends a response
//! 3. The communication ends

use sez::chan::Chan;
use sez::proto::{End, Protocol, Recv, Send};
use sez::api::{RequestClient, RequestServer};
use sez::error::Error;
use std::thread;
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use futures_core::future::Future;
use std::pin::Pin;
use futures_core::task::{Context, Poll};

// Bidirectional channel that implements both AsyncSender and AsyncReceiver
struct BiChannel<T> {
    sender: mpsc::Sender<T>,
    receiver: Arc<Mutex<mpsc::Receiver<T>>>,
}

// Specialized channel for request-response communication
struct RequestChannel {
    sender: mpsc::Sender<Request>,
    receiver: Arc<Mutex<mpsc::Receiver<Response>>>,
}

// Specialized channel for response-request communication
struct ResponseChannel {
    sender: mpsc::Sender<Response>,
    receiver: Arc<Mutex<mpsc::Receiver<Request>>>,
}

// Future returned by BiChannel::send
struct SendFuture<T> {
    sender: mpsc::Sender<T>,
    value: Option<T>,
}

// Future returned by BiChannel::recv
struct RecvFuture<T> {
    receiver: Arc<Mutex<mpsc::Receiver<T>>>,
    _marker: std::marker::PhantomData<T>,
}

// Implement Future for SendFuture
impl<T: std::marker::Unpin> Future for SendFuture<T> {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        if let Some(value) = this.value.take() {
            match this.sender.try_send(value) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(mpsc::error::TrySendError::Full(value)) => {
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

// Implement Future for RecvFuture
impl<T: std::marker::Unpin> Future for RecvFuture<T> {
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        match this.receiver.try_lock() {
            Ok(mut receiver) => {
                match receiver.try_recv() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(mpsc::error::TryRecvError::Empty) => {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        Poll::Ready(Err(Error::ChannelClosed))
                    }
                }
            }
            Err(_) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

// Implement AsyncSender for BiChannel
impl<T: Clone + std::marker::Unpin> sez::io::AsyncSender<T> for BiChannel<T> {
    type Error = Error;
    type SendFuture<'a> = SendFuture<T> where T: 'a, Self: 'a;

    fn send(&mut self, value: T) -> Self::SendFuture<'_> {
        SendFuture {
            sender: self.sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver for BiChannel
impl<T: std::marker::Unpin> sez::io::AsyncReceiver<T> for BiChannel<T> {
    type Error = Error;
    type RecvFuture<'a> = RecvFuture<T> where T: 'a, Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        RecvFuture {
            receiver: self.receiver.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

// Implement AsyncSender for RequestChannel
impl sez::io::AsyncSender<Request> for RequestChannel {
    type Error = Error;
    type SendFuture<'a> = SendFuture<Request> where Request: 'a, Self: 'a;

    fn send(&mut self, value: Request) -> Self::SendFuture<'_> {
        SendFuture {
            sender: self.sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver for RequestChannel
impl sez::io::AsyncReceiver<Response> for RequestChannel {
    type Error = Error;
    type RecvFuture<'a> = RecvFuture<Response> where Response: 'a, Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        RecvFuture {
            receiver: self.receiver.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

// Implement AsyncSender for ResponseChannel
impl sez::io::AsyncSender<Response> for ResponseChannel {
    type Error = Error;
    type SendFuture<'a> = SendFuture<Response> where Response: 'a, Self: 'a;

    fn send(&mut self, value: Response) -> Self::SendFuture<'_> {
        SendFuture {
            sender: self.sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver for ResponseChannel
impl sez::io::AsyncReceiver<Request> for ResponseChannel {
    type Error = Error;
    type RecvFuture<'a> = RecvFuture<Request> where Request: 'a, Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        RecvFuture {
            receiver: self.receiver.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

/// Creates a pair of BiChannel instances for bidirectional communication
fn create_channel_pair<T>() -> (BiChannel<T>, BiChannel<T>) {
    // Create channels for client -> server and server -> client communication
    let (client_tx, server_rx) = mpsc::channel(10);
    let (server_tx, client_rx) = mpsc::channel(10);
    
    let client_channel = BiChannel {
        sender: client_tx,
        receiver: Arc::new(Mutex::new(client_rx)),
    };
    
    let server_channel = BiChannel {
        sender: server_tx,
        receiver: Arc::new(Mutex::new(server_rx)),
    };
    
    (client_channel, server_channel)
}

/// Creates a pair of specialized channels for request-response communication
fn create_request_response_pair() -> (RequestChannel, ResponseChannel) {
    // Create channels for client -> server and server -> client communication
    let (client_tx, server_rx) = mpsc::channel::<Request>(10);
    let (server_tx, client_rx) = mpsc::channel::<Response>(10);
    
    let client_channel = RequestChannel {
        sender: client_tx,
        receiver: Arc::new(Mutex::new(client_rx)),
    };
    
    let server_channel = ResponseChannel {
        sender: server_tx,
        receiver: Arc::new(Mutex::new(server_rx)),
    };
    
    (client_channel, server_channel)
}

/// Represents a request with a query string
#[derive(Clone, Debug)]
struct Request {
    query: String,
}

// Implement Unpin for Request
impl std::marker::Unpin for Request {}

/// Represents a response with a result string
#[derive(Clone, Debug)]
struct Response {
    result: String,
}

// Implement Unpin for Response
impl std::marker::Unpin for Response {}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request {{ query: {} }}", self.query)
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Response {{ result: {} }}", self.result)
    }
}

/// Implements the client side of the request-response protocol
async fn run_client(chan: Chan<RequestClient<Request, Response>, RequestChannel>, request: Request) -> Result<(), Error>
where
    Request: std::marker::Unpin,
    Response: std::marker::Unpin,
{
    println!("Client: Starting protocol");
    
    // Send a request to the server
    println!("Client: Sending request: {}", request);
    let chan = chan.send(request).await?;
    println!("Client: Request sent successfully");
    
    // Receive the response from the server
    println!("Client: Waiting for response");
    let (response, chan) = chan.recv().await?;
    println!("Client: Received response: {}", response);
    
    // Close the channel
    chan.close()?;
    println!("Client: Protocol completed successfully");
    
    Ok(())
}

/// Implements the server side of the request-response protocol
async fn run_server(chan: Chan<RequestServer<Request, Response>, ResponseChannel>) -> Result<(), Error>
where
    Request: std::marker::Unpin,
    Response: std::marker::Unpin,
{
    println!("Server: Starting protocol");
    
    // Receive a request from the client
    println!("Server: Waiting for request");
    let (request, chan) = chan.recv().await?;
    println!("Server: Received request: {}", request);
    
    // Process the request and send a response
    println!("Server: Processing request");
    let response = process_request(&request);
    println!("Server: Sending response: {}", response);
    let chan = chan.send(response).await?;
    println!("Server: Response sent successfully");
    
    // Close the channel
    chan.close()?;
    println!("Server: Protocol completed successfully");
    
    Ok(())
}

/// Process a request and return a response
fn process_request(request: &Request) -> Response {
    match request.query.as_str() {
        "What is the meaning of life?" => Response { result: "42".to_string() },
        "How are you?" => Response { result: "I'm fine, thank you!".to_string() },
        _ => Response { result: format!("Echo: {}", request.query) },
    }
}

/// Demonstrates using the request-response protocol
async fn demonstrate_request_response() -> Result<(), Error>
where
    Request: std::marker::Unpin,
    Response: std::marker::Unpin,
{
    println!("\nDemonstrating request-response protocol:");
    
    // Create a channel pair for request-response communication
    let (client_channel, server_channel) = create_request_response_pair();
    
    // Create client and server channels with their respective protocols
    let client_chan = Chan::<RequestClient<Request, Response>, RequestChannel>::new(client_channel);
    let server_chan = Chan::<RequestServer<Request, Response>, ResponseChannel>::new(server_channel);
    
    // Create a request
    let request = Request {
        query: "What is the meaning of life?".to_string(),
    };
    
    // Spawn the server task in a separate thread
    let server_handle = tokio::spawn(async move {
        run_server(server_chan).await
    });
    
    // Run the client task in the current thread
    run_client(client_chan, request).await?;
    
    // Wait for the server task to complete
    server_handle.await.unwrap()?;
    
    println!("Request-response demonstration completed successfully");
    Ok(())
}

/// Demonstrates using the channel_pair function for request-response protocol
fn demonstrate_request_response_pair() {
    println!("\nDemonstrating channel_pair function for request-response:");
    
    // Create a pair of channels for a request-response protocol using the helper function
    let (client, server) = sez::api::channel_pair::<RequestClient<String, i32>, ()>();
    
    // Verify that the channels have the correct types
    let _: Chan<RequestClient<String, i32>, ()> = client;
    let _: Chan<RequestServer<String, i32>, ()> = server;
    
    println!("Successfully created request-response channel pair with correct types");
}

/// Demonstrates the request-response protocol
#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("=== Request-Response Protocol Example ===\n");
    
    // Demonstrate the request-response protocol
    demonstrate_request_response().await?;
    
    // Demonstrate the request_response_pair helper function
    demonstrate_request_response_pair();
    
    println!("\n=== Example completed successfully ===");
    Ok(())
}