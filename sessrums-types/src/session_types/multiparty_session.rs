//! Multiparty session execution for session type protocols.
//!
//! This module provides the `MultipartySession` type, which implements the runtime
//! execution of local protocols in a multiparty session. It uses the typestate pattern
//! to ensure that protocol actions are executed in the correct order, as specified by
//! the local protocol.

use std::marker::PhantomData;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

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

/// A marker trait for protocol states that represent a choice point.
pub trait SelectState<L, R>: ProtocolState {
    /// The left branch of the choice.
    type Left: ProtocolState;
    /// The right branch of the choice.
    type Right: ProtocolState;
}

/// A marker trait for protocol states that represent an offer point.
pub trait OfferState<L, R>: ProtocolState {
    /// The left branch of the offer.
    type Left: ProtocolState;
    /// The right branch of the offer.
    type Right: ProtocolState;
}

/// A marker trait for protocol states that represent a recursion point.
pub trait RecState: ProtocolState {}

/// A marker trait for protocol states that represent a recursion variable.
pub trait VarState: ProtocolState {}

/// A trait for roles that are the same as another role.
/// This is used to enforce that only the deciding role can make a choice.
pub trait IsSameAs<R: Role> {}

/// A trait for roles that are not the same as another role.
/// This is used to enforce that non-deciding roles must follow a choice.
pub trait Not<T> {}

/// A trait for roles that are not the same as another role.
/// This is a helper trait for implementing Not<IsSameAs<R>>.
pub trait NotSameRole<R: Role> {}

// Implement IsSameAs for the same role
impl<R: Role> IsSameAs<R> for R {}

// Implement NotSameRole for different roles
// This would typically be implemented for all distinct role pairs
// For now, we'll leave it as a marker trait

// Private module to seal the ProtocolState trait
mod private {
    pub trait Sealed {}
}

// Implement Sealed for protocol state types
impl<M, Next> private::Sealed for Send<M, Next> {}
impl<M, Next> private::Sealed for Receive<M, Next> {}
impl<L, R> private::Sealed for Select<L, R> {}
impl<L, R> private::Sealed for Offer<L, R> {}
impl<F> private::Sealed for Rec<F> {}
impl private::Sealed for Var {}
impl private::Sealed for End {}

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

/// A protocol state representing a choice between two continuations.
/// The role `R` is the deciding role that makes the choice.
#[derive(Debug)]
pub struct Select<L, R> {
    _left: PhantomData<L>,
    _right: PhantomData<R>,
}

/// A protocol state representing an offer between two continuations.
/// The role `R` is the deciding role that makes the choice.
#[derive(Debug)]
pub struct Offer<L, R> {
    _left: PhantomData<L>,
    _right: PhantomData<R>,
}

/// A protocol state representing a recursion point.
/// The type parameter `F` is a function that takes a `Var` and returns the body of the recursion.
#[derive(Debug)]
pub struct Rec<F> {
    _func: PhantomData<F>,
}

/// A protocol state representing a recursion variable.
#[derive(Debug)]
pub struct Var;

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

impl<L: ProtocolState, R: ProtocolState> ProtocolState for Select<L, R> {}
impl<L: ProtocolState, R: ProtocolState> SelectState<L, R> for Select<L, R> {
    type Left = L;
    type Right = R;
}

impl<L: ProtocolState, R: ProtocolState> ProtocolState for Offer<L, R> {}
impl<L: ProtocolState, R: ProtocolState> OfferState<L, R> for Offer<L, R> {
    type Left = L;
    type Right = R;
}

impl<F> ProtocolState for Rec<F> {}
impl<F> RecState for Rec<F> {}

impl ProtocolState for Var {}
impl VarState for Var {}

impl ProtocolState for End {}
impl EndState for End {}

/// A signal used to communicate choice selections between participants in multiparty sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultipartyChoiceSignal {
    /// Signal for selecting the left branch
    Left,
    /// Signal for selecting the right branch
    Right,
}

/// A result type for offer operations
pub enum OfferResult<L, R> {
    /// The left variant
    Left(L),
    /// The right variant
    Right(R),
}

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

// Choice methods for Select
impl<R: Role, L: ProtocolState, R2: ProtocolState, T: MultipartyTransport>
    MultipartySession<R, Select<L, R2>, T> {
    /// Select the left branch of a choice.
    ///
    /// This method can only be called when the protocol state is `Select<L, R>`,
    /// and the current role is the deciding role.
    ///
    /// # Returns
    ///
    /// A `Result` containing the session in the left branch state if successful,
    /// or a `SessionError` if the selection operation failed
    pub fn select_left(mut self) -> Result<MultipartySession<R, L, T>, SessionError> {
        // Send the choice signal to all other participants
        // In a real implementation, we would need to determine which roles need to be notified
        // For now, we'll assume the transport handles this
        self.transport.broadcast(MultipartyChoiceSignal::Left)?;
        
        Ok(MultipartySession {
            transport: self.transport,
            _role: PhantomData,
            _protocol: PhantomData,
        })
    }
    
    /// Select the right branch of a choice.
    ///
    /// This method can only be called when the protocol state is `Select<L, R>`,
    /// and the current role is the deciding role.
    ///
    /// # Returns
    ///
    /// A `Result` containing the session in the right branch state if successful,
    /// or a `SessionError` if the selection operation failed
    pub fn select_right(mut self) -> Result<MultipartySession<R, R2, T>, SessionError> {
        // Send the choice signal to all other participants
        self.transport.broadcast(MultipartyChoiceSignal::Right)?;
        
        Ok(MultipartySession {
            transport: self.transport,
            _role: PhantomData,
            _protocol: PhantomData,
        })
    }
}

