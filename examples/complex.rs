//! # Complex Protocol Example
//!
//! This example demonstrates a complex protocol using the session types library.
//! It showcases multiple features of the library, including:
//!
//! 1. Offering and choosing between protocol branches
//! 2. Sending and receiving different types of data
//! 3. Integration with Tokio for asynchronous execution

use futures_core::future::Future;
use futures_core::task::{Context, Poll};
use sez::chan::Chan;
use sez::error::Error;
use sez::io::{AsyncReceiver, AsyncSender};
use sez::proto::{Choose, End, Protocol, Recv, Send};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Define the protocol types

// Client protocol: choose between Add, Multiply, or Quit
type ClientProtocol = Choose<
    Send<(i32, i32), Recv<i32, End>>, // Add operation
    Choose<
        Send<(i32, i32), Recv<i32, End>>, // Multiply operation
        End // Quit operation
    >
>;

// Server protocol: dual of the client protocol
type ServerProtocol = <ClientProtocol as Protocol>::Dual;

/// A custom IO implementation using Tokio's mpsc channels
struct MathChannel {
    tuple_sender: mpsc::Sender<(i32, i32)>,
    tuple_receiver: Arc<Mutex<mpsc::Receiver<(i32, i32)>>>,
    int_sender: mpsc::Sender<i32>,
    int_receiver: Arc<Mutex<mpsc::Receiver<i32>>>,
    bool_sender: mpsc::Sender<bool>,
    bool_receiver: Arc<Mutex<mpsc::Receiver<bool>>>,
}

// Define futures for MathChannel
struct TupleSendFuture {
    sender: mpsc::Sender<(i32, i32)>,
    value: Option<(i32, i32)>,
}

struct IntSendFuture {
    sender: mpsc::Sender<i32>,
    value: Option<i32>,
}

struct BoolSendFuture {
    sender: mpsc::Sender<bool>,
    value: Option<bool>,
}

struct TupleRecvFuture {
    receiver: Arc<Mutex<mpsc::Receiver<(i32, i32)>>>,
}

struct IntRecvFuture {
    receiver: Arc<Mutex<mpsc::Receiver<i32>>>,
}

struct BoolRecvFuture {
    receiver: Arc<Mutex<mpsc::Receiver<bool>>>,
}

// Implement Future for TupleSendFuture
impl Future for TupleSendFuture {
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

// Implement Future for IntSendFuture
impl Future for IntSendFuture {
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

// Implement Future for BoolSendFuture
impl Future for BoolSendFuture {
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

// Implement Future for TupleRecvFuture
impl Future for TupleRecvFuture {
    type Output = Result<(i32, i32), Error>;

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

// Implement Future for IntRecvFuture
impl Future for IntRecvFuture {
    type Output = Result<i32, Error>;

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

// Implement Future for BoolRecvFuture
impl Future for BoolRecvFuture {
    type Output = Result<bool, Error>;

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

// Implement AsyncSender and AsyncReceiver traits for MathChannel
impl AsyncSender<(i32, i32)> for MathChannel {
    type Error = Error;
    type SendFuture<'a> = TupleSendFuture where Self: 'a;

    fn send(&mut self, value: (i32, i32)) -> Self::SendFuture<'_> {
        TupleSendFuture {
            sender: self.tuple_sender.clone(),
            value: Some(value),
        }
    }
}

impl AsyncReceiver<(i32, i32)> for MathChannel {
    type Error = Error;
    type RecvFuture<'a> = TupleRecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        TupleRecvFuture {
            receiver: self.tuple_receiver.clone(),
        }
    }
}

impl AsyncSender<i32> for MathChannel {
    type Error = Error;
    type SendFuture<'a> = IntSendFuture where Self: 'a;

    fn send(&mut self, value: i32) -> Self::SendFuture<'_> {
        IntSendFuture {
            sender: self.int_sender.clone(),
            value: Some(value),
        }
    }
}

impl AsyncReceiver<i32> for MathChannel {
    type Error = Error;
    type RecvFuture<'a> = IntRecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        IntRecvFuture {
            receiver: self.int_receiver.clone(),
        }
    }
}

impl AsyncSender<bool> for MathChannel {
    type Error = Error;
    type SendFuture<'a> = BoolSendFuture where Self: 'a;

    fn send(&mut self, value: bool) -> Self::SendFuture<'_> {
        BoolSendFuture {
            sender: self.bool_sender.clone(),
            value: Some(value),
        }
    }
}

impl AsyncReceiver<bool> for MathChannel {
    type Error = Error;
    type RecvFuture<'a> = BoolRecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        BoolRecvFuture {
            receiver: self.bool_receiver.clone(),
        }
    }
}

