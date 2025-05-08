//! Binary session types implementation.
//! 
//! This module provides the core typestate machinery for binary (two-party)
//! session types, including Send, Receive, and End states.

use std::marker::PhantomData;
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use crate::{
    transport::Transport,
    error::SessionError,
};

/// Represents the binary choice signal sent over the transport to indicate
/// which branch of a protocol was selected.
///
/// This enum is used in the implementation of external choice (Offer/Select) in
/// the binary session type system. When a participant selects a branch in a protocol,
/// they send either `ChoiceSignal::Left` or `ChoiceSignal::Right` to indicate their choice.
///
/// # Serialization
///
/// `ChoiceSignal` implements `Serialize` and `Deserialize` from serde, allowing it to be
/// transmitted over any transport that implements the `Transport` trait. The serialization
/// is handled by the transport implementation, which typically uses a format like bincode
/// or JSON.
///
/// # Implementation Notes for Custom Transports
///
/// When implementing a custom `Transport`:
///
/// 1. Ensure your serialization mechanism can handle simple enums like `ChoiceSignal`
/// 2. The serialized representation should be compact and efficient, as choice signals
///    are frequently transmitted during protocol execution
/// 3. Error handling should account for potential deserialization failures of choice signals,
///    which could indicate protocol violations or transport corruption
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChoiceSignal {
    /// Indicates selection of the left branch in a binary choice
    Left,
    /// Indicates selection of the right branch in a binary choice
    Right,
}

/// Represents the result of offering a choice between two protocol continuations.
///
/// This enum is used to represent the outcome of an external choice operation,
/// where a participant offers two possible protocol continuations (L or R) and
/// the other participant selects one of them. When a participant calls the `offer()`
/// method, they receive an `Either<L, R>` that contains the selected continuation.
///
/// # Type Parameters
/// * `L` - The type representing the left branch continuation
/// * `R` - The type representing the right branch continuation
///
/// # Usage
///
/// `Either<L, R>` is typically used with pattern matching to handle both possible
/// continuations of a protocol:
///
/// ```rust
/// use sessrums_types::session_types::{Either, Offer, Session, Receive, End};
/// use sessrums_types::transport::MockChannelEnd;
/// use sessrums_types::error::SessionError;
/// use sessrums_types::Transport;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct StringMsg(String);
///
/// // Define protocol types
/// type ServerProtocol = Offer<
///     Receive<StringMsg, End>,
///     Receive<StringMsg, End>
/// >;
///
/// // Create a server session
/// let (client_chan, server_chan) = MockChannelEnd::new_pair();
/// let server = Session::<ServerProtocol, _>::new(server_chan);
///
/// // Wait for client to select a branch
/// # let mut other_end = client_chan;
/// # other_end.send_payload(&sessrums_types::session_types::ChoiceSignal::Left).unwrap();
/// match server.offer()? {
///     Either::Left(server) => {
///         // Handle left branch continuation
///         // server has type Session<Receive<StringMsg, End>, _>
///     },
///     Either::Right(server) => {
///         // Handle right branch continuation
///         // server has type Session<Receive<StringMsg, End>, _>
///     }
/// }
/// # Ok::<(), SessionError>(())
/// ```
///
/// # Relationship with Other Types
///
/// - Returned by the `offer()` method of `Session<Offer<L, R>, T>`
/// - Works with `ChoiceSignal` which determines which variant is returned
/// - Contains the session continuation with the appropriate protocol state
#[derive(Debug, Clone, PartialEq)]
pub enum Either<L, R> {
    /// Contains the left branch continuation
    Left(L),
    /// Contains the right branch continuation
    Right(R),
}

/// Represents the end of a session type protocol.
///
/// The `End` state signifies that a participant has completed their part of the protocol
/// and no further communication is expected. When a session reaches the `End` state,
/// it can be closed using the `close()` method, which returns the underlying transport.
///
/// # Duality
///
/// `End` is self-dual, meaning that when one participant reaches the `End` state,
/// the other participant must also reach the `End` state for the protocol to be well-formed.
///
/// # Example Usage
///
/// ```rust
/// use sessrums_types::session_types::{Send, End, Session};
/// use sessrums_types::transport::MockChannelEnd;
/// use sessrums_types::error::SessionError;
/// use sessrums_types::Transport;
/// use serde::{Serialize, Deserialize};
///
/// // Define a message type
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Message(String);
///
/// // In a protocol definition:
/// type SimpleProtocol = Send<Message, End>;
///
/// // In implementation:
/// let (chan, _) = MockChannelEnd::new_pair();
/// let session = Session::<SimpleProtocol, _>::new(chan);
/// let message = Message("Hello".to_string());
/// let session = session.send(message)?;
/// let transport = session.close(); // Session is now ended
/// # Ok::<(), SessionError>(())
/// ```
#[derive(Debug, Default)]
pub struct End;

