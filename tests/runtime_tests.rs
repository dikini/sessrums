use sessrums::chan::Chan;
use sessrums::proto::{Protocol, Role, End};
use sessrums::proto::global::{GlobalProtocol, GSend, GRecv, GChoice, GOffer, GRec, GVar, GEnd};
use sessrums::proto::projection::Project;
use std::marker::PhantomData;
use std::sync::mpsc;

// Define roles for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Server;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Logger;

impl Role for Client {
    fn name(&self) -> &'static str {
        "Client"
    }
}

impl Role for Server {
    fn name(&self) -> &'static str {
        "Server"
    }
}

impl Role for Logger {
    fn name(&self) -> &'static str {
        "Logger"
    }
}

// Mock IO implementation for testing
struct MockIO<T> {
    value: Option<T>,
}

impl<T> MockIO<T> {
    fn new(value: Option<T>) -> Self {
        MockIO { value }
    }
}

// Implement AsyncSender for MockIO
impl<T: Clone + std::marker::Unpin> sessrums::io::AsyncSender<T> for MockIO<T> {
    type Error = std::io::Error;
    type SendFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + 'a>> where T: 'a, Self: 'a;

    fn send(&mut self, value: T) -> Self::SendFuture<'_> {
        self.value = Some(value);
        Box::pin(async { Ok(()) })
    }
}

// Implement AsyncReceiver for MockIO
impl<T: Clone + std::marker::Unpin> sessrums::io::AsyncReceiver<T> for MockIO<T> {
    type Error = std::io::Error;
    type RecvFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, Self::Error>> + 'a>> where T: 'a, Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        let value = self.value.clone();
        Box::pin(async move {
            match value {
                Some(v) => Ok(v),
                None => Err(std::io::Error::new(std::io::ErrorKind::Other, "No value available")),
            }
        })
    }
}

