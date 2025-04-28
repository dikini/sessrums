use sessrums::chan::Chan;
use sessrums::proto::{Send, Recv, End, Role};
use sessrums::proto::roles::{RoleA, RoleB};
use sessrums::proto::global::{GSend, GEnd};
use sessrums::proto::projection::Project;
use sessrums::proto::compat::MPSTWrapper;

// Define a third role for testing multi-party protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct RoleC;

impl Role for RoleC {
    fn name(&self) -> &'static str {
        "RoleC"
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
    // Define a simple protocol: RoleA sends a String to RoleB, then ends
    
    // Project the global protocol for each role
    type RoleALocal = Send<String, End>;
    type RoleBLocal = Recv<String, End>;
    
    // Create channels with roles
    let client_io = MockIO::new(Some("Hello".to_string()));
    let server_io = MockIO::new(Some("Hello".to_string()));
    
    let client_chan = Chan::<RoleALocal, RoleA, _>::new(client_io);
    let server_chan = Chan::<RoleBLocal, RoleB, _>::new(server_io);
    
    // Test role access
    assert_eq!(client_chan.role().name(), "RoleA");
    assert_eq!(server_chan.role().name(), "RoleB");
    
    // Test protocol execution
    let client_chan = client_chan.send("Hello".to_string()).await.unwrap();
    let (message, server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(message, "Hello");
    
    // Close the channels
    client_chan.close().unwrap();
    server_chan.close().unwrap();
}

#[tokio::test]
async fn test_mpst_channel_with_three_roles() {
    // Define local protocols for a three-role scenario
    type RoleALocal = Send<String, End>;
    type RoleCLocal = Recv<i32, End>;
    
    // Create channels with roles
    let a_io = MockIO::new(Some("Hello".to_string()));
    let b_io_recv = MockIO::new(Some("Hello".to_string()));
    let b_io_send = MockIO::new(Some(42));
    let c_io = MockIO::new(Some(42));
    
    let chan_a = Chan::<RoleALocal, RoleA, _>::new(a_io);
    let chan_b_recv = Chan::<Recv<String, End>, RoleB, _>::new(b_io_recv);
    let chan_c = Chan::<RoleCLocal, RoleC, _>::new(c_io);
    
    // Test role access
    assert_eq!(chan_a.role().name(), "RoleA");
    assert_eq!(chan_b_recv.role().name(), "RoleB");
    assert_eq!(chan_c.role().name(), "RoleC");
    
    // Test protocol execution - first part
    let chan_a = chan_a.send("Hello".to_string()).await.unwrap();
    let (message, _) = chan_b_recv.recv().await.unwrap();
    assert_eq!(message, "Hello");
    
    // Second part - B sends to C
    let chan_b_send = Chan::<Send<i32, End>, RoleB, _>::new(b_io_send);
    let chan_b_send = chan_b_send.send(42).await.unwrap();
    let (response, chan_c) = chan_c.recv().await.unwrap();
    assert_eq!(response, 42);
    
    // Close the channels
    chan_a.close().unwrap();
    chan_b_send.close().unwrap();
    chan_c.close().unwrap();
}

#[tokio::test]
async fn test_mpst_channel_with_role_conversion() {
    // Define a global protocol: RoleA sends a String to RoleB, then ends
    type GlobalProtocol = GSend<String, RoleA, RoleB, GEnd>;
    
    // Project the global protocol for RoleA
    type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
    
    // Create a channel for RoleA
    let a_io = MockIO::new(Some("Hello".to_string()));
    let chan_a = Chan::<RoleALocal, RoleA, _>::new(a_io);
    
    // Create a channel for RoleB with the same protocol but a different role
    let role_b = RoleB;
    let chan_b = chan_a.for_role(role_b);
    
    // Test role access
    assert_eq!(chan_b.role().name(), "RoleB");
}

#[tokio::test]
async fn test_mpst_channel_with_protocol_conversion() {
    // Define a binary protocol
    type BinaryProtocol = Send<String, End>;
    
    // Create a channel with the binary protocol
    let io = MockIO::new(Some("Hello".to_string()));
    let chan = Chan::<BinaryProtocol, RoleA, _>::new(io);
    
    // Convert it to use an MPST wrapper
    let mpst_chan = chan.convert::<MPSTWrapper<BinaryProtocol, RoleA>>();
    
    // Test that the role is preserved
    assert_eq!(mpst_chan.role().name(), "RoleA");
}

#[tokio::test]
async fn test_mpst_channel_with_binary_compatibility() {
    // Define a binary protocol
    type BinaryProtocol = Send<String, End>;
    
    // Create a channel with the binary protocol
    let io = MockIO::new(Some("Hello".to_string()));
    let chan = Chan::<BinaryProtocol, RoleA, _>::new(io);
    
    // Test protocol execution
    let chan = chan.send("Hello".to_string()).await.unwrap();
    
    // Close the channel
    chan.close().unwrap();
    
    // Create a new channel for the next part of the test
    let io2 = MockIO::new(Some(42));
    let chan2 = Chan::<Recv<i32, End>, RoleA, _>::new(io2);
    
    // Continue with the protocol
    let (response, chan2) = chan2.recv().await.unwrap();
    assert_eq!(response, 42);
    
    // Close the channel
    chan2.close().unwrap();
}