//! # Simple Protocol Example
//!
//! This example demonstrates a simple client-server interaction using session types.
//! It shows how to define protocols, create channels, and use the `send`, `recv`,
//! and `close` methods to communicate according to the protocol.
//!
//! ## Protocol Description
//!
//! The protocol implemented here is a simple query-response pattern:
//!
//! ```text
//! Client                                Server
//!   |                                     |
//!   |------- Send(String) - Query ------->|
//!   |                                     |
//!   |<------ Recv(String) - Response -----|
//!   |                                     |
//!   |-------------- End ---------------->|
//! ```
//!
//! This is a common pattern in client-server interactions where:
//! 1. The client sends a query (String)
//! 2. The server responds with an answer (String)
//! 3. The communication ends

use sessrums::chan::Chan;
use sessrums::proto::{End, Protocol, Recv, Send};
use sessrums::io::{AsyncSender, AsyncReceiver};
use sessrums::error::Error;
use std::sync::mpsc;
use std::thread;
use futures_core::future::Future;
use std::pin::Pin;
use futures_core::task::{Context, Poll};
use std::marker::PhantomData;

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
impl<T: Default + Unpin> AsyncSender<T> for BiChannel<T> {
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
impl<T: Unpin> AsyncReceiver<T> for BiChannel<T> {
    type Error = mpsc::RecvError;
    type RecvFuture<'a> = BiChannelRecvFuture<T> where T: 'a, Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        BiChannelRecvFuture {
            receiver: &mut self.receiver as *mut mpsc::Receiver<T>,
            _marker: PhantomData,
        }
    }
}

// Define the client's protocol: Send a query, receive a response, then end
type ClientProtocol = Send<String, Recv<String, End>>;

// Define the server's protocol: Receive a query, send a response, then end
// Note: This is the dual of the client's protocol
type ServerProtocol = <ClientProtocol as Protocol>::Dual;

// Alternatively, we could explicitly define the server protocol as:
// type ServerProtocol = Recv<String, Send<String, End>>;

/// Implements the client side of the protocol
async fn run_client(chan: Chan<ClientProtocol, BiChannel<String>>) -> Result<(), Error> {
    println!("Client: Starting protocol");
    
    // Step 1: Send a query to the server
    println!("Client: Sending query");
    let query = "What is the meaning of life?".to_string();
    let chan = chan.send(query).await?;
    println!("Client: Query sent successfully");
    
    // Step 2: Receive the response from the server
    println!("Client: Waiting for response");
    let (response, chan) = chan.recv().await?;
    println!("Client: Received response: {}", response);
    
    // Step 3: Close the channel (end the protocol)
    println!("Client: Closing channel");
    chan.close()?;
    println!("Client: Protocol completed successfully");
    
    Ok(())
}

/// Implements the server side of the protocol
async fn run_server(chan: Chan<ServerProtocol, BiChannel<String>>) -> Result<(), Error> {
    println!("Server: Starting protocol");
    
    // Step 1: Receive a query from the client
    println!("Server: Waiting for query");
    let (query, chan) = chan.recv().await?;
    println!("Server: Received query: {}", query);
    
    // Step 2: Process the query and send a response
    println!("Server: Processing query");
    let response = process_query(&query);
    println!("Server: Sending response");
    let chan = chan.send(response).await?;
    println!("Server: Response sent successfully");
    
    // Step 3: Close the channel (end the protocol)
    println!("Server: Closing channel");
    chan.close()?;
    println!("Server: Protocol completed successfully");
    
    Ok(())
}

/// Process a query and return a response
fn process_query(query: &str) -> String {
    match query {
        "What is the meaning of life?" => "42".to_string(),
        "How are you?" => "I'm fine, thank you!".to_string(),
        _ => "I don't know the answer to that question.".to_string(),
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

/// Demonstrates error handling with a custom IO implementation that fails on recv
async fn demonstrate_error_handling() -> Result<(), Error> {
    println!("\nDemonstrating error handling:");
    
    // Create a custom IO implementation that fails on recv
    struct FailingIO;
    
    // Define futures for FailingIO
    struct FailingIOSendFuture {
        success: bool,
    }

    struct FailingIORecvFuture {
        success: bool,
    }

    // Implement Future for FailingIOSendFuture
    impl Future for FailingIOSendFuture {
        type Output = Result<(), ()>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.success {
                println!("Send operation succeeded");
                Poll::Ready(Ok(()))
            } else {
                println!("Send operation failed");
                Poll::Ready(Err(()))
            }
        }
    }

    // Implement Future for FailingIORecvFuture
    impl Future for FailingIORecvFuture {
        type Output = Result<String, ()>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.success {
                println!("Receive operation succeeded");
                Poll::Ready(Ok("Success".to_string()))
            } else {
                println!("Receive operation failed");
                Poll::Ready(Err(()))
            }
        }
    }

    // Implement AsyncSender for FailingIO
    impl AsyncSender<String> for FailingIO {
        type Error = ();
        type SendFuture<'a> = FailingIOSendFuture where Self: 'a;

        fn send(&mut self, _value: String) -> Self::SendFuture<'_> {
            FailingIOSendFuture {
                success: true,
            }
        }
    }
    
    // Implement AsyncReceiver for FailingIO
    impl AsyncReceiver<String> for FailingIO {
        type Error = ();
        type RecvFuture<'a> = FailingIORecvFuture where Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            FailingIORecvFuture {
                success: false,
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
    
    // Receive should fail
    println!("Attempting to receive a response (should fail)...");
    match chan.recv().await {
        Ok(_) => println!("Unexpected success!"),
        Err(e) => {
            println!("Received expected error: {}", e);
            return Ok(());
        }
    }
    
    // We should never reach here
    Err(Error::Protocol("Expected an error but got success"))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("=== Simple Protocol Example ===\n");
    
    // Create a channel pair for communication
    let (client_channel, server_channel) = create_channel_pair();
    
    // Create client and server channels with their respective protocols
    let client_chan = Chan::<ClientProtocol, _>::new(client_channel);
    let server_chan = Chan::<ServerProtocol, _>::new(server_channel);
    
    // Run the server in a separate thread
    let server_handle = thread::spawn(|| {
        let server_future = run_server(server_chan);
        
        // Create a runtime for the server thread
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(server_future)
    });
    
    // Run the client in the main thread
    run_client(client_chan).await?;
    
    // Wait for the server to complete
    server_handle.join().unwrap()?;
    
    // Demonstrate error handling
    demonstrate_error_handling().await?;
    
    println!("\n=== Example completed successfully ===");
    Ok(())
}