#[tokio::test]
async fn test_mpst_simple_protocol() {
    // Define a global protocol: Client sends a String to Server, then ends
    type GlobalProtocol = GSend<String, Client, Server, GEnd>;
    
    // Project the global protocol to local protocols for each role
    type ClientProtocol = <GlobalProtocol as Project<Client>>::LocalProtocol;
    type ServerProtocol = <GlobalProtocol as Project<Server>>::LocalProtocol;
    
    // Create channels with roles
    let client_io = MockIO::new(Some("Hello".to_string()));
    let server_io = MockIO::new(None::<String>);
    
    let client_chan = Chan::<ClientProtocol, Client, _>::new(client_io);
    let server_chan = Chan::<ServerProtocol, Server, _>::new(server_io);
    
    // Test protocol execution
    let client_chan = client_chan.send("Hello".to_string()).await.unwrap();
    let (message, server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(message, "Hello");
    
    // Close the channels
    client_chan.close().unwrap();
    server_chan.close().unwrap();
}

#[tokio::test]
async fn test_mpst_complex_protocol() {
    // Define a global protocol: Client sends a String to Server, Server sends an i32 to Client, then ends
    type GlobalProtocol = GSend<String, Client, Server, GSend<i32, Server, Client, GEnd>>;
    
    // Project the global protocol to local protocols for each role
    type ClientProtocol = <GlobalProtocol as Project<Client>>::LocalProtocol;
    type ServerProtocol = <GlobalProtocol as Project<Server>>::LocalProtocol;
    
    // Create channels with roles
    let client_io = MockIO::new(Some("Hello".to_string()));
    let server_io = MockIO::new(Some(42));
    
    let client_chan = Chan::<ClientProtocol, Client, _>::new(client_io);
    let server_chan = Chan::<ServerProtocol, Server, _>::new(server_io);
    
    // Test protocol execution
    let client_chan = client_chan.send("Hello".to_string()).await.unwrap();
    let (message, server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(message, "Hello");
    
    let server_chan = server_chan.send(42).await.unwrap();
    let (response, client_chan) = client_chan.recv().await.unwrap();
    assert_eq!(response, 42);
    
    // Close the channels
    client_chan.close().unwrap();
    server_chan.close().unwrap();
}

#[tokio::test]
async fn test_mpst_three_roles() {
    // Define a global protocol: Client sends a String to Server, Server sends an i32 to Logger, then ends
    type GlobalProtocol = GSend<String, Client, Server, GSend<i32, Server, Logger, GEnd>>;
    
    // Project the global protocol to local protocols for each role
    type ClientProtocol = <GlobalProtocol as Project<Client>>::LocalProtocol;
    type ServerProtocol = <GlobalProtocol as Project<Server>>::LocalProtocol;
    type LoggerProtocol = <GlobalProtocol as Project<Logger>>::LocalProtocol;
    
    // Create channels with roles
    let client_io = MockIO::new(Some("Hello".to_string()));
    let server_io = MockIO::new(Some(42));
    let logger_io = MockIO::new(None::<i32>);
    
    let client_chan = Chan::<ClientProtocol, Client, _>::new(client_io);
    let server_chan = Chan::<ServerProtocol, Server, _>::new(server_io);
    let logger_chan = Chan::<LoggerProtocol, Logger, _>::new(logger_io);
    
    // Test protocol execution
    let client_chan = client_chan.send("Hello".to_string()).await.unwrap();
    let (message, server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(message, "Hello");
    
    let server_chan = server_chan.send(42).await.unwrap();
    let (log_value, logger_chan) = logger_chan.recv().await.unwrap();
    assert_eq!(log_value, 42);
    
    // Close the channels
    client_chan.close().unwrap();
    server_chan.close().unwrap();
    logger_chan.close().unwrap();
}

// Implement AsyncSender for bool (for choice/offer)
impl sessrums::io::AsyncSender<bool> for MockIO<bool> {
    type Error = std::io::Error;
    type SendFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + 'a>> where bool: 'a, Self: 'a;

    fn send(&mut self, value: bool) -> Self::SendFuture<'_> {
        self.value = Some(value);
        Box::pin(async { Ok(()) })
    }
}

// Implement AsyncReceiver for bool (for choice/offer)
impl sessrums::io::AsyncReceiver<bool> for MockIO<bool> {
    type Error = std::io::Error;
    type RecvFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, Self::Error>> + 'a>> where bool: 'a, Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        let value = self.value.clone();
        Box::pin(async move {
            match value {
                Some(v) => Ok(v),
                None => Err(std::io::Error::new(std::io::ErrorKind::Other, "No value available")),
            }
        })
    }
}

#[tokio::test]
async fn test_mpst_choice() {
    // Define a global protocol with choice
    type GlobalProtocol = GSend<String, Client, Server, 
        GChoice<Server, (
            GSend<i32, Server, Client, GEnd>,
            GSend<bool, Server, Client, GEnd>
        )>
    >;
    
    // Project the global protocol to local protocols for each role
    type ClientProtocol = <GlobalProtocol as Project<Client>>::LocalProtocol;
    type ServerProtocol = <GlobalProtocol as Project<Server>>::LocalProtocol;
    
    // Create channels with roles
    let client_io = MockIO::new(Some("Hello".to_string()));
    let server_io = MockIO::new(Some(42));
    let server_choice_io = MockIO::new(Some(true));
    
    let client_chan = Chan::<ClientProtocol, Client, _>::new(client_io);
    let server_chan = Chan::<ServerProtocol, Server, _>::new(server_io);
    
    // Test protocol execution
    let client_chan = client_chan.send("Hello".to_string()).await.unwrap();
    let (message, server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(message, "Hello");
    
    // Server chooses the left branch (i32)
    let server_chan = server_chan.choose_left().await.unwrap();
    
    // Client offers a choice
    let client_chan = client_chan.offer(
        // Left branch handler (i32)
        |chan| async move {
            let (value, chan) = chan.recv().await.unwrap();
            assert_eq!(value, 42);
            Ok(chan)
        },
        // Right branch handler (bool)
        |chan| async move {
            let (value, chan) = chan.recv().await.unwrap();
            assert_eq!(value, true);
            Ok(chan)
        }
    ).await.unwrap();
    
    // Close the channels
    client_chan.close().unwrap();
    server_chan.close().unwrap();
}