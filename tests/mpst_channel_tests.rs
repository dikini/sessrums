use sessrums::chan::Chan;
use sessrums::proto::{Protocol, Send, Recv, End, Role};
use sessrums::proto::roles::{RoleA, RoleB};
use std::marker::PhantomData;

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
async fn test_mpst_channel_with_roles() {
    // Define a simple protocol
    type ClientProtocol = Send<String, Recv<i32, End>>;
    type ServerProtocol = Recv<String, Send<i32, End>>;

    // Create channels with roles
    let client_io = MockIO::new(Some("Hello".to_string()));
    let server_io = MockIO::new(Some(42));

    let mut client_chan = Chan::<ClientProtocol, RoleA, _>::new(client_io);
    let mut server_chan = Chan::<ServerProtocol, RoleB, _>::new(server_io);

    // Test role access
    assert_eq!(client_chan.role().name(), "RoleA");
    assert_eq!(server_chan.role().name(), "RoleB");

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
async fn test_mpst_channel_role_preservation() {
    // Define a protocol with multiple steps
    type MultiStepClient = Send<String, Recv<i32, Send<bool, End>>>;
    type MultiStepServer = Recv<String, Send<i32, Recv<bool, End>>>;

    // Create channels with roles
    let client_io = MockIO::new(Some("Hello".to_string()));
    let server_io = MockIO::new(Some(42));

    let client_chan = Chan::<MultiStepClient, RoleA, _>::new(client_io);
    let server_chan = Chan::<MultiStepServer, RoleB, _>::new(server_io);

    // Execute first step
    let client_chan = client_chan.send("Hello".to_string()).await.unwrap();
    let (message, server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(message, "Hello");

    // Execute second step
    let server_chan = server_chan.send(42).await.unwrap();
    let (response, client_chan) = client_chan.recv().await.unwrap();
    assert_eq!(response, 42);

    // Execute third step and verify roles are preserved
    let client_chan = client_chan.send(true).await.unwrap();
    let (flag, server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(flag, true);

    // Verify roles are preserved throughout the protocol
    assert_eq!(client_chan.role().name(), "RoleA");
    assert_eq!(server_chan.role().name(), "RoleB");

    // Close the channels
    client_chan.close().unwrap();
    server_chan.close().unwrap();
}