/// Represents a protocol state where a message is sent before continuing to the next state.
///
/// The `Send<M, NextP>` state indicates that a participant must send a message of type `M`
/// and then continue with the protocol state `NextP`. This is one of the fundamental
/// building blocks of session types, representing the action of sending data.
///
/// # Type Parameters
/// * `M` - The type of message to be sent
/// * `NextP` - The protocol state to transition to after sending
///
/// # Duality
///
/// The dual of `Send<M, NextP>` is `Receive<M, NextP::DualType>`. When one participant
/// sends a message, the other must receive it.
///
/// # Example Usage
///
/// ```rust
/// use sessrums_types::session_types::{Send, Receive, End, Session};
/// use sessrums_types::transport::MockChannelEnd;
/// use sessrums_types::error::SessionError;
/// use sessrums_types::Transport;
/// use serde::{Serialize, Deserialize};
///
/// // Define message types
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Request { id: u32 }
///
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Response { id: u32, result: String }
///
/// // In a protocol definition:
/// type ClientProtocol = Send<Request, Receive<Response, End>>;
///
/// // In implementation:
/// let (chan, _) = MockChannelEnd::new_pair();
/// let client = Session::<ClientProtocol, _>::new(chan);
/// let client = client.send(Request { id: 42 })?;
/// // Now client has type Session<Receive<Response, End>, _>
/// # Ok::<(), SessionError>(())
/// ```
#[derive(Debug, Default)]
pub struct Send<M, NextP>(pub PhantomData<(M, NextP)>);

impl<M, NextP> Send<M, NextP> {
    /// Creates a new Send protocol state.
    pub fn new() -> Self {
        Send(PhantomData)
    }
}

/// Represents a protocol state where a message is received before continuing to the next state.
///
/// The `Receive<M, NextP>` state indicates that a participant must receive a message of type `M`
/// and then continue with the protocol state `NextP`. This is one of the fundamental
/// building blocks of session types, representing the action of receiving data.
///
/// # Type Parameters
/// * `M` - The type of message to be received
/// * `NextP` - The protocol state to transition to after receiving
///
/// # Duality
///
/// The dual of `Receive<M, NextP>` is `Send<M, NextP::DualType>`. When one participant
/// receives a message, the other must send it.
///
/// # Example Usage
///
/// ```rust
/// use sessrums_types::session_types::{Send, Receive, End, Session};
/// use sessrums_types::transport::MockChannelEnd;
/// use sessrums_types::error::SessionError;
/// use sessrums_types::Transport;
/// use serde::{Serialize, Deserialize};
///
/// // Define message types
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Request { id: u32 }
///
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Response { id: u32, result: String }
///
/// // In a protocol definition:
/// type ServerProtocol = Receive<Request, Send<Response, End>>;
///
/// // In implementation:
/// let (client_chan, server_chan) = MockChannelEnd::new_pair();
/// let server = Session::<ServerProtocol, _>::new(server_chan);
/// // Note: In a real scenario, the client would send a message first
/// # let mut other_end = client_chan;
/// # other_end.send_payload(&Request { id: 42 }).unwrap();
/// let (request, server) = server.receive()?;
/// // Now server has type Session<Send<Response, End>, _>
/// # Ok::<(), SessionError>(())
/// ```
#[derive(Debug, Default)]
pub struct Receive<M, NextP>(pub PhantomData<(M, NextP)>);

impl<M, NextP> Receive<M, NextP> {
    /// Creates a new Receive protocol state.
    pub fn new() -> Self {
        Receive(PhantomData)
    }
}

/// Represents a protocol state where a participant offers a choice between two continuations.
///
/// In session type theory, external choice is represented by the Offer construct, where
/// one participant offers a choice and the other selects which branch to take. The `Offer<L, R>`
/// struct represents the protocol state of the participant who offers the choice.
///
/// When a participant is in an `Offer<L, R>` state, they:
/// 1. Wait to receive a `ChoiceSignal` (Left or Right) from the other participant
/// 2. Based on the received signal, continue with either the `L` or `R` protocol continuation
///
/// # Duality
/// The dual of `Offer<L, R>` is `Select<L, R>`. When one participant offers a choice,
/// the other participant must select a branch.
///
/// # Relationship with Other Types
/// - Works with `ChoiceSignal` enum, which is sent over the transport to indicate the selected branch
/// - Returns an `Either<L, R>` containing the selected continuation
///
/// # Type Parameters
/// * `L` - The type representing the left branch continuation
/// * `R` - The type representing the right branch continuation
///
/// # Example Usage
/// ```rust
/// use sessrums_types::session_types::{Send, Receive, Offer, End, Session, Either, ChoiceSignal};
/// use sessrums_types::transport::MockChannelEnd;
/// use sessrums_types::error::SessionError;
/// use sessrums_types::Transport;
/// use serde::{Serialize, Deserialize};
///
/// // Define message types
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Request { id: u32 }
///
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Response1 { id: u32, result: String }
///
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Response2 { id: u32, error: String }
///
/// // In a protocol definition:
/// type ServerProtocol = Receive<Request, Offer<
///     Send<Response1, End>,  // Left branch
///     Send<Response2, End>   // Right branch
/// >>;
///
/// // In implementation:
/// let (client_chan, server_chan) = MockChannelEnd::new_pair();
/// let server = Session::<ServerProtocol, _>::new(server_chan);
/// # let mut other_end = client_chan;
/// # other_end.send_payload(&Request { id: 42 }).unwrap();
/// let (request, server) = server.receive()?;
/// # other_end.send_payload(&ChoiceSignal::Left).unwrap();
/// let branch = server.offer()?;
/// match branch {
///     Either::Left(server) => {
///         let response = Response1 { id: request.id, result: "Success".to_string() };
///         server.send(response)?
///     },
///     Either::Right(server) => {
///         let response = Response2 { id: request.id, error: "Error".to_string() };
///         server.send(response)?
///     },
/// };
/// # Ok::<(), SessionError>(())
/// ```
#[derive(Debug, Default)]
pub struct Offer<L, R>(PhantomData<(L, R)>);

