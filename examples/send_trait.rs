//! # Send Trait Implementation Example
//!
//! This example demonstrates how the futures in the session types library
//! implement the Send trait, allowing them to be used across thread boundaries.
//! This is crucial for asynchronous programming, especially when using
//! runtimes like Tokio or async-std that may move futures between threads.
//!
//! The example shows:
//! 1. How to create futures that implement Send
//! 2. How to use these futures with Tokio and async-std
//! 3. How to spawn tasks that use session-typed channels
//! 4. How to ensure thread safety with shared state

use futures_core::future::Future;
use futures_core::task::{Context, Poll};
use sessrums::chan::Chan;
use sessrums::error::Error;
use sessrums::io::{AsyncReceiver, AsyncSender};
use sessrums::proto::{End, Protocol, Recv, Send};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Define a simple protocol for demonstration
type PingProtocol = Send<String, Recv<String, End>>;
type PongProtocol = <PingProtocol as Protocol>::Dual;

/// A thread-safe channel implementation using Tokio's mpsc channels
struct ThreadSafeChannel {
    sender: mpsc::Sender<String>,
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
}

// Define futures for ThreadSafeChannel
struct SendFuture {
    sender: mpsc::Sender<String>,
    value: Option<String>,
}

struct RecvFuture {
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
}

// Implement Future for SendFuture
// Note: This implementation must be Send to work across thread boundaries
impl Future for SendFuture {
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

// Implement Future for RecvFuture
// Note: This implementation must be Send to work across thread boundaries
impl Future for RecvFuture {
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

// Implement AsyncSender<String> for ThreadSafeChannel
impl AsyncSender<String> for ThreadSafeChannel {
    type Error = Error;
    // Note: The SendFuture must be Send to work across thread boundaries
    type SendFuture<'a> = SendFuture where Self: 'a;

    fn send(&mut self, value: String) -> Self::SendFuture<'_> {
        SendFuture {
            sender: self.sender.clone(),
            value: Some(value),
        }
    }
}

// Implement AsyncReceiver<String> for ThreadSafeChannel
impl AsyncReceiver<String> for ThreadSafeChannel {
    type Error = Error;
    // Note: The RecvFuture must be Send to work across thread boundaries
    type RecvFuture<'a> = RecvFuture where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        RecvFuture {
            receiver: self.receiver.clone(),
        }
    }
}

/// Creates a pair of ThreadSafeChannel instances for bidirectional communication
fn create_channel_pair() -> (ThreadSafeChannel, ThreadSafeChannel) {
    // Create channels for ping -> pong and pong -> ping communication
    let (ping_tx, pong_rx) = mpsc::channel(10);
    let (pong_tx, ping_rx) = mpsc::channel(10);
    
    let ping_channel = ThreadSafeChannel {
        sender: ping_tx,
        receiver: Arc::new(Mutex::new(ping_rx)),
    };
    
    let pong_channel = ThreadSafeChannel {
        sender: pong_tx,
        receiver: Arc::new(Mutex::new(pong_rx)),
    };
    
    (ping_channel, pong_channel)
}

/// Implements the ping side of the protocol
async fn run_ping(chan: Chan<PingProtocol, ThreadSafeChannel>) -> Result<(), Error> {
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
async fn run_pong(chan: Chan<PongProtocol, ThreadSafeChannel>) -> Result<(), Error> {
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
    let (ping_channel, pong_channel) = create_channel_pair();
    
    // Create ping and pong channels with their respective protocols
    let ping_chan = Chan::<PingProtocol, _>::new(ping_channel);
    let pong_chan = Chan::<PongProtocol, _>::new(pong_channel);
    
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
    let (ping_channel, pong_channel) = create_channel_pair();
    
    // Create ping and pong channels with their respective protocols
    let ping_chan = Chan::<PingProtocol, _>::new(ping_channel);
    let pong_chan = Chan::<PongProtocol, _>::new(pong_channel);
    
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
    println!("=== Send Trait Implementation Example ===\n");
    
    // Demonstrate Send trait with Tokio
    demonstrate_tokio_send().await?;
    
    // Demonstrate Send trait with async-std
    demonstrate_async_std_send().await?;
    
    println!("\n=== Example completed successfully ===");
    Ok(())
}

// This unit test verifies at compile time that our futures implement Send
#[cfg(test)]
mod tests {
    use super::*;
    
    // This function will fail to compile if SendFuture doesn't implement Send
    fn assert_send<T: Send>() {}
    
    #[test]
    fn test_send_future_is_send() {
        assert_send::<SendFuture>();
    }
    
    // This function will fail to compile if RecvFuture doesn't implement Send
    #[test]
    fn test_recv_future_is_send() {
        assert_send::<RecvFuture>();
    }
    
    // This function will fail to compile if the Chan's futures don't implement Send
    #[test]
    fn test_chan_futures_are_send() {
        // We can't directly test the future type from chan.send() in a unit test,
        // but we can verify that our channel implementation's futures are Send
        type SendFut = <ThreadSafeChannel as AsyncSender<String>>::SendFuture<'static>;
        type RecvFut = <ThreadSafeChannel as AsyncReceiver<String>>::RecvFuture<'static>;
        
        assert_send::<SendFut>();
        assert_send::<RecvFut>();
    }
}