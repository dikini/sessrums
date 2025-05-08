//! Multiparty session execution for session type protocols.
//!
//! This module provides the `MultipartySession` type, which implements the runtime
//! execution of local protocols in a multiparty session. It uses the typestate pattern
//! to ensure that protocol actions are executed in the correct order, as specified by
//! the local protocol.

use std::marker::PhantomData;
use serde::{Serialize, de::DeserializeOwned};

use crate::error::SessionError;
use crate::roles::Role;
use crate::transport::{MultipartyTransport, RoleIdentifier};

/// A marker trait for protocol states.
///
/// This trait is used to constrain the protocol state type parameter `P` in
/// `MultipartySession<R, P, T>`. It is implemented for the various protocol
/// state types like `Send`, `Receive`, and `End`.
///
/// This trait is sealed and cannot be implemented outside this crate.
pub trait ProtocolState: private::Sealed {}

/// A marker trait for protocol states that represent sending a message.
pub trait SendState<M>: ProtocolState {
    /// The next protocol state after sending a message.
    type Next: ProtocolState;
}

/// A marker trait for protocol states that represent receiving a message.
pub trait ReceiveState<M>: ProtocolState {
    /// The next protocol state after receiving a message.
    type Next: ProtocolState;
}

/// A marker trait for protocol states that represent the end of a protocol.
pub trait EndState: ProtocolState {}

// Private module to seal the ProtocolState trait
mod private {
    pub trait Sealed {}
    
    // Implement for common protocol state types
    impl<M, Next> Sealed for super::Send<M, Next> {}
    impl<M, Next> Sealed for super::Receive<M, Next> {}
    impl Sealed for super::End {}
}

/// A protocol state representing sending a message of type `M` and then
/// transitioning to the next state `Next`.
#[derive(Debug)]
pub struct Send<M, Next> {
    _message: PhantomData<M>,
    _next: PhantomData<Next>,
}

/// A protocol state representing receiving a message of type `M` and then
/// transitioning to the next state `Next`.
#[derive(Debug)]
pub struct Receive<M, Next> {
    _message: PhantomData<M>,
    _next: PhantomData<Next>,
}

/// A protocol state representing the end of a protocol.
#[derive(Debug)]
pub struct End;

// Implement the marker traits for the protocol state types
impl<M, Next: ProtocolState> ProtocolState for Send<M, Next> {}
impl<M, Next: ProtocolState> SendState<M> for Send<M, Next> {
    type Next = Next;
}

impl<M, Next: ProtocolState> ProtocolState for Receive<M, Next> {}
impl<M, Next: ProtocolState> ReceiveState<M> for Receive<M, Next> {
    type Next = Next;
}

impl ProtocolState for End {}
impl EndState for End {}

/// A multiparty session that executes a local protocol.
///
/// `MultipartySession<R, P, T>` represents a session from the perspective of role `R`,
/// currently in protocol state `P`, using transport implementation `T`.
///
/// The typestate pattern is used to ensure that protocol actions are executed in the
/// correct order, as specified by the local protocol. The type parameter `P` evolves
/// as protocol actions are executed, ensuring that only valid actions can be performed
/// at each step.
///
/// # Type Parameters
///
/// * `R` - The role type, implementing the `Role` trait
/// * `P` - The current protocol state, implementing the `ProtocolState` trait
/// * `T` - The multiparty transport implementation, implementing the `MultipartyTransport` trait
///
/// # Examples
///
/// ```compile_only
/// use sessrums_types::roles::Client;
/// use sessrums_types::session_types::{MultipartySession, MultipartySend, MultipartyReceive, MultipartyEnd};
/// use sessrums_types::transport::{MockMultipartyBroker, RoleIdentifier};
///
/// // Define a simple protocol: Client sends a String to Server, then receives an i32, then ends
/// type ClientProtocol = MultipartySend<String, MultipartyReceive<i32, MultipartyEnd>>;
///
/// // Create a broker and register participants
/// let broker = MockMultipartyBroker::new();
/// let client_id = RoleIdentifier::new("client");
/// let server_id = RoleIdentifier::new("server");
/// broker.register_participant(&client_id).unwrap();
/// broker.register_participant(&server_id).unwrap();
///
/// // Create a channel for the client
/// let client_channel = broker.create_channel::<Client>(&client_id).unwrap();
///
/// // Create a session for the client
/// let session = MultipartySession::<Client, ClientProtocol, _>::new(client_channel);
///
/// // The following code demonstrates how to use the session API,
/// // but we don't actually execute it in the doctest since it requires
/// // a server to respond
/// //
/// // let session = session.send(&server_id, "Hello".to_string()).unwrap();
/// // let (response, session) = session.receive::<i32>(&server_id).unwrap();
/// // let _transport = session.close().unwrap();
/// ```
#[derive(Debug)]
pub struct MultipartySession<R: Role, P: ProtocolState, T: MultipartyTransport> {
    /// The transport implementation used for communication
    transport: T,
    /// Phantom data for the role type
    _role: PhantomData<R>,
    /// Phantom data for the protocol state
    _protocol: PhantomData<P>,
}

impl<R: Role, P: ProtocolState, T: MultipartyTransport> MultipartySession<R, P, T> {
    /// Create a new multiparty session with the given transport.
    ///
    /// # Parameters
    ///
    /// * `transport` - The multiparty transport implementation to use
    ///
    /// # Returns
    ///
    /// A new `MultipartySession` in the initial protocol state
    pub fn new(transport: T) -> Self {
        MultipartySession {
            transport,
            _role: PhantomData,
            _protocol: PhantomData,
        }
    }
}