/// Represents a protocol state where a participant selects between two continuations.
///
/// In session type theory, external choice is represented by the Select construct, where
/// one participant selects which branch to take and the other offers the choice. The `Select<L, R>`
/// struct represents the protocol state of the participant who makes the selection.
///
/// When a participant is in a `Select<L, R>` state, they:
/// 1. Decide which branch of the protocol to follow (Left or Right)
/// 2. Send the corresponding `ChoiceSignal` to the other participant
/// 3. Continue with either the `L` or `R` protocol continuation based on their selection
///
/// # Duality
/// The dual of `Select<L, R>` is `Offer<L, R>`. When one participant selects a branch,
/// the other participant must offer the choice.
///
/// # Relationship with Other Types
/// - Uses `ChoiceSignal` enum to communicate the selection over the transport
/// - Works with `Either<L, R>` to represent the selected continuation
///
/// # Type Parameters
/// * `L` - The type representing the left branch continuation
/// * `R` - The type representing the right branch continuation
///
/// # Example Usage
/// ```rust
/// use sessrums_types::session_types::{Send, Receive, Select, End, Session};
/// use sessrums_types::transport::MockChannelEnd;
/// use sessrums_types::error::SessionError;
/// use sessrums_types::Transport;
/// use serde::{Serialize, Deserialize};
///
/// // Define message types
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Request { id: u32 }
///
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Response1 { id: u32, result: String }
///
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Response2 { id: u32, error: String }
///
/// // In a protocol definition:
/// type ClientProtocol = Send<Request, Select<
///     Receive<Response1, End>,  // Left branch
///     Receive<Response2, End>   // Right branch
/// >>;
///
/// // In implementation:
/// let (client_chan, server_chan) = MockChannelEnd::new_pair();
/// let client = Session::<ClientProtocol, _>::new(client_chan);
/// let client = client.send(Request { id: 42 })?;
/// // Choose the left branch
/// let client = client.select_left()?;
/// # let mut other_end = server_chan;
/// # other_end.send_payload(&Response1 { id: 42, result: "Success".to_string() }).unwrap();
/// let (response, client) = client.receive()?;
/// # Ok::<(), SessionError>(())
/// ```
#[derive(Debug, Default)]
pub struct Select<L, R>(PhantomData<(L, R)>);

/// A trait for all protocol state types in the session type system.
///
/// This trait is implemented by all protocol state types, including `Send`, `Receive`,
/// `Offer`, `Select`, `End`, `Rec`, and `Var`. It enables generic handling of protocol states.
pub trait ProtocolState {}

// Implement for all existing protocol states
impl ProtocolState for End {}
impl<M, P> ProtocolState for Send<M, P> {}
impl<M, P> ProtocolState for Receive<M, P> {}
impl<L, R> ProtocolState for Offer<L, R> {}
impl<L, R> ProtocolState for Select<L, R> {}

/// Represents a recursive protocol definition.
///
/// The `Rec<F>` type is a fixed-point combinator that enables defining recursive protocols.
/// It wraps a function `F` that, when given a `Var`, produces the body of the recursive protocol.
///
/// # Type Parameters
/// * `F` - A function type that takes a `Var` and returns a protocol state
///
/// # Example
///
/// ```rust
/// use sessrums_types::session_types::{Rec, Var, Send, Receive, End};
/// use sessrums_types::messages::{PingMsg, PongMsg};
///
/// // Define a recursive ping-pong protocol
/// fn ping_pong_body(_: Var) -> Send<PingMsg, Receive<PongMsg, Var>> {
///     Send::new()
/// }
/// type RecursivePingPong = Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>;
///
/// // Create a recursive protocol state
/// let rec_state = Rec::new(ping_pong_body);
/// ```
#[derive(Debug)]
pub struct Rec<F>(pub F);

impl<F> Rec<F> {
    /// Creates a new Rec protocol state.
    pub fn new(f: F) -> Self {
        Rec(f)
    }
}

// Remove Default constraint from Rec

// Implement ProtocolState for Rec
impl<F> ProtocolState for Rec<F> {}

/// Represents a recursion variable in a recursive protocol.
///
/// The `Var` type is a marker for the recursion point in a recursive protocol.
/// When a session reaches a `Var` state, it should loop back to the beginning of the recursion.
///
/// # Example
///
/// ```rust
/// use sessrums_types::session_types::{Rec, Var, Send, Receive, End};
/// use sessrums_types::messages::{PingMsg, PongMsg};
///
/// // Define a recursive ping-pong protocol
/// type RecursivePingPong = Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>;
/// ```
#[derive(Debug, Default)]
pub struct Var;

// Implement ProtocolState for Var
impl ProtocolState for Var {}

