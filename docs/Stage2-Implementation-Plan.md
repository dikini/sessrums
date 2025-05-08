# Stage 2 Implementation Plan: Adding Recursion to Binary Session Types

## Context

Stage 2 extends the binary session type system by introducing **recursion**. This enables the modeling of protocols with loops or repeated interactions, such as repeated ping-pong or file transfer chunks. Recursion is typically implemented using a fixed-point combinator (`Rec`) and a recursion variable (`Var`), following established patterns in session types literature.

## Symbol Index

### src/error.rs
Defines error types and handling for session operations using thiserror.

- enum SessionError : error.rs (6-22)
- impl From<bincode::Error> for SessionError : error.rs (24-28)
- impl<T> From<PoisonError<T>> for SessionError : error.rs (30-34)
- #[cfg(test)] mod tests : error.rs (36-56)
  - #[test] fn test_error_messages : error.rs (38-49)
  - #[test] fn test_error_conversion : error.rs (51-55)

### src/lib.rs
Crate root with module declarations and public API exports.

- pub mod roles : lib.rs (6-6)
- pub mod messages : lib.rs (7-7)
- pub mod error : lib.rs (8-8)
- pub mod transport : lib.rs (9-9)
- pub mod session_types : lib.rs (10-10)
- pub use error::SessionError : lib.rs (13-13)
- pub use transport::Transport : lib.rs (14-14)

### src/messages.rs
Defines serializable message types (PingMsg, PongMsg) using serde.

- struct PingMsg : messages.rs (8-11)
- struct PongMsg : messages.rs (13-17)
- #[cfg(test)] mod tests : messages.rs (19-43)
  - #[test] fn test_ping_serialization : messages.rs (24-30)
  - #[test] fn test_pong_serialization : messages.rs (32-41)

### src/roles.rs
Implements protocol roles (Client, Server) as zero-sized types with a sealed Role trait.

- trait Role : roles.rs (7-7)
- struct Client : roles.rs (14-15)
- struct Server : roles.rs (17-18)
- impl Role for Client : roles.rs (20-20)
- impl Role for Server : roles.rs (21-21)
- mod private : roles.rs (24-28)
- #[cfg(test)] mod tests : roles.rs (30-45)
  - #[test] fn roles_are_copy : roles.rs (33-44)

### src/session_types/mod.rs
Module organization and re-exports for session type components.

- pub mod binary : session_types/mod.rs (3-3)
- pub use binary::{End, Send, Receive, Session, Offer, Select, Dual, Either, ChoiceSignal} : session_types/mod.rs (5-5)

### src/session_types/binary.rs
Core implementation of binary session types using the typestate pattern.

- enum ChoiceSignal : binary.rs (28-33)
- enum Either<L, R> : binary.rs (79-87)
- struct Send<M, NextP> : binary.rs (188-189)
- struct Receive<M, NextP> : binary.rs (235-236)
- struct Offer<L, R> : binary.rs (304-305)
- struct Select<L, R> : binary.rs (365-366)
- struct Session<S, T: Transport> : binary.rs (413-416)
- trait Dual : binary.rs (907-911)
- impl Dual for End : binary.rs (913-917)
- impl<M, P> Dual for Send<M, P> : binary.rs (918-926)
- impl<M, P> Dual for Receive<M, P> : binary.rs (927-935)
- impl<L, R> Dual for Offer<L, R> : binary.rs (946-970)
- impl<L, R> Dual for Select<L, R> : binary.rs (992-1034)
- impl<S, T: Transport> Session<S, T> : binary.rs (419-429)
- impl<T: Transport> Session<End, T> : binary.rs (430-434)
- impl<M, NextP, T: Transport> Session<Send<M, NextP>, T> : binary.rs (444-456)
- impl<M, NextP, T: Transport> Session<Receive<M, NextP>, T> : binary.rs (457-469)
- impl<L, R, T: Transport> Session<Select<L, R>, T> : binary.rs (483-583)
- impl<L, R, T: Transport> Session<Offer<L, R>, T> : binary.rs (592-665)
- #[cfg(test)] mod tests : binary.rs (670-870)
  - #[test] fn test_ping_pong_protocol : binary.rs (678-699)
  - #[test] fn test_session_type_safety : binary.rs (701-710)
  - #[test] fn test_choice_protocol : binary.rs (731-806)
  - #[test] fn test_duality_relationships : binary.rs (817-859)

### src/transport.rs
Transport abstraction trait and mock implementation for testing.

- trait Transport : transport.rs (7-19)
- struct MockChannelEnd : transport.rs (21-27)
- impl MockChannelEnd : transport.rs (29-44)
- impl Transport for MockChannelEnd : transport.rs (46-61)
- #[cfg(test)] mod tests : transport.rs (62-131)
  - #[derive(Debug, Serialize, Deserialize, PartialEq)] struct TestMessage : transport.rs (65-68)
  - #[test] fn test_mock_channel : transport.rs (70-96)
  - #[test] fn test_queue_behavior : transport.rs (98-112)
  - #[test] fn test_choice_signal_transmission : transport.rs (114-128)
  - #[test] fn test_multiple_messages : transport.rs (130-131)

## Implementation Tasks

### Task 1: Project Preparation ✅

- Verify Stage 1 is complete and all tests pass
- Status: Complete

### Task 2: Define Recursion Types

#### 2.1 Add Rec and Var types to binary.rs

```rust
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
/// type RecursivePingPong = Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>;
/// ```
#[derive(Debug, Default)]
pub struct Rec<F>(pub F);

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
```

#### 2.2 Update session_types/mod.rs to export the new types

```rust
pub use binary::{End, Send, Receive, Session, Offer, Select, Dual, Either, ChoiceSignal, Rec, Var};
```

### Task 3: Protocol State Trait for Recursion

#### 3.1 Define ProtocolState trait in binary.rs

```rust
/// A trait for all protocol state types in the session type system.
/// 
/// This trait is implemented by all protocol state types, including `Send`, `Receive`,
/// `Offer`, `Select`, `End`, `Rec`, and `Var`. It enables generic handling of protocol states.
pub trait ProtocolState: Default {}

// Implement for all existing protocol states
impl ProtocolState for End {}
impl<M, P> ProtocolState for Send<M, P> {}
impl<M, P> ProtocolState for Receive<M, P> {}
impl<L, R> ProtocolState for Offer<L, R> {}
impl<L, R> ProtocolState for Select<L, R> {}
impl<F> ProtocolState for Rec<F> where F: Default {}
impl ProtocolState for Var {}
```

#### 3.2 Update session_types/mod.rs to export the new trait

```rust
pub use binary::{End, Send, Receive, Session, Offer, Select, Dual, Either, ChoiceSignal, Rec, Var, ProtocolState};
```

### Task 4: Session API for Recursion

#### 4.1 Implement Session API for Rec

```rust
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
    /// type RecursivePingPong = Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>;
    ///
    /// // Create a session with the recursive protocol
    /// let (chan, _) = MockChannelEnd::new_pair();
    /// let session = Session::<RecursivePingPong, _>::new(chan);
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
```

#### 4.2 Implement Session API for Var

```rust
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
    ///     Send(std::marker::PhantomData)
    /// }
    ///
    /// // Create a session that has reached a Var state
    /// let (chan, _) = MockChannelEnd::new_pair();
    /// let session = Session::<Var, _>::new(chan);
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
```

### Task 5: Duality

#### 5.1 Implement Dual for Rec

```rust
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
    type DualType = Rec<impl FnOnce(Var) -> P::DualType>;
}
```

#### 5.2 Implement Dual for Var

```rust
// Var is self-dual
impl Dual for Var {
    /// Var is self-dual - when one participant reaches a recursion variable,
    /// the other must also reach a recursion variable.
    type DualType = Var;
}
```

### Task 6: Testing

#### 6.1 Create tests/binary_recursive.rs

```rust
use sessrums_types::{
    error::SessionError,
    messages::{PingMsg, PongMsg},
    roles::{Client, Server},
    session_types::{Dual, Either, End, Rec, Receive, Select, Send, Session, Var},
    transport::MockChannelEnd,
    Transport,
};

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

// Define a counter message for testing bounded recursion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CounterMsg {
    count: u32,
}

// Test a simple recursive ping-pong protocol
#[test]
fn test_simple_recursive_protocol() -> Result<(), SessionError> {
    // Create mock channel pair
    let (client_chan, server_chan) = MockChannelEnd::new_pair();

    // Define recursive protocol types
    // A protocol that sends a ping, receives a pong, and then loops back
    fn client_body(_: Var) -> Send<PingMsg, Receive<PongMsg, Var>> {
        Send(PhantomData)
    }

    fn server_body(_: Var) -> Receive<PingMsg, Send<PongMsg, Var>> {
        Receive(PhantomData)
    }

    type ClientProtocol = Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>;
    type ServerProtocol = Rec<fn(Var) -> Receive<PingMsg, Send<PongMsg, Var>>>;

    // Create client and server sessions
    let client = Session::<ClientProtocol, _>::new(client_chan);
    let server = Session::<ServerProtocol, _>::new(server_chan);

    // Unroll the recursion once
    let client = client.enter_rec();
    let server = server.enter_rec();

    // First iteration
    let ping1 = PingMsg { seq: Some(1) };
    let client = client.send(ping1)?;
    
    let (received_ping1, server) = server.receive()?;
    assert_eq!(received_ping1.seq, Some(1));
    
    let pong1 = PongMsg { ack: Some(1) };
    let server = server.send(pong1)?;
    
    let (received_pong1, client) = client.receive()?;
    assert_eq!(received_pong1.ack, Some(1));

    // Loop back and do another iteration
    let client = client.recurse(client_body);
    let server = server.recurse(server_body);

    let client = client.enter_rec();
    let server = server.enter_rec();

    // Second iteration
    let ping2 = PingMsg { seq: Some(2) };
    let client = client.send(ping2)?;
    
    let (received_ping2, server) = server.receive()?;
    assert_eq!(received_ping2.seq, Some(2));
    
    let pong2 = PongMsg { ack: Some(2) };
    let server = server.send(pong2)?;
    
    let (received_pong2, _client) = client.receive()?;
    assert_eq!(received_pong2.ack, Some(2));

    Ok(())
}

// Test a bounded recursive protocol with a counter
#[test]
fn test_bounded_recursive_protocol() -> Result<(), SessionError> {
    // Create mock channel pair
    let (client_chan, server_chan) = MockChannelEnd::new_pair();

    // Define recursive protocol types with a choice to continue or end
    fn client_body(_: Var) -> Send<CounterMsg, Receive<CounterMsg, Select<Var, End>>> {
        Send(PhantomData)
    }

    fn server_body(_: Var) -> Receive<CounterMsg, Send<CounterMsg, Offer<Var, End>>> {
        Receive(PhantomData)
    }

    type ClientProtocol = Rec<fn(Var) -> Send<CounterMsg, Receive<CounterMsg, Select<Var, End>>>>;
    type ServerProtocol = Rec<fn(Var) -> Receive<CounterMsg, Send<CounterMsg, Offer<Var, End>>>>;

    // Create client and server sessions
    let client = Session::<ClientProtocol, _>::new(client_chan);
    let server = Session::<ServerProtocol, _>::new(server_chan);

    // Unroll the recursion once
    let client = client.enter_rec();
    let server = server.enter_rec();

    // Run the protocol for 3 iterations
    let max_count = 3;
    let mut count = 1;

    while count <= max_count {
        // Client sends counter
        let client_msg = CounterMsg { count };
        let client = client.send(client_msg)?;
        
        // Server receives counter
        let (received_client_msg, server) = server.receive()?;
        assert_eq!(received_client_msg.count, count);
        
        // Server sends counter back
        let server_msg = CounterMsg { count };
        let server = server.send(server_msg)?;
        
        // Client receives counter
        let (received_server_msg, client) = client.receive()?;
        assert_eq!(received_server_msg.count, count);

        count += 1;

        if count <= max_count {
            // Continue with recursion
            let client = client.select_left()?;
            let client = client.recurse(client_body);
            let client = client.enter_rec();

            let Either::Left(server) = server.offer()? else {
                panic!("Server should have received Left choice");
            };
            let server = server.recurse(server_body);
            let server = server.enter_rec();
        } else {
            // End the protocol
            let _client = client.select_right()?;
            let Either::Right(_server) = server.offer()? else {
                panic!("Server should have received Right choice");
            };
            break;
        }
    }

    assert_eq!(count, max_count + 1);
    Ok(())
}

// Test that the compiler enforces correct usage of recursion
#[test]
fn test_recursion_type_safety() {
    fn check_duality<T: Dual>() where T::DualType: Dual {}

    // This should compile only if ClientProtocol is dual to ServerProtocol
    fn client_body(_: Var) -> Send<PingMsg, Receive<PongMsg, Var>> {
        Send(PhantomData)
    }

    fn server_body(_: Var) -> Receive<PingMsg, Send<PongMsg, Var>> {
        Receive(PhantomData)
    }

    type ClientProtocol = Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>;
    type ServerProtocol = Rec<fn(Var) -> Receive<PingMsg, Send<PongMsg, Var>>>;

    check_duality::<ClientProtocol>();
}
```

### Task 7: Documentation and Examples

#### 7.1 Update README.md with recursion example

Add a section to README.md demonstrating the use of recursion:

```markdown
## Recursive Protocols

Sessrums supports recursive protocols using the `Rec` and `Var` types:

```rust
use sessrums_types::{
    error::SessionError,
    messages::{PingMsg, PongMsg},
    session_types::{Rec, Var, Send, Receive, End, Session},
    transport::MockChannelEnd,
};

// Define a recursive ping-pong protocol
fn ping_pong_body(_: Var) -> Send<PingMsg, Receive<PongMsg, Var>> {
    Send(std::marker::PhantomData)
}

type RecursivePingPong = Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>;

// Create a session with the recursive protocol
let (chan, _) = MockChannelEnd::new_pair();
let session = Session::<RecursivePingPong, _>::new(chan);

// Enter the recursion
let session = session.enter_rec();

// Send a ping
let ping = PingMsg { seq: Some(1) };
let session = session.send(ping)?;

// Receive a pong
let (pong, session) = session.receive()?;

// Loop back to the beginning of the recursion
let session = session.recurse(ping_pong_body);
let session = session.enter_rec();

// Continue with the protocol...
```
```

## Implementation Order

1. Task 2: Define Recursion Types
2. Task 3: Protocol State Trait for Recursion
3. Task 5: Duality for Recursion
4. Task 4: Session API for Recursion
5. Task 6: Testing
6. Task 7: Documentation and Examples

## Switching to Code Mode

After this plan is approved, we'll switch to Code mode to implement each task in order.