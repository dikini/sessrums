//! # Send Trait Implementation Example (Simple)
//!
//! This example demonstrates how the futures in the session types library
//! implement the Send trait, allowing them to be used across thread boundaries.
//! This is crucial for asynchronous programming, especially when using
//! runtimes like Tokio or async-std that may move futures between threads.

use futures_core::future::Future;
use futures_core::task::{Context, Poll};
use sez::chan::Chan;
use sez::error::Error;
use sez::io::{AsyncReceiver, AsyncSender};
use sez::proto::{End, Protocol, Recv, Send as ProtoSend};
use std::marker::Send as MarkerSend;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Define a simple protocol for demonstration
type PingProtocol = ProtoSend<String, Recv<String, End>>;
type PongProtocol = <PingProtocol as Protocol>::Dual;

// Bidirectional channel that implements both AsyncSender and AsyncReceiver
struct BiChannel {
    sender: mpsc::Sender<String>,
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
}

// Future returned by BiChannel::send
struct SendFuture {
    sender: mpsc::Sender<String>,
    value: Option<String>,
}

// Future returned by BiChannel::recv
struct RecvFuture {
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
}

// Implement Future for SendFuture
impl Future for SendFuture {
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
impl Future for RecvFuture {
    type Output = Result<String, Error>;

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
impl AsyncSender<String> for BiChannel {
    type Error = Error;
    type SendFuture<'a> = SendFuture where Self: 'a;

    fn send(&mut self, value: String) -> Self::SendFuture<'_> {
        SendFuture {
            sender: self.sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver for BiChannel
impl AsyncReceiver<String> for BiChannel {
    type Error = Error;
    type RecvFuture<'a> = RecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        RecvFuture {
            receiver: self.receiver.clone(),
        }
    }
}

/// Creates a pair of BiChannel instances for bidirectional communication
fn create_channel_pair() -> (BiChannel, BiChannel) {
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

/// Implements the ping side of the protocol
async fn run_ping(chan: Chan<PingProtocol, BiChannel>) -> Result<(), Error> {
    println!("Ping: Starting protocol");
    
    // Send a ping message
    println!("Ping: Sending message");
    let message = "ping".to_string();
    let chan = chan.send(message).await?;
    println!("Ping: Message sent successfully");
    
    // Receive a pong message
    println!("Ping: Waiting for response");
    let (response, chan) = chan.recv().await?;
    println!("Ping: Received response: {}", response);
    
    // Close the channel
    chan.close()?;
    println!("Ping: Protocol completed successfully");
    
    Ok(())
}

/// Implements the pong side of the protocol
async fn run_pong(chan: Chan<PongProtocol, BiChannel>) -> Result<(), Error> {
    println!("Pong: Starting protocol");
    
    // Receive a ping message
    println!("Pong: Waiting for message");
    let (message, chan) = chan.recv().await?;
    println!("Pong: Received message: {}", message);
    
    // Send a pong message
    println!("Pong: Sending response");
    let response = "pong".to_string();
    let chan = chan.send(response).await?;
    println!("Pong: Response sent successfully");
    
    // Close the channel
    chan.close()?;
    println!("Pong: Protocol completed successfully");
    
    Ok(())
}

/// Demonstrates using Send trait with Tokio
async fn demonstrate_tokio_send() -> Result<(), Error> {
    println!("\nDemonstrating Send trait with Tokio:");
    
    // Create a channel pair
    let (client_channel, server_channel) = create_channel_pair();
    
    // Create ping and pong channels with their respective protocols
    let ping_chan = Chan::<PingProtocol, _>::new(client_channel);
    let pong_chan = Chan::<PongProtocol, _>::new(server_channel);
    
    // Spawn the pong task in a separate thread
    // This requires the futures to implement Send
    let pong_handle = tokio::spawn(async move {
        run_pong(pong_chan).await
    });
    
    // Run the ping task in the current thread
    run_ping(ping_chan).await?;
    
    // Wait for the pong task to complete
    pong_handle.await.unwrap()?;
    
    println!("Tokio Send trait demonstration completed successfully");
    Ok(())
}

/// Demonstrates using Send trait with async-std
async fn demonstrate_async_std_send() -> Result<(), Error> {
    println!("\nDemonstrating Send trait with async-std:");
    
    // Create a channel pair
    let (client_channel, server_channel) = create_channel_pair();
    
    // Create ping and pong channels with their respective protocols
    let ping_chan = Chan::<PingProtocol, _>::new(client_channel);
    let pong_chan = Chan::<PongProtocol, _>::new(server_channel);
    
    // Spawn the pong task in a separate thread
    // This requires the futures to implement Send
    let pong_handle = async_std::task::spawn(async move {
        run_pong(pong_chan).await
    });
    
    // Run the ping task in the current thread
    run_ping(ping_chan).await?;
    
    // Wait for the pong task to complete
    pong_handle.await?;
    
    println!("async-std Send trait demonstration completed successfully");
    Ok(())
}

/// Demonstrates that our futures implement Send by using them with both Tokio and async-std
#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("=== Send Trait Implementation Example (Simple) ===\n");
    
    // Demonstrate Send trait with Tokio
    demonstrate_tokio_send().await?;
    
    // Demonstrate Send trait with async-std
    demonstrate_async_std_send().await?;
    
    println!("\n=== Example completed successfully ===");
    Ok(())
}