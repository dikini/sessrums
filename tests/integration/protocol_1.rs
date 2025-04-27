//! Protocol 1: Simple Send/Recv Ping-Pong
//!
//! # Protocol Description
//!
//! This example demonstrates a simple ping-pong protocol where:
//! - Client sends an i32 value
//! - Server receives the i32 value
//! - Server sends a String response
//! - Client receives the String response
//! - Both sides close the connection
//!
//! # Session Type Safety
//!
//! This protocol demonstrates several key aspects of session type safety:
//!
//! 1. **Type-level Protocol Definition**: The protocol is defined at the type level using
//!    `Send<T, P>`, `Recv<T, P>`, and `End` types, ensuring that the communication
//!    sequence is enforced by the Rust type system.
//!
//! 2. **Duality**: The client and server protocols are duals of each other, ensuring
//!    that they can communicate without deadlocks or protocol violations. When the client
//!    sends, the server receives; when the server sends, the client receives.
//!
//! 3. **Type Safety**: The protocol ensures that the correct types are sent and received
//!    at each step. The client must send an i32, and the server must respond with a String.
//!
//! 4. **Protocol Completion**: Both sides must follow the protocol to completion, ending
//!    with the `End` type, which ensures that no communication is left hanging.
//!
//! # Visual Diagram
//!
//! ```text
//!                   PingPongClient                 PingPongServer
//!                   --------------                 --------------
//!                         |                              |
//!                         |        Send(i32)            |
//!                         | ---------------------------> |
//!                         |                              |
//!                         |        Recv(String)          |
//!                         | <--------------------------- |
//!                         |                              |
//!                         |           End                |
//!                         | - - - - - - - - - - - - - - -|
//!                         |                              |
//! ```
//!
//! # Type-Level Representation
//!
//! ```
//! Client: Send<i32, Recv<String, End>>
//! Server: Recv<i32, Send<String, End>>
//! ```
//!
//! This demonstrates how the types mirror the communication flow and ensure
//! protocol adherence at compile time.

use sessrums::proto::{Send, Recv, End};
use sessrums::chan::Chan;
use futures_core::future::Future;
use std::pin::Pin;
use futures_core::task::{Context, Poll};
use std::marker::PhantomData;

// Import helper functions from the integration test module
use crate::integration::{assert_protocol, assert_dual, mock_channel};

// Define the protocol types
// Client: Send an i32, then receive a String, then end
type PingPongClient = Send<i32, Recv<String, End>>;
// Server: Receive an i32, then send a String, then end
type PingPongServer = Recv<i32, Send<String, End>>;

/// This test verifies the type-level properties of the ping-pong protocol.
///
/// This test demonstrates the runtime behavior of the ping-pong protocol
/// using the send, recv, and close methods.
#[tokio::test]
async fn test_ping_pong_protocol() {
    // Verify that PingPongClient and PingPongServer implement the Protocol trait
    assert_protocol::<PingPongClient>();
    assert_protocol::<PingPongServer>();
    
    // Verify that PingPongServer is the dual of PingPongClient
    assert_dual::<PingPongClient, PingPongServer>();
    
    // Create a pair of channels with the PingPongClient and PingPongServer types
    // We'll use a custom IO implementation for testing
    #[derive(Clone)]
    struct TestIO {
        client_to_server: Option<i32>,
        server_to_client: Option<String>,
    }
    
    // Custom error type
    #[derive(Debug)]
    struct TestError;
    
    // Define futures for async operations
    struct TestSendFuture<T> {
        io: TestIO,
        value: Option<T>,
        is_client_to_server: bool,
    }

    struct TestRecvFuture<T> {
        io: TestIO,
        is_client_to_server: bool,
        _marker: PhantomData<T>,
    }

    // Implement Future for TestSendFuture<i32>
    impl Future for TestSendFuture<i32> {
        type Output = Result<(), TestError>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if let Some(value) = this.value.take() {
                if this.is_client_to_server {
                    this.io.client_to_server = Some(value);
                }
                Poll::Ready(Ok(()))
            } else {
                Poll::Ready(Err(TestError))
            }
        }
    }

    // Implement Future for TestSendFuture<String>
    impl Future for TestSendFuture<String> {
        type Output = Result<(), TestError>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if let Some(value) = this.value.take() {
                if !this.is_client_to_server {
                    this.io.server_to_client = Some(value);
                }
                Poll::Ready(Ok(()))
            } else {
                Poll::Ready(Err(TestError))
            }
        }
    }

    // Implement Future for TestRecvFuture<i32>
    impl Future for TestRecvFuture<i32> {
        type Output = Result<i32, TestError>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if this.is_client_to_server {
                match this.io.client_to_server.take() {
                    Some(value) => Poll::Ready(Ok(value)),
                    None => Poll::Ready(Err(TestError)),
                }
            } else {
                Poll::Ready(Err(TestError))
            }
        }
    }

    // Implement Future for TestRecvFuture<String>
    impl Future for TestRecvFuture<String> {
        type Output = Result<String, TestError>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if !this.is_client_to_server {
                match this.io.server_to_client.take() {
                    Some(value) => Poll::Ready(Ok(value)),
                    None => Poll::Ready(Err(TestError)),
                }
            } else {
                Poll::Ready(Err(TestError))
            }
        }
    }

    // Implement AsyncSender<i32> for TestIO (client sending to server)
    impl sessrums::io::AsyncSender<i32> for TestIO {
        type Error = TestError;
        type SendFuture<'a> = TestSendFuture<i32> where Self: 'a;

        fn send(&mut self, value: i32) -> Self::SendFuture<'_> {
            TestSendFuture {
                io: TestIO {
                    client_to_server: self.client_to_server,
                    server_to_client: self.server_to_client.clone(),
                },
                value: Some(value),
                is_client_to_server: true,
            }
        }
    }
    
    // Implement AsyncReceiver<i32> for TestIO (server receiving from client)
    impl sessrums::io::AsyncReceiver<i32> for TestIO {
        type Error = TestError;
        type RecvFuture<'a> = TestRecvFuture<i32> where Self: 'a;
        
        fn recv(&mut self) -> Self::RecvFuture<'_> {
            TestRecvFuture {
                io: TestIO {
                    client_to_server: self.client_to_server,
                    server_to_client: self.server_to_client.clone(),
                },
                is_client_to_server: true,
                _marker: PhantomData,
            }
        }
    }
    
    // Implement AsyncSender<String> for TestIO (server sending to client)
    impl sessrums::io::AsyncSender<String> for TestIO {
        type Error = TestError;
        type SendFuture<'a> = TestSendFuture<String> where Self: 'a;
        
        fn send(&mut self, value: String) -> Self::SendFuture<'_> {
            TestSendFuture {
                io: TestIO {
                    client_to_server: self.client_to_server,
                    server_to_client: self.server_to_client.clone(),
                },
                value: Some(value),
                is_client_to_server: false,
            }
        }
    }
    
    // Implement AsyncReceiver<String> for TestIO (client receiving from server)
    impl sessrums::io::AsyncReceiver<String> for TestIO {
        type Error = TestError;
        type RecvFuture<'a> = TestRecvFuture<String> where Self: 'a;
        
        fn recv(&mut self) -> Self::RecvFuture<'_> {
            TestRecvFuture {
                io: TestIO {
                    client_to_server: self.client_to_server,
                    server_to_client: self.server_to_client.clone(),
                },
                is_client_to_server: false,
                _marker: PhantomData,
            }
        }
    }
    
    // Create the IO implementation
    let io = TestIO {
        client_to_server: None,
        server_to_client: None,
    };
    
    // Create client and server channels
    let client_chan = Chan::<PingPongClient, _>::new(io);
    
    // 1. Client sends an i32 value (42)
    let _client_chan = client_chan.send(42).await.unwrap();
    
    // Create a new IO implementation for the server
    // In a real implementation, we would use a proper channel mechanism
    let io = TestIO {
        client_to_server: Some(42), // Simulate the client's message
        server_to_client: None,
    };
    
    // Create the server channel
    let server_chan = Chan::<PingPongServer, _>::new(io);
    
    // 2. Server receives the i32 value
    let (value, server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(value, 42);
    
    // 3. Server sends a String response
    let server_chan = server_chan.send("Hello, client!".to_string()).await.unwrap();
    
    // Create a new IO implementation for the client to receive the response
    let io = TestIO {
        client_to_server: None,
        server_to_client: Some("Hello, client!".to_string()), // Simulate the server's response
    };
    
    // Update the client channel
    let client_chan = Chan::<Recv<String, End>, _>::new(io);
    
    // 4. Client receives the String response
    let (response, client_chan) = client_chan.recv().await.unwrap();
    assert_eq!(response, "Hello, client!");
    
    // 5. Both sides close the connection
    client_chan.close().unwrap();
    server_chan.close().unwrap();
}

/// This test demonstrates how the type system prevents protocol violations.
///
/// If we were to try to use the wrong protocol type for a channel, the code
/// would fail to compile, demonstrating the type safety provided by session types.
#[test]
fn test_ping_pong_type_safety() {
    // The following line would fail to compile if uncommented because
    // we're trying to use a server protocol for a client channel:
    //
    // let _invalid_chan: Chan<PingPongClient, ()> = mock_channel::<PingPongServer, ()>();
    //
    // Similarly, trying to send a String when the protocol expects an i32
    // would fail to compile in the actual implementation.
    
    // Instead, we verify that the correct types work
    let _client_chan: Chan<PingPongClient, ()> = mock_channel::<PingPongClient, ()>();
    let _server_chan: Chan<PingPongServer, ()> = mock_channel::<PingPongServer, ()>();
}