/// A session with current protocol state S using transport T.
///
/// The `Session<S, T>` struct is the core runtime representation of a session-typed
/// communication channel. It combines a protocol state `S` with a transport implementation `T`
/// to provide a type-safe communication API.
///
/// Sessions evolve through a series of state transitions as defined by the protocol.
/// Each operation on a session (send, receive, offer, select) consumes the current session
/// and returns a new session with an updated protocol state, ensuring type safety throughout
/// the communication.
///
/// # Type Parameters
/// * `S` - The current protocol state (e.g., `Send<M, P>`, `Receive<M, P>`, `Offer<L, R>`, etc.)
/// * `T` - The transport implementation used for communication
///
/// # Example Usage
///
/// ```rust
/// use sessrums_types::session_types::{Send, Receive, End, Session};
/// use sessrums_types::transport::MockChannelEnd;
/// use sessrums_types::error::SessionError;
/// use sessrums_types::Transport;
/// use serde::{Serialize, Deserialize};
///
/// // Define message types
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Request { id: u32 }
///
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// struct Response { id: u32, result: String }
///
/// // Define a protocol
/// type ClientProtocol = Send<Request, Receive<Response, End>>;
///
/// // Create a session with the initial protocol state
/// let (client_chan, server_chan) = MockChannelEnd::new_pair();
/// let client = Session::<ClientProtocol, _>::new(client_chan);
///
/// // Use the session according to the protocol
/// let client = client.send(Request { id: 42 })?;
/// # let mut other_end = server_chan;
/// # other_end.send_payload(&Response { id: 42, result: "Success".to_string() }).unwrap();
/// let (response, client) = client.receive()?;
/// let transport = client.close();
/// # Ok::<(), SessionError>(())
/// ```
#[derive(Debug)]
pub struct Session<S, T: Transport> {
    state: S,
    channel: T,
}

// Session constructors and general methods
impl<S, T: Transport> Session<S, T> {
    /// Create a new session with the given transport channel.
    pub fn new(channel: T) -> Self
    where S: Default {
        Session {
            state: S::default(),
            channel,
        }
    }
    
    /// Create a new session with the given state and transport channel.
    /// This is useful for recursive protocols where the state doesn't implement Default.
    pub fn with_state(state: S, channel: T) -> Self {
        Session {
            state,
            channel,
        }
    }

    /// Get the underlying transport channel, consuming the session.
    pub fn into_transport(self) -> T {
        self.channel
    }
}

// Implementation for End state
impl<T: Transport> Session<End, T> {
    /// Close the session, returning the transport channel.
    pub fn close(self) -> T {
        self.channel
    }
}

// Implementation for Send state
impl<M, NextP, T: Transport> Session<Send<M, NextP>, T> 
where
    M: Serialize + 'static,
    NextP: Default,
{
    /// Send a message and transition to the next protocol state.
    pub fn send(mut self, message: M) -> Result<Session<NextP, T>, SessionError> {
        self.channel.send_payload(&message)?;
        
        Ok(Session {
            state: NextP::default(),
            channel: self.channel,
        })
    }
}

// Implementation for Receive state
impl<M, NextP, T: Transport> Session<Receive<M, NextP>, T>
where
    M: DeserializeOwned + 'static,
    NextP: Default,
{
    /// Receive a message and transition to the next protocol state.
    pub fn receive(mut self) -> Result<(M, Session<NextP, T>), SessionError> {
        let message = self.channel.receive_payload()?;
        
        Ok((message, Session {
            state: NextP::default(),
            channel: self.channel,
        }))
    }
}