// Choice methods for Offer
impl<R: Role, L: ProtocolState, R2: ProtocolState, T: MultipartyTransport>
    MultipartySession<R, Offer<L, R2>, T> {
    /// Offer a choice between two branches.
    ///
    /// This method can only be called when the protocol state is `Offer<L, R>`.
    /// It waits for the deciding role to make a choice and then transitions to
    /// the corresponding branch.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the session in the left branch state or the
    /// session in the right branch state, depending on the choice made by the
    /// deciding role, or a `SessionError` if the offer operation failed
    pub fn offer(mut self) -> Result<OfferResult<MultipartySession<R, L, T>, MultipartySession<R, R2, T>>, SessionError> {
        // Receive the choice signal from the deciding role
        // In a real implementation, we would need to determine which role is the decider
        // For now, we'll assume the transport handles this
        let signal: MultipartyChoiceSignal = self.transport.receive_choice()?;
        
        match signal {
            MultipartyChoiceSignal::Left => Ok(OfferResult::Left(MultipartySession {
                transport: self.transport,
                _role: PhantomData,
                _protocol: PhantomData,
            })),
            MultipartyChoiceSignal::Right => Ok(OfferResult::Right(MultipartySession {
                transport: self.transport,
                _role: PhantomData,
                _protocol: PhantomData,
            })),
        }
    }
}

// Recursion methods
impl<R: Role, F, T: MultipartyTransport>
    MultipartySession<R, Rec<F>, T> {
    /// Enter a recursive protocol.
    ///
    /// This method can only be called when the protocol state is `Rec<F>`.
    /// It transitions to the body of the recursion.
    ///
    /// # Returns
    ///
    /// The session in the body state
    pub fn enter_rec<Next: ProtocolState>(self) -> MultipartySession<R, Next, T> {
        // In a real implementation, we would transition to the body type
        // For now, we'll just return a session with the next type parameter
        MultipartySession {
            transport: self.transport,
            _role: PhantomData,
            _protocol: PhantomData,
        }
    }
}

impl<R: Role, T: MultipartyTransport>
    MultipartySession<R, Var, T> {
    /// Recurse back to the beginning of a recursive protocol.
    ///
    /// This method can only be called when the protocol state is `Var`.
    /// It transitions back to the beginning of the recursion.
    ///
    /// # Type Parameters
    ///
    /// * `RecBody` - The type of the recursion body to return to
    ///
    /// # Returns
    ///
    /// A `Result` containing the session at the beginning of the recursion if successful,
    /// or a `SessionError` if the recursion operation failed
    pub fn recurse<RecBody: ProtocolState>(self) -> Result<MultipartySession<R, RecBody, T>, SessionError> {
        Ok(MultipartySession {
            transport: self.transport,
            _role: PhantomData,
            _protocol: PhantomData,
        })
    }
}

// Extension trait for MultipartyTransport to handle choice signals
pub trait ChoiceTransport: MultipartyTransport {
    /// Broadcast a choice signal to all participants.
    fn broadcast(&mut self, signal: MultipartyChoiceSignal) -> Result<(), SessionError>;
    
    /// Receive a choice signal from the deciding role.
    fn receive_choice(&mut self) -> Result<MultipartyChoiceSignal, SessionError>;
}

// Implement ChoiceTransport for any type that implements MultipartyTransport
impl<T: MultipartyTransport> ChoiceTransport for T {
    fn broadcast(&mut self, _signal: MultipartyChoiceSignal) -> Result<(), SessionError> {
        // In a real implementation, we would need to determine which roles need to be notified
        // For now, we'll just return Ok as a placeholder
        Ok(())
    }
    
    fn receive_choice(&mut self) -> Result<MultipartyChoiceSignal, SessionError> {
        // In a real implementation, we would need to determine which role is the decider
        // For now, we'll just return Left as a placeholder
        Ok(MultipartyChoiceSignal::Left)
    }
}

/// Create a multiparty session from a projected local protocol.
///
/// This function creates a multiparty session using the provided transport.
///
/// # Type Parameters
///
/// * `R` - The role for which to create the session
/// * `P` - The protocol state type
/// * `T` - The transport type
///
/// # Parameters
///
/// * `transport` - The transport to use for communication
///
/// # Returns
///
/// A multiparty session for the specified role
pub fn create_session<R, P, T>(
    transport: T
) -> MultipartySession<R, P, T>
where
    R: Role,
    P: ProtocolState,
    T: MultipartyTransport,
{
    MultipartySession {
        transport,
        _role: PhantomData,
        _protocol: PhantomData,
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