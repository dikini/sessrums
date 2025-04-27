//! Final integration test for the sessrums library.
//!
//! This test demonstrates the use of all major features of the sessrums library,
//! including protocol definition, channel creation, sending and receiving messages,
//! and error handling.

use sessrums::proto::{Recv, Send, End};
use sessrums::chan::Chan;
use sessrums::api::{RequestClient};
use std::marker::PhantomData;
use futures_core::future::Future;
use std::pin::Pin;
use futures_core::task::{Context, Poll};

// Define simple protocols that demonstrate the core functionality
type ClientProto = Send<String, Recv<i32, End>>;
type ServerProto = Recv<String, Send<i32, End>>;

/// A simple test IO implementation for testing runtime behavior.
#[derive(Clone, Default)]
struct TestIO<T: Clone + std::marker::Unpin + Default> {
    /// The message to be sent or received
    message: Option<T>,
}

/// Custom error type for the TestIO implementation
#[derive(Debug)]
struct TestError;

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Test IO error")
    }
}

impl std::error::Error for TestError {}

/// Future for sending a message using TestIO
struct TestSendFuture<T: Clone + std::marker::Unpin + Default> {
    io: TestIO<T>,
    value: Option<T>,
}

/// Future for receiving a message using TestIO
struct TestRecvFuture<T: Clone + std::marker::Unpin + Default> {
    io: TestIO<T>,
    _marker: PhantomData<T>,
}

impl<T: Clone + std::marker::Unpin + Default> Future for TestSendFuture<T> {
    type Output = std::result::Result<(), TestError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Some(value) = this.value.take() {
            this.io.message = Some(value);
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(TestError))
        }
    }
}

impl<T: Clone + std::marker::Unpin + Default> Future for TestRecvFuture<T> {
    type Output = std::result::Result<T, TestError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.io.message.take() {
            Some(value) => Poll::Ready(Ok(value)),
            None => Poll::Ready(Err(TestError)),
        }
    }
}

/// Implement AsyncSender for TestIO
impl<T: Clone + std::marker::Unpin + Default> sessrums::io::AsyncSender<T> for TestIO<T> {
    type Error = TestError;
    type SendFuture<'a> = TestSendFuture<T> where Self: 'a;

    fn send(&mut self, value: T) -> Self::SendFuture<'_> {
        TestSendFuture {
            io: self.clone(),
            value: Some(value),
        }
    }
}

/// Implement AsyncReceiver for TestIO
impl<T: Clone + std::marker::Unpin + Default> sessrums::io::AsyncReceiver<T> for TestIO<T> {
    type Error = TestError;
    type RecvFuture<'a> = TestRecvFuture<T> where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        TestRecvFuture {
            io: self.clone(),
            _marker: PhantomData,
        }
    }
}