// Implementation for Select state
impl<L, R, T: Transport> Session<Select<L, R>, T>
where
    L: Default,
    R: Default,
{
    /// Selects the left branch of a binary choice and transitions to the corresponding protocol state.
    ///
    /// This method is used when a participant wants to follow the left branch of a protocol choice.
    /// It sends a `ChoiceSignal::Left` over the transport to inform the other participant of this decision,
    /// and then transitions to the protocol state represented by type parameter `L`.
    ///
    /// # Returns
    /// - `Ok(Session<L, T>)` - A new session with the left branch protocol state if successful
    /// - `Err(SessionError)` - If there was an error sending the choice signal
    ///
    /// # Example Usage
    /// ```rust
    /// use sessrums_types::session_types::{Send, Receive, Select, End, Session};
    /// use sessrums_types::transport::MockChannelEnd;
    /// use sessrums_types::error::SessionError;
    /// use serde::{Serialize, Deserialize};
    ///
    /// // Define message types
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    /// struct RequestA { id: u32, action: String }
    ///
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    /// struct RequestB { id: u32, query: String }
    ///
    /// // In a protocol definition:
    /// type ClientProtocol = Select<
    ///     Send<RequestA, End>,  // Left branch
    ///     Send<RequestB, End>   // Right branch
    /// >;
    ///
    /// // In implementation:
    /// let (chan, _) = MockChannelEnd::new_pair();
    /// let client = Session::<ClientProtocol, _>::new(chan);
    /// // Choose the left branch
    /// let client = client.select_left()?;
    /// // Now client has type Session<Send<RequestA, End>, _>
    /// let request = RequestA { id: 42, action: "create".to_string() };
    /// let client = client.send(request)?;
    /// # Ok::<(), SessionError>(())
    /// ```
    pub fn select_left(mut self) -> Result<Session<L, T>, SessionError> {
        // Send the Left choice signal over the transport
        self.channel.send_payload(&ChoiceSignal::Left)?;
        
        // Transition to the left branch protocol state
        Ok(Session {
            state: L::default(),
            channel: self.channel,
        })
    }

    /// Selects the right branch of a binary choice and transitions to the corresponding protocol state.
    ///
    /// This method is used when a participant wants to follow the right branch of a protocol choice.
    /// It sends a `ChoiceSignal::Right` over the transport to inform the other participant of this decision,
    /// and then transitions to the protocol state represented by type parameter `R`.
    ///
    /// # Returns
    /// - `Ok(Session<R, T>)` - A new session with the right branch protocol state if successful
    /// - `Err(SessionError)` - If there was an error sending the choice signal
    ///
    /// # Example Usage
    /// ```rust
    /// use sessrums_types::session_types::{Send, Receive, Select, End, Session};
    /// use sessrums_types::transport::MockChannelEnd;
    /// use sessrums_types::error::SessionError;
    /// use serde::{Serialize, Deserialize};
    ///
    /// // Define message types
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    /// struct RequestA { id: u32, action: String }
    ///
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    /// struct RequestB { id: u32, query: String }
    ///
    /// // In a protocol definition:
    /// type ClientProtocol = Select<
    ///     Send<RequestA, End>,  // Left branch
    ///     Send<RequestB, End>   // Right branch
    /// >;
    ///
    /// // In implementation:
    /// let (chan, _) = MockChannelEnd::new_pair();
    /// let client = Session::<ClientProtocol, _>::new(chan);
    /// // Choose the right branch
    /// let client = client.select_right()?;
    /// // Now client has type Session<Send<RequestB, End>, _>
    /// let request = RequestB { id: 42, query: "status".to_string() };
    /// let client = client.send(request)?;
    /// # Ok::<(), SessionError>(())
    /// ```
    pub fn select_right(mut self) -> Result<Session<R, T>, SessionError> {
        // Send the Right choice signal over the transport
        self.channel.send_payload(&ChoiceSignal::Right)?;
        
        // Transition to the right branch protocol state
        Ok(Session {
            state: R::default(),
            channel: self.channel,
        })
    }
}

// Implementation for Offer state
impl<L, R, T: Transport> Session<Offer<L, R>, T>
where
    L: Default,
    R: Default,
{
    /// Offers a binary choice and waits for the other participant to select a branch.
    ///
    /// This method is used when a participant offers two possible protocol continuations
    /// and waits for the other participant to select which branch to follow. It receives
    /// a `ChoiceSignal` from the transport and returns either the left or right session
    /// continuation based on the received signal.
    ///
    /// # Returns
    /// - `Ok(Either<Session<L, T>, Session<R, T>>)` - Either the left or right session continuation
    ///   based on the choice made by the other participant
    /// - `Err(SessionError)` - If there was an error receiving the choice signal
    ///
    /// # Example Usage
    /// ```rust
    /// use sessrums_types::session_types::{Send, Receive, Offer, End, Session, Either, ChoiceSignal};
    /// use sessrums_types::transport::MockChannelEnd;
    /// use sessrums_types::error::SessionError;
    /// use sessrums_types::Transport;
    /// use serde::{Serialize, Deserialize};
    ///
    /// // Define message types
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    /// struct RequestA { id: u32, action: String }
    ///
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    /// struct RequestB { id: u32, query: String }
    ///
    /// // In a protocol definition:
    /// type ServerProtocol = Offer<
    ///     Receive<RequestA, End>,  // Left branch
    ///     Receive<RequestB, End>   // Right branch
    /// >;
    ///
    /// // In implementation:
    /// let (client_chan, server_chan) = MockChannelEnd::new_pair();
    /// let server = Session::<ServerProtocol, _>::new(server_chan);
    /// # let mut other_end = client_chan;
    /// # other_end.send_payload(&ChoiceSignal::Left).unwrap();
    /// // Wait for client to select a branch
    /// let branch = server.offer()?;
    /// match branch {
    ///     Either::Left(server) => {
    ///         // Client selected left branch
    ///         # other_end.send_payload(&RequestA { id: 42, action: "create".to_string() }).unwrap();
    ///         let (request_a, server) = server.receive()?;
    ///         // Process RequestA...
    ///         assert_eq!(request_a.id, 42);
    ///     },
    ///     Either::Right(server) => {
    ///         // Client selected right branch
    ///         # other_end.send_payload(&RequestB { id: 42, query: "status".to_string() }).unwrap();
    ///         let (request_b, server) = server.receive()?;
    ///         // Process RequestB...
    ///         assert_eq!(request_b.id, 42);
    ///     }
    /// }
    /// # Ok::<(), SessionError>(())
    /// ```
    pub fn offer(mut self) -> Result<Either<Session<L, T>, Session<R, T>>, SessionError> {
        // Receive the choice signal from the transport
        let choice: ChoiceSignal = self.channel.receive_payload()?;
        
        // Transition to either the left or right branch protocol state based on the received signal
        match choice {
            ChoiceSignal::Left => Ok(Either::Left(Session {
                state: L::default(),
                channel: self.channel,
            })),
            ChoiceSignal::Right => Ok(Either::Right(Session {
                state: R::default(),
                channel: self.channel,
            })),
        }
    }
}

// Implementation for Rec state
impl<F, P, T: Transport> Session<Rec<F>, T>
where
    F: FnOnce(Var) -> P,
    P: ProtocolState,
{
    /// Enters a recursive protocol by unrolling the recursion once.
    ///
    /// This method "unrolls" the recursion by invoking the closure in `Rec` with a `Var`,
    /// returning a session in the body state.
    ///
    /// # Returns
    /// - A new session with the protocol state produced by the function in `Rec`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sessrums_types::session_types::{Rec, Var, Send, Receive, End, Session};
    /// use sessrums_types::transport::MockChannelEnd;
    /// use sessrums_types::messages::{PingMsg, PongMsg};
    /// use sessrums_types::error::SessionError;
    ///
    /// // Define a recursive ping-pong protocol
    /// fn ping_pong_body(_: Var) -> Send<PingMsg, Receive<PongMsg, Var>> {
    ///     Send::new()
    /// }
    /// type RecursivePingPong = Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>;
    ///
    /// // Create a session with the recursive protocol
    /// let (chan, _) = MockChannelEnd::new_pair();
    /// let rec_state = Rec::new(ping_pong_body);
    /// let session = Session::with_state(rec_state, chan);
    ///
    /// // Enter the recursion
    /// let session = session.enter_rec();
    /// // Now session has type Session<Send<PingMsg, Receive<PongMsg, Var>>, _>
    /// # Ok::<(), SessionError>(())
    /// ```
    pub fn enter_rec(self) -> Session<P, T> {
        let body = (self.state.0)(Var);
        
        Session {
            state: body,
            channel: self.channel,
        }
    }
}

// Implementation for Var state
impl<T: Transport> Session<Var, T> {
    /// Loops back to the beginning of a recursive protocol.
    ///
    /// This method is used to jump back to the start of a recursion.
    /// It requires a function that recreates the recursive protocol.
    ///
    /// # Type Parameters
    /// * `F` - A function type that takes a `Var` and returns a protocol state
    /// * `P` - The protocol state type produced by `F`
    ///
    /// # Parameters
    /// * `f` - A function that produces the body of the recursive protocol
    ///
    /// # Returns
    /// - A new session with the recursive protocol state
    ///
    /// # Example
    ///
    /// ```rust
    /// use sessrums_types::session_types::{Rec, Var, Send, Receive, End, Session};
    /// use sessrums_types::transport::MockChannelEnd;
    /// use sessrums_types::messages::{PingMsg, PongMsg};
    /// use sessrums_types::error::SessionError;
    ///
    /// // Define a recursive function
    /// fn ping_pong_body(v: Var) -> Send<PingMsg, Receive<PongMsg, Var>> {
    ///     Send::new()
    /// }
    ///
    /// // Create a session that has reached a Var state
    /// let (chan, _) = MockChannelEnd::new_pair();
    /// let var_state = Var;
    /// let session = Session::with_state(var_state, chan);
    ///
    /// // Loop back to the beginning of the recursion
    /// let session = session.recurse(ping_pong_body);
    /// // Now session has type Session<Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>, _>
    /// # Ok::<(), SessionError>(())
    /// ```
    pub fn recurse<F, P>(self, f: F) -> Session<Rec<F>, T>
    where
        F: FnOnce(Var) -> P,
        P: ProtocolState,
    {
        Session {
            state: Rec(f),
            channel: self.channel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        messages::{PingMsg, PongMsg},
        transport::MockChannelEnd,
    };

    #[test]
    fn test_ping_pong_protocol() -> Result<(), SessionError> {
        // Create mock channel pair
        let (client_chan, server_chan) = MockChannelEnd::new_pair();
    
        // Define protocol steps using type aliases
        type PingStep = Send<PingMsg, Receive<PongMsg, End>>;
        type PongStep = Receive<PingMsg, Send<PongMsg, End>>;
    
        // Create client and server sessions
        let client = Session::<PingStep, _>::new(client_chan);
        let server = Session::<PongStep, _>::new(server_chan);
    
        // Run the protocol
        let ping = PingMsg { seq: Some(1) };
        
        // Client sends ping
        let client = client.send(ping)?;
        
        // Server receives ping and sends pong
        let (received_ping, server) = server.receive()?;
        assert_eq!(received_ping.seq, Some(1));
        
        let pong = PongMsg { 
            seq: received_ping.seq, 
            timestamp: 0 
        };
        let server = server.send(pong)?;
        
        // Client receives pong
        let (received_pong, client) = client.receive()?;
        assert_eq!(received_pong.seq, Some(1));
        assert_eq!(received_pong.timestamp, 0);
        
        // Close both sessions
        let _client_chan = client.close();
        let _server_chan = server.close();
        
        Ok(())
    }

    #[test]
    fn test_session_type_safety() {
        let (client_chan, _) = MockChannelEnd::new_pair();
        let session = Session::<End, _>::new(client_chan);
        
        // Following line would not compile:
        // let _ = session.send(PingMsg { seq: None });
        
        // But we can close an End session:
        let _ = session.close();
    }

    #[test]
    fn test_choice_protocol() -> Result<(), SessionError> {
        // Create mock channel pair
        let (client_chan, server_chan) = MockChannelEnd::new_pair();
        
        // Define protocol steps using type aliases
        // Client protocol: Select between sending "Hello" or "Goodbye"
        type ClientProtocol = Select<
            Send<String, End>,  // Left branch
            Send<String, End>   // Right branch
        >;
        
        // Server protocol: Offer to receive either "Hello" or "Goodbye"
        type ServerProtocol = Offer<
            Receive<String, End>,  // Left branch
            Receive<String, End>   // Right branch
        >;
        
        // Create client and server sessions
        let client = Session::<ClientProtocol, _>::new(client_chan);
        let server = Session::<ServerProtocol, _>::new(server_chan);
        
        // Test left branch
        {
            // Client selects left branch
            let client = client.select_left()?;
            
            // Server offers choice and receives client's selection
            let server_branch = server.offer()?;
            
            // Server should receive the left branch
            match server_branch {
                Either::Left(server) => {
                    // Client sends "Hello"
                    let client = client.send("Hello".to_string())?;
                    
                    // Server receives "Hello"
                    let (message, server) = server.receive()?;
                    assert_eq!(message, "Hello");
                    
                    // Close both sessions
                    let _client_chan = client.close();
                    let _server_chan = server.close();
                },
                Either::Right(_) => {
                    panic!("Server received Right branch when Left was selected");
                }
            }
        }
        
        // Create new channel pair for right branch test
        let (client_chan, server_chan) = MockChannelEnd::new_pair();
        let client = Session::<ClientProtocol, _>::new(client_chan);
        let server = Session::<ServerProtocol, _>::new(server_chan);
        
        // Test right branch
        {
            // Client selects right branch
            let client = client.select_right()?;
            
            // Server offers choice and receives client's selection
            let server_branch = server.offer()?;
            
            // Server should receive the right branch
            match server_branch {
                Either::Left(_) => {
                    panic!("Server received Left branch when Right was selected");
                },
                Either::Right(server) => {
                    // Client sends "Goodbye"
                    let client = client.send("Goodbye".to_string())?;
                    
                    // Server receives "Goodbye"
                    let (message, server) = server.receive()?;
                    assert_eq!(message, "Goodbye");
                    
                    // Close both sessions
                    let _client_chan = client.close();
                    let _server_chan = server.close();
                }
            }
        }
        
        Ok(())
    }
    
    #[test]
    fn test_duality_relationships() {
        // This test verifies duality relationships at compile time
        // If this test compiles, it means all the type relationships are correct
        
        // Test End duality
        let _: Same<End, <End as Dual>::DualType> = Same::new();
        
        // Test Send/Receive duality
        let _: Same<Receive<String, End>, <Send<String, End> as Dual>::DualType> = Same::new();
        let _: Same<Send<String, End>, <Receive<String, End> as Dual>::DualType> = Same::new();
        
        // Test nested Send/Receive duality
        let _: Same<
            Receive<String, Send<i32, End>>,
            <Send<String, Receive<i32, End>> as Dual>::DualType
        > = Same::new();
        
        // Test Offer/Select duality
        let _: Same<
            Select<Receive<String, End>, Receive<i32, End>>,
            <Offer<Send<String, End>, Send<i32, End>> as Dual>::DualType
        > = Same::new();
        
        let _: Same<
            Offer<Receive<String, End>, Receive<i32, End>>,
            <Select<Send<String, End>, Send<i32, End>> as Dual>::DualType
        > = Same::new();
        
        // Test complex nested duality
        type ComplexProtocol = Send<String, Offer<
            Receive<i32, End>,
            Send<bool, End>
        >>;
        
        type DualComplexProtocol = Receive<String, Select<
            Send<i32, End>,
            Receive<bool, End>
        >>;
        
        let _: Same<DualComplexProtocol, <ComplexProtocol as Dual>::DualType> = Same::new();
    }
    
    // Helper struct for compile-time type equality checking
    struct Same<T, U>(std::marker::PhantomData<(T, U)>);
    
    impl<T> Same<T, T> {
        fn new() -> Self {
            Same(std::marker::PhantomData)
        }
    }
}

/// The `Dual` trait represents the duality relationship between session types.
///
/// In session type theory, duality is a fundamental concept that ensures protocol
/// compatibility between two participants. For each protocol state of one participant,
/// there is a corresponding dual state for the other participant:
///
/// - When one participant sends, the other must receive
/// - When one participant offers a choice, the other must select
/// - When one participant ends the session, the other must also end
///
/// This trait provides an associated type `DualType` which represents the dual
/// of the implementing type.
///
/// # Examples
///
/// ```rust
/// use sessrums_types::session_types::{Send, Receive, End, Dual};
/// use serde::{Serialize, Deserialize};
///
/// // Define message types
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// struct Request { id: u32 }
///
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// struct Response { id: u32, result: String }
///
/// // If Alice's protocol is:
/// type AliceProtocol = Send<Request, Receive<Response, End>>;
///
/// // Then Bob's protocol must be the dual:
/// type BobProtocol = Receive<Request, Send<Response, End>>;
///
/// // We can verify this at compile time:
/// fn check_duality<T: Dual>() where T::DualType: Dual {}
/// fn _test() {
///     // This compiles only if BobProtocol is the dual of AliceProtocol
///     check_duality::<AliceProtocol>();
/// }
/// ```
pub trait Dual {
    /// The dual type of the implementing type.
    type DualType;
}

// End is self-dual
impl Dual for End {
    /// End is self-dual - when one participant ends, the other must also end.
    type DualType = End;
}

// Send is dual to Receive
impl<M, P> Dual for Send<M, P>
where
    P: Dual,
{
    /// The dual of Send<M, P> is Receive<M, Dual<P>>
    /// When one participant sends a message of type M and continues with protocol P,
    /// the other participant must receive a message of type M and continue with the dual of P.
    type DualType = Receive<M, P::DualType>;
}

// Receive is dual to Send
impl<M, P> Dual for Receive<M, P>
where
    P: Dual,
{
    /// The dual of Receive<M, P> is Send<M, Dual<P>>
    /// When one participant receives a message of type M and continues with protocol P,
    /// the other participant must send a message of type M and continue with the dual of P.
    type DualType = Send<M, P::DualType>;
}

// Offer is dual to Select
impl<L, R> Dual for Offer<L, R>
where
    L: Dual,
    R: Dual,
{
    /// The dual of Offer<L, R> is Select<Dual<L>, Dual<R>>
    ///
    /// When one participant offers a choice between protocols L and R,
    /// the other participant must select between the duals of L and R.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sessrums_types::session_types::{Send, Receive, Offer, Select, End, Dual};
    /// use serde::{Serialize, Deserialize};
    ///
    /// // Define message types
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    /// struct Response1 { id: u32, result: String }
    ///
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    /// struct Response2 { id: u32, error: String }
    ///
    /// // If Server's protocol is:
    /// type ServerProtocol = Offer<
    ///     Send<Response1, End>,
    ///     Send<Response2, End>
    /// >;
    ///
    /// // Then Client's protocol must be:
    /// type ClientProtocol = Select<
    ///     Receive<Response1, End>,
    ///     Receive<Response2, End>
    /// >;
    ///
    /// // We can verify this at compile time:
    /// fn check_duality<T: Dual>() where T::DualType: Dual {}
    /// fn _test() {
    ///     // This compiles only if ClientProtocol is the dual of ServerProtocol
    ///     check_duality::<ServerProtocol>();
    /// }
    /// ```
    type DualType = Select<L::DualType, R::DualType>;
}

// Select is dual to Offer
impl<L, R> Dual for Select<L, R>
where
    L: Dual,
    R: Dual,
{
    /// The dual of Select<L, R> is Offer<Dual<L>, Dual<R>>
    ///
    /// When one participant selects between protocols L and R,
    /// the other participant must offer a choice between the duals of L and R.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sessrums_types::session_types::{Send, Receive, Offer, Select, End, Dual};
    /// use serde::{Serialize, Deserialize};
    ///
    /// // Define message types
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    /// struct Request1 { id: u32, action: String }
    ///
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    /// struct Request2 { id: u32, query: String }
    ///
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    /// struct Response1 { id: u32, result: String }
    ///
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    /// struct Response2 { id: u32, error: String }
    ///
    /// // If Client's protocol is:
    /// type ClientProtocol = Select<
    ///     Send<Request1, Receive<Response1, End>>,
    ///     Send<Request2, Receive<Response2, End>>
    /// >;
    ///
    /// // Then Server's protocol must be:
    /// type ServerProtocol = Offer<
    ///     Receive<Request1, Send<Response1, End>>,
    ///     Receive<Request2, Send<Response2, End>>
    /// >;
    ///
    /// // We can verify this at compile time:
    /// fn check_duality<T: Dual>() where T::DualType: Dual {}
    /// fn _test() {
    ///     // This compiles only if ServerProtocol is the dual of ClientProtocol
    ///     check_duality::<ClientProtocol>();
    /// }
    /// ```
    type DualType = Offer<L::DualType, R::DualType>;
}

// Var is self-dual
impl Dual for Var {
    /// Var is self-dual - when one participant reaches a recursion variable,
    /// the other must also reach a recursion variable.
    type DualType = Var;
}

// Rec is dual to Rec with a dual body
impl<F, P> Dual for Rec<F>
where
    F: FnOnce(Var) -> P,
    P: Dual,
{
    /// The dual of Rec<F> is Rec<G> where G produces the dual of what F produces.
    ///
    /// When one participant has a recursive protocol, the other participant must also
    /// have a recursive protocol with a dual body.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sessrums_types::session_types::{Rec, Var, Send, Receive, End, Dual};
    /// use sessrums_types::messages::{PingMsg, PongMsg};
    ///
    /// // Define protocol body functions
    /// fn client_body(_: Var) -> Send<PingMsg, Receive<PongMsg, Var>> {
    ///     Send::new()
    /// }
    ///
    /// fn server_body(_: Var) -> Receive<PingMsg, Send<PongMsg, Var>> {
    ///     Receive::new()
    /// }
    ///
    /// // Client's recursive protocol
    /// type ClientProtocol = Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>;
    ///
    /// // Server's protocol should be the dual
    /// type ServerProtocol = Rec<fn(Var) -> Receive<PingMsg, Send<PongMsg, Var>>>;
    ///
    /// // We can verify this at compile time
    /// fn check_duality<T: Dual>() where T::DualType: Dual {}
    /// fn _test() {
    ///     check_duality::<ClientProtocol>();
    /// }
    /// ```
    type DualType = Rec<fn(Var) -> P::DualType>;
}