/// Creates a pair of MathChannel instances for bidirectional communication
fn create_channel_pair() -> (MathChannel, MathChannel) {
    // Create channels for client -> server and server -> client communication
    let (client_tuple_tx, server_tuple_rx) = mpsc::channel(10);
    let (server_int_tx, client_int_rx) = mpsc::channel(10);
    let (client_bool_tx, server_bool_rx) = mpsc::channel(10);
    let (server_bool_tx, client_bool_rx) = mpsc::channel(10);
    
    let client_channel = MathChannel {
        tuple_sender: client_tuple_tx,
        tuple_receiver: Arc::new(Mutex::new(mpsc::channel(1).1)), // Dummy
        int_sender: mpsc::channel(1).0, // Dummy
        int_receiver: Arc::new(Mutex::new(client_int_rx)),
        bool_sender: client_bool_tx,
        bool_receiver: Arc::new(Mutex::new(client_bool_rx)),
    };
    
    let server_channel = MathChannel {
        tuple_sender: mpsc::channel(1).0, // Dummy
        tuple_receiver: Arc::new(Mutex::new(server_tuple_rx)),
        int_sender: server_int_tx,
        int_receiver: Arc::new(Mutex::new(mpsc::channel(1).1)), // Dummy
        bool_sender: server_bool_tx,
        bool_receiver: Arc::new(Mutex::new(server_bool_rx)),
    };
    
    (client_channel, server_channel)
}

/// Implements the client side of the protocol
async fn run_client(chan: Chan<ClientProtocol, MathChannel>) -> Result<(), Error> {
    println!("Client: Starting protocol");
    
    // First operation: Add 5 + 3
    println!("Client: Choosing Add operation");
    let chan = chan.choose_left().await?;
    
    println!("Client: Sending numbers (5, 3)");
    let chan = chan.send((5, 3)).await?;
    println!("Client: Numbers sent successfully");
    
    println!("Client: Waiting for result");
    let (result, chan) = chan.recv().await?;
    println!("Client: Received result: {}", result);
    
    // Close the channel
    chan.close()?;
    println!("Client: Protocol completed successfully");
    
    Ok(())
}

/// Implements the server side of the protocol
async fn run_server(chan: Chan<ServerProtocol, MathChannel>) -> Result<(), Error> {
    println!("Server: Starting protocol");
    
    println!("Server: Waiting for client choice");
    
    // Offer different operations to the client
    chan.offer(
        // Add operation
        |chan| Ok(async move {
            println!("Server: Client chose Add operation");
            
            // Receive the numbers
            println!("Server: Waiting for numbers");
            let ((a, b), chan) = chan.recv().await?;
            println!("Server: Received numbers: {} and {}", a, b);
            
            // Calculate the sum
            let sum = a + b;
            
            // Send the result
            println!("Server: Sending result: {}", sum);
            let chan = chan.send(sum).await?;
            println!("Server: Result sent successfully");
            
            // Close the channel
            chan.close()?;
            println!("Server: Protocol completed successfully");
            
            Ok(())
        }),
        |chan| Ok(async move {
            // Offer other operations
            chan.offer(
                // Multiply operation
                |chan| Ok(async move {
                    println!("Server: Client chose Multiply operation");
                    
                    // Receive the numbers
                    println!("Server: Waiting for numbers");
                    let ((a, b), chan) = chan.recv().await?;
                    println!("Server: Received numbers: {} and {}", a, b);
                    
                    // Calculate the product
                    let product = a * b;
                    
                    // Send the result
                    println!("Server: Sending result: {}", product);
                    let chan = chan.send(product).await?;
                    println!("Server: Result sent successfully");
                    
                    // Close the channel
                    chan.close()?;
                    println!("Server: Protocol completed successfully");
                    
                    Ok(())
                }),
                // Quit operation
                |chan| Ok(async move {
                    println!("Server: Client chose Quit operation");
                    println!("Server: Closing connection");
                    chan.close()?;
                    println!("Server: Protocol completed successfully");
                    
                    Ok(())
                })
            ).await
        })
    ).await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("=== Complex Protocol Example ===\n");
    
    // Create a channel pair
    let (client_channel, server_channel) = create_channel_pair();
    
    // Create client and server channels with their respective protocols
    let client_chan = Chan::<ClientProtocol, _>::new(client_channel);
    let server_chan = Chan::<ServerProtocol, _>::new(server_channel);
    
    // Spawn the server in a separate task
    let server_handle = tokio::spawn(async move {
        match run_server(server_chan).await {
            Ok(()) => println!("Server completed successfully"),
            Err(e) => println!("Server error: {:?}", e),
        }
    });
    
    // Run the client in the main task
    match run_client(client_chan).await {
        Ok(()) => println!("Client completed successfully"),
        Err(e) => println!("Client error: {:?}", e),
    }
    
    // Wait for the server to complete
    server_handle.await.unwrap();
    
    println!("\n=== Example completed successfully ===");
    Ok(())
}