/// This module demonstrates how the type system ensures protocol adherence
mod type_safety_examples {
    use sessrums::chan::Chan;
    use sessrums::proto::{End, Recv, Send};
    use sessrums::io::{AsyncSender, AsyncReceiver};
    use futures_core::future::Future;
    use std::pin::Pin;
    use futures_core::task::{Context, Poll};
    use std::marker::PhantomData;
    
    // Define a protocol: Send an i32, then receive a String, then end
    type MyProtocol = Send<i32, Recv<String, End>>;
    
    // For demonstration purposes only - this is a simplified example
    // In a real application, you would use a proper bidirectional channel
    struct DummyIO;
    
    // Define futures for DummyIO
    struct DummyIOSendFuture<T> {
        _marker: PhantomData<T>,
    }

    struct DummyIORecvFuture {
        response: String,
    }

    // Implement Future for DummyIOSendFuture<i32>
    impl Future for DummyIOSendFuture<i32> {
        type Output = Result<(), ()>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Ready(Ok(()))
        }
    }

    // Implement Future for DummyIORecvFuture
    impl Future for DummyIORecvFuture {
        type Output = Result<String, ()>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Ready(Ok(self.response.clone()))
        }
    }

    // Implement AsyncSender<i32> for DummyIO
    impl AsyncSender<i32> for DummyIO {
        type Error = ();
        type SendFuture<'a> = DummyIOSendFuture<i32> where Self: 'a;
        
        fn send(&mut self, _value: i32) -> Self::SendFuture<'_> {
            DummyIOSendFuture {
                _marker: PhantomData,
            }
        }
    }
    
    // Implement AsyncReceiver<String> for DummyIO
    impl AsyncReceiver<String> for DummyIO {
        type Error = ();
        type RecvFuture<'a> = DummyIORecvFuture where Self: 'a;
        
        fn recv(&mut self) -> Self::RecvFuture<'_> {
            DummyIORecvFuture {
                response: "dummy response".to_string(),
            }
        }
    }
    
    // The following function compiles because it follows the protocol correctly
    #[allow(dead_code)]
    async fn correct_protocol_usage(chan: Chan<MyProtocol, DummyIO>) {
        // First send an i32 as required by the protocol
        let chan = chan.send(42).await.unwrap();
        
        // Then receive a String as required by the protocol
        let (response, chan) = chan.recv().await.unwrap();
        let _: String = response; // Type check
        
        // Finally close the channel as required by the protocol
        chan.close().unwrap();
    }
    
    // The following function would not compile because it violates the protocol
    // by trying to receive before sending
    /*
    #[allow(dead_code)]
    async fn incorrect_protocol_usage(chan: Chan<MyProtocol, DummyIO>) {
        // Error: The protocol requires sending an i32 first, but we're trying to receive
        let (response, chan) = chan.recv().await.unwrap();
        
        // Send an i32
        let chan = chan.send(42).await.unwrap();
        
        // Close the channel
        chan.close().unwrap();
    }
    */
    
    // The following function would not compile because it violates the protocol
    // by trying to send a String instead of an i32
    /*
    #[allow(dead_code)]
    async fn incorrect_type_usage(chan: Chan<MyProtocol, DummyIO>) {
        // Error: The protocol requires sending an i32, but we're trying to send a String
        let chan = chan.send("hello".to_string()).await.unwrap();
        
        // Receive a String
        let (response, chan) = chan.recv().await.unwrap();
        
        // Close the channel
        chan.close().unwrap();
    }
    */
    
    // The following function would not compile because it violates the protocol
    // by not closing the channel at the end
    /*
    #[allow(dead_code)]
    async fn incomplete_protocol_usage(chan: Chan<MyProtocol, DummyIO>) {
        // Send an i32
        let chan = chan.send(42).await.unwrap();
        
        // Receive a String
        let (response, chan) = chan.recv().await.unwrap();
        
        // Error: The protocol requires closing the channel, but we're not doing it
        // Missing: chan.close().unwrap();
    }
    */
}