impl<R: Role, M: Serialize + 'static, Next: ProtocolState, T: MultipartyTransport> 
    MultipartySession<R, Send<M, Next>, T> {
    /// Send a message to the specified role.
    ///
    /// This method can only be called when the protocol state is `Send<M, Next>`,
    /// ensuring that sending is the next expected action in the protocol.
    ///
    /// # Parameters
    ///
    /// * `to` - The role identifier of the recipient
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// A `Result` containing the session in the next protocol state if successful,
    /// or a `SessionError` if the send operation failed
    pub fn send(
        mut self,
        to: &RoleIdentifier,
        message: M,
    ) -> Result<MultipartySession<R, Next, T>, SessionError> {
        self.transport.send_to(to, message)?;
        
        Ok(MultipartySession {
            transport: self.transport,
            _role: PhantomData,
            _protocol: PhantomData,
        })
    }
}

impl<R: Role, M: DeserializeOwned + 'static, Next: ProtocolState, T: MultipartyTransport> 
    MultipartySession<R, Receive<M, Next>, T> {
    /// Receive a message from the specified role.
    ///
    /// This method can only be called when the protocol state is `Receive<M, Next>`,
    /// ensuring that receiving is the next expected action in the protocol.
    ///
    /// # Parameters
    ///
    /// * `from` - The role identifier of the sender
    ///
    /// # Returns
    ///
    /// A `Result` containing the received message and the session in the next protocol state
    /// if successful, or a `SessionError` if the receive operation failed
    pub fn receive<Msg: DeserializeOwned + 'static>(
        mut self,
        from: &RoleIdentifier,
    ) -> Result<(Msg, MultipartySession<R, Next, T>), SessionError> {
        let message = self.transport.receive_from::<Msg>(from)?;
        
        Ok((message, MultipartySession {
            transport: self.transport,
            _role: PhantomData,
            _protocol: PhantomData,
        }))
    }
}

impl<R: Role, T: MultipartyTransport> MultipartySession<R, End, T> {
    /// Close the session.
    ///
    /// This method can only be called when the protocol state is `End`,
    /// ensuring that the protocol has been fully executed before closing.
    ///
    /// # Returns
    ///
    /// A `Result` containing the underlying transport if successful,
    /// or a `SessionError` if the close operation failed
    pub fn close(self) -> Result<T, SessionError> {
        Ok(self.transport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roles::{Client, Server};
    use crate::transport::MockMultipartyBroker;
    use std::sync::Arc;

    // Define simple protocol types for testing
    type ClientProtocol = Send<String, Receive<i32, End>>;
    type ServerProtocol = Receive<String, Send<i32, End>>;

    #[test]
    fn test_simple_protocol_execution() -> Result<(), SessionError> {
        // This test is more of a compilation test to ensure the typestate pattern works
        // We'll create a simplified version that doesn't rely on actual message passing
        
        // Create a broker and register participants
        let broker = Arc::new(MockMultipartyBroker::new());
        let client_id = RoleIdentifier::new("client");
        let server_id = RoleIdentifier::new("server");
        
        broker.register_participant(&client_id)?;
        broker.register_participant(&server_id)?;
        
        // Create channels for each participant
        let client_channel = broker.create_channel::<Client>(&client_id)?;
        let server_channel = broker.create_channel::<Server>(&server_id)?;
        
        // Create sessions for each participant
        let client_session = MultipartySession::<Client, ClientProtocol, _>::new(client_channel);
        let server_session = MultipartySession::<Server, ServerProtocol, _>::new(server_channel);
        
        // Verify that the type system allows these operations
        let _ = client_session.send(&server_id, "Hello".to_string());
        let _ = server_session.receive::<String>(&client_id);
        
        Ok(())
    }

    #[test]
    fn test_type_safety() {
        // This test verifies that the compiler enforces protocol adherence
        // If this code compiles, it means the typestate pattern is working correctly
        
        // Create a broker and register participants
        let broker = MockMultipartyBroker::new();
        let client_id = RoleIdentifier::new("client");
        let server_id = RoleIdentifier::new("server");
        
        broker.register_participant(&client_id).unwrap();
        broker.register_participant(&server_id).unwrap();
        
        // Create a channel for the client
        let client_channel = broker.create_channel::<Client>(&client_id).unwrap();
        
        // Create a session for the client
        let client_session = MultipartySession::<Client, ClientProtocol, _>::new(client_channel);
        
        // The following code would not compile because it violates the protocol:
        // Uncommenting any of these lines should result in a compilation error
        
        // let (_, _) = client_session.receive::<i32>(&server_id).unwrap(); // Error: can't receive before sending
        // let _ = client_session.close().unwrap(); // Error: can't close before completing the protocol
        
        // This is the correct sequence (we don't actually execute it since we don't have a server):
        // let client_session = client_session.send(&server_id, "Hello".to_string()).unwrap();
        // let (_, client_session) = client_session.receive::<i32>(&server_id).unwrap();
        // let _ = client_session.close().unwrap();
        
        // Just verify that the type system allows these operations
        let _ = client_session.send(&server_id, "Hello".to_string());
    }

    #[test]
    fn test_error_handling() {
        // Create a broker and register only the client
        let broker = MockMultipartyBroker::new();
        let client_id = RoleIdentifier::new("client");
        let server_id = RoleIdentifier::new("server");
        
        broker.register_participant(&client_id).unwrap();
        // Deliberately not registering the server
        
        // Create a channel for the client
        let client_channel = broker.create_channel::<Client>(&client_id).unwrap();
        
        // Create a session for the client
        let client_session = MultipartySession::<Client, ClientProtocol, _>::new(client_channel);
        
        // Sending to a non-existent server should fail
        let result = client_session.send(&server_id, "Hello".to_string());
        assert!(result.is_err());
    }
}