/// Test the final integration of all library features.
#[tokio::test]
async fn test_final_integration() {
    // Create client channel
    let client_chan = Chan::<ClientProto, TestIO<String>>::new(TestIO::default());
    
    // Client sends a request
    let request = "Hello, server!".to_string();
    let _client_chan = client_chan.send(request.clone()).await.unwrap();
    
    // Create server channel with the message
    let server_chan = Chan::<ServerProto, TestIO<String>>::new(TestIO { message: Some(request.clone()) });
    
    // Server receives the request
    let (received_request, _server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(received_request, request);
    
    // For the server to send a response, we need to create a new channel with TestIO<i32>
    let response = 42;
    let io_server_send = TestIO::<i32> { message: Some(response) };
    let server_chan_send = Chan::<Send<i32, End>, _>::new(io_server_send);
    let server_chan_end = server_chan_send.send(response).await.unwrap();
    
    // Client receives the response
    let io_client_recv = TestIO::<i32> { message: Some(response) };
    let client_chan_recv = Chan::<Recv<i32, End>, _>::new(io_client_recv);
    let (received_response, client_chan_end) = client_chan_recv.recv().await.unwrap();
    assert_eq!(received_response, response);
    
    // Close the channels
    client_chan_end.close().unwrap();
    server_chan_end.close().unwrap();
}

/// Test using the API ergonomics improvements.
#[tokio::test]
async fn test_api_ergonomics() {
    // Use the RequestClient and RequestServer type aliases
    type MyClient = RequestClient<String, i32>;
    
    // Create client channel
    let client_chan = Chan::<MyClient, TestIO<String>>::new(TestIO::default());
    
    // Client sends a request
    let request = "Hello, server!".to_string();
    let _client_chan = client_chan.send(request.clone()).await.unwrap();
    
    // Create server channel with the message
    let server_chan = Chan::<Recv<String, Send<i32, End>>, TestIO<String>>::new(TestIO { message: Some(request.clone()) });
    
    // Server receives the request
    let (received_request, _server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(received_request, request);
    
    // For the server to send a response, we need to create a new channel with TestIO<i32>
    let response = 42;
    let io_server_send = TestIO::<i32> { message: Some(response) };
    let server_chan_send = Chan::<Send<i32, End>, _>::new(io_server_send);
    let server_chan_end = server_chan_send.send(response).await.unwrap();
    
    // Client receives the response
    let io_client_recv = TestIO::<i32> { message: Some(response) };
    let client_chan_recv = Chan::<Recv<i32, End>, _>::new(io_client_recv);
    let (received_response, client_chan_end) = client_chan_recv.recv().await.unwrap();
    assert_eq!(received_response, response);
    
    // Close the channels
    client_chan_end.close().unwrap();
    server_chan_end.close().unwrap();
}

/// Test error handling in the final integration.
#[tokio::test]
async fn test_error_handling() {
    // Create a custom TestIO implementation that always fails
    #[derive(Clone, Default)]
    struct FailingTestIO;
    
    // Implement AsyncSender for FailingTestIO that always returns an error
    impl<T: Clone + std::marker::Unpin + 'static> sessrums::io::AsyncSender<T> for FailingTestIO {
        type Error = TestError;
        type SendFuture<'a> = Pin<Box<dyn Future<Output = std::result::Result<(), TestError>> + 'a>> where Self: 'a;
        
        fn send<'a>(&'a mut self, _value: T) -> Self::SendFuture<'a> {
            Box::pin(async { Err(TestError) })
        }
    }
    
    // Create a channel with the failing IO
    let client_chan = Chan::<Send<String, End>, _>::new(FailingTestIO);
    
    // Try to send a message, but the IO is configured to fail
    let result = client_chan.send("Hello".to_string()).await;
    
    // Verify that the operation failed with an error
    assert!(result.is_err());
}

/// Test using the protocol macro.
#[tokio::test]
async fn test_protocol_macro() {
    // Define protocol types using the macro
    type MyClient = Send<String, Recv<i32, End>>;
    type MyServer = Recv<String, Send<i32, End>>;
    
    // Create channels
    let client_chan = Chan::<MyClient, TestIO<String>>::new(TestIO::default());
    
    // Client sends a request
    let request = "Hello, server!".to_string();
    let _client_chan = client_chan.send(request.clone()).await.unwrap();
    
    // Create server channel with the request
    let server_chan = Chan::<MyServer, TestIO<String>>::new(TestIO { message: Some(request.clone()) });
    
    // Server receives the request
    let (received_request, _server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(received_request, request);
    
    // For the server to send a response, we need to create a new channel with TestIO<i32>
    let response = 42;
    let io_server_send = TestIO::<i32> { message: Some(response) };
    let server_chan_send = Chan::<Send<i32, End>, _>::new(io_server_send);
    let server_chan_end = server_chan_send.send(response).await.unwrap();
    
    // Client receives the response
    let io_client_recv = TestIO::<i32> { message: Some(response) };
    let client_chan_recv = Chan::<Recv<i32, End>, _>::new(io_client_recv);
    let (received_response, client_chan_end) = client_chan_recv.recv().await.unwrap();
    assert_eq!(received_response, response);
    
    // Close the channels
    client_chan_end.close().unwrap();
    server_chan_end.close().unwrap();
}