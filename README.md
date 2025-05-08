# sessrums: Session Types

## Overview

sessrums implements session types, a type discipline for communication protocols that allows compile-time verification of protocol adherence. This library ensures that communicating parties follow the agreed-upon protocol without runtime errors or deadlocks. The project is under active development, with Stages 0-5 completed and further stages planned.

## Core Concepts

### What are Session Types?

Session types are a formal method for describing communication protocols at the type level. They allow you to specify the sequence and types of messages exchanged between communicating parties, ensuring that:

1. Messages are sent and received in the correct order
2. Messages have the expected types
3. The protocol is followed to completion
4. Communication is free from deadlocks and race conditions

### Duality

Duality is a fundamental concept in session types. For every protocol `P`, there exists a dual protocol `P::Dual` that represents the complementary behavior:

- If `P` sends a message of type `T`, then `P::Dual` receives a message of type `T`
- If `P` offers a choice between protocols, then `P::Dual` makes a choice between the dual protocols

```rust
// A protocol that sends an i32, then receives a String, then ends
type ClientProtocol = Send<i32, Recv<String, End>>;

// The dual protocol receives an i32, then sends a String, then ends
type ServerProtocol = <ClientProtocol as Protocol>::Dual;
// Equivalent to: Recv<i32, Send<String, End>>
```

## Protocol Types

sessrums provides the following protocol types:

### Binary Session Types

- **Send\<T, P\>**: Sends a value of type `T` and then continues with protocol `P`
- **Recv\<T, P\>**: Receives a value of type `T` and then continues with protocol `P`
- **End**: Represents the end of a communication session
- **Offer\<L, R\>**: Offers a choice between continuing with protocol `L` or protocol `R`
- **Choose\<L, R\>**: Makes a choice between continuing with protocol `L` or protocol `R`
- **Rec\<P\>**: Represents a recursive protocol definition
- **Var\<N\>**: Represents a reference to a recursive protocol definition

### Multiparty Session Types (MPST)

- **Role**: A trait representing a participant in a multiparty protocol
- **GlobalProtocol**: A trait representing a global protocol that describes the communication behavior between multiple roles
- **GSend\<T, From, To, Next\>**: Represents sending a value of type `T` from role `From` to role `To`, then continuing with protocol `Next`
- **GRecv\<T, From, To, Next\>**: Represents receiving a value of type `T` by role `To` from role `From`, then continuing with protocol `Next`
- **GChoice\<Chooser, Branches\>**: Represents a choice made by role `Chooser` between different protocol branches
- **GOffer\<Offeree, Branches\>**: Represents an offer received by role `Offeree` with different protocol branches
- **GRec\<Label, Protocol\>**: Represents a recursive global protocol definition
- **GVar\<Label\>**: Represents a reference to a recursive global protocol definition
- **GSeq\<First, Second\>**: Represents sequential composition of two protocols
- **GPar\<First, Second\>**: Represents parallel composition of two protocols
- **GEnd**: Represents the end of a global protocol path

Additionally, sessrums provides macros for defining protocols with a more concise syntax:

```rust
use sessrums::protocol;

// Define a protocol using the macro
type ClientProto = protocol!(send(i32) >> recv(String) >> end);
type ServerProto = protocol!(recv(i32) >> send(String) >> end);

// Define a protocol pair using the protocol_pair macro
protocol_pair! {
    pub MyProtocol<Req, Resp> {
        client: send(Req) >> recv(Resp) >> end,
        server: recv(Req) >> send(Resp) >> end
    }
}

// Use the generated type aliases
type MyClient = MyProtocol::Client<String, i32>;
type MyServer = MyProtocol::Server<String, i32>;
```

## Dependencies

sessrums is designed with minimal dependencies:

- **futures-core**: Provides core traits for asynchronous programming, enabling the implementation of async versions of communication traits.
- **serde**: Provides serialization and deserialization capabilities for sending and receiving values over channels.
- **bincode**: Provides binary encoding and decoding for efficient data transmission.

### Dev Dependencies

For development and examples, the following dependencies are used:

- **tokio**: A runtime for asynchronous programming in Rust, used in examples and tests.
- **async-std**: An asynchronous runtime and utilities, providing an alternative to tokio for examples.
- **trybuild**: A tool for testing compile failures, used for testing the type system.

## Channel Implementation

### Binary Session Types

The `Chan<P, R, IO>` type represents a communication channel that follows protocol `P` from the perspective of role `R` using the underlying IO implementation `IO`:

```rust
pub struct Chan<P: Protocol, R: Role, IO> {
    io: IO,
    role: R,
    _marker: PhantomData<P>,
}
```

### Multiparty Session Types

For multiparty session types, the `Chan` type is used with local protocols that are projected from global protocols:

```rust
// Define a global protocol: RoleA sends a String to RoleB, then ends
type GlobalProtocol = GSend<String, RoleA, RoleB, GEnd>;

// Project it for RoleA
type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;

// Create a channel for RoleA
let chan = Chan::<RoleALocal, RoleA, _>::new(io);
```

The `Project` trait is used to project a global protocol to a local protocol for a specific role:

```rust
pub trait Project<R: Role> {
    /// The resulting local protocol type after projection.
    type LocalProtocol;
}
```

The `ProtocolCompat` trait allows for seamless conversion between binary and multiparty session types:

```rust
pub trait ProtocolCompat<R: Role> {
    /// The binary protocol type that is compatible with this multiparty protocol.
    type BinaryProtocol: Protocol;
    
    /// Converts a multiparty protocol to a binary protocol.
    fn to_binary(self) -> Self::BinaryProtocol;
    
    /// Converts a binary protocol to a multiparty protocol.
    fn from_binary(binary: Self::BinaryProtocol) -> Self;
}
```

## Error Handling

sessrums defines an `Error` enum that represents various error conditions:

```rust
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Protocol(&'static str),
    Connection(&'static str),
    Serialization(&'static str),
    Deserialization(&'static str),
    ChannelClosed,
    Timeout(std::time::Duration),
    Negotiation(&'static str),
    StateMismatch(&'static str),
}
```

The library also provides a `Result` type alias for convenience:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## API Reference

### Core Channel Methods

#### send Method

```rust
// For Chan<Send<T, P>, IO>
pub async fn send(mut self, value: T) -> Result<Chan<P, IO>, Error>
```

Sends a value of type `T` over the channel and advances the protocol from `Send<T, P>` to `P`.

#### recv Method

```rust
// For Chan<Recv<T, P>, IO>
pub async fn recv(mut self) -> Result<(T, Chan<P, IO>), Error>
```

Receives a value of type `T` from the channel and advances the protocol from `Recv<T, P>` to `P`.

#### choose_left and choose_right Methods

```rust
// For Chan<Choose<L, R>, IO>
pub fn choose_left(self) -> (Chan<L, IO>, ChoiceTag)
pub fn choose_right(self) -> (Chan<R, IO>, ChoiceTag)
```

Makes a choice between the left or right protocol option.

#### offer Method

```rust
// For Chan<Offer<L, R>, IO>
pub async fn offer<T, F1, F2, Fut1, Fut2>(
    self,
    left_handler: F1,
    right_handler: F2,
) -> Result<T, Error>
where
    F1: FnOnce(Chan<L, IO>) -> Fut1,
    F2: FnOnce(Chan<R, IO>) -> Fut2,
    Fut1: Future<Output = Result<T, Error>>,
    Fut2: Future<Output = Result<T, Error>>,
```

Offers a choice between two protocol continuations and handles each case with the provided handlers.

#### close Method

```rust
// For Chan<End, IO>
pub fn close(self) -> Result<(), Error>
```

Closes the channel, indicating that the communication session has ended.

### API Ergonomics

sessrums provides several type aliases and helper functions to improve API ergonomics:

#### Type Aliases

```rust
// Request-response protocol (client side)
pub type RequestClient<Req, Resp> = Send<Req, Recv<Resp, End>>;

// Request-response protocol (server side)
pub type RequestServer<Req, Resp> = Recv<Req, Send<Resp, End>>;

// Ping-pong protocol (client side)
pub type PingClient<Ping, Pong> = Send<Ping, Recv<Pong, End>>;

// Ping-pong protocol (server side)
pub type PingServer<Ping, Pong> = Recv<Ping, Send<Pong, End>>;

// Choice protocol (client side)
pub type ChoiceClient<P1, P2> = Choose<P1, P2>;

// Choice protocol (server side)
pub type OfferServer<P1, P2> = Offer<P1, P2>;
```

#### Helper Functions

```rust
// Create a pair of channels with dual protocols
pub fn channel_pair<P, IO>() -> (Chan<P, IO>, Chan<P::Dual, IO>)
where
    P: Protocol,
    IO: Default + Clone;

// Create a request-response channel pair
pub fn request_response_pair<Req, Resp, IO>() -> (Chan<RequestClient<Req, Resp>, IO>, Chan<RequestServer<Req, Resp>, IO>)
where
    IO: Default + Clone;

// Create a ping-pong channel pair
pub fn ping_pong_pair<Ping, Pong, IO>() -> (Chan<PingClient<Ping, Pong>, IO>, Chan<PingServer<Ping, Pong>, IO>)
where
    IO: Default + Clone;

// Establish a connection with a specific protocol
pub async fn connect_with_protocol<P, IO, C>(conn_info: C) -> Result<Chan<P, IO>>
where
    P: Protocol,
    IO: Default,
    C: connect::ConnectInfo<IO>;
```

## Example Usage

### Basic Example

```rust
use sessrums::proto::{Send, Recv, End};
use sessrums::chan::Chan;
use sessrums::error::Result;

// Define the client's protocol: Send a query, receive a response, then end
type ClientProtocol = Send<String, Recv<String, End>>;

// Define the server's protocol: Receive a query, send a response, then end
type ServerProtocol = <ClientProtocol as Protocol>::Dual;

// Client implementation
async fn run_client(chan: Chan<ClientProtocol, BiChannel<String>>) -> Result<()> {
    // Send a query
    let query = "What is the meaning of life?".to_string();
    let chan = chan.send(query).await?;
    
    // Receive the response
    let (response, chan) = chan.recv().await?;
    println!("Client received: {}", response);
    
    // Close the channel
    chan.close()?;
    
    Ok(())
}

// Server implementation
async fn run_server(chan: Chan<ServerProtocol, BiChannel<String>>) -> Result<()> {
    // Receive the query
    let (query, chan) = chan.recv().await?;
    println!("Server received: {}", query);
    
    // Process and send response
    let response = "42".to_string();
    let chan = chan.send(response).await?;
    
    // Close the channel
    chan.close()?;
    
    Ok(())
}
```

### Using API Ergonomics

```rust
use sessrums::api::{RequestClient, RequestServer, request_response_pair};
use sessrums::error::Result;

// Use type aliases for request-response protocol
type MyClient = RequestClient<String, i32>;
type MyServer = RequestServer<String, i32>;

// Create a pair of channels
let (client_chan, server_chan) = request_response_pair::<String, i32, ()>();

// Client implementation
async fn run_client(chan: Chan<MyClient, IO>) -> Result<()> {
    // Send a request
    let request = "Hello, server!".to_string();
    let chan = chan.send(request).await?;
    
    // Receive the response
    let (response, chan) = chan.recv().await?;
    println!("Client received: {}", response);
    
    // Close the channel
    chan.close()?;
    
    Ok(())
}

// Server implementation
async fn run_server(chan: Chan<MyServer, IO>) -> Result<()> {
    // Receive the request
    let (request, chan) = chan.recv().await?;
    println!("Server received: {}", request);
    
    // Process and send response
    let response = 42;
    let chan = chan.send(response).await?;
    
    // Close the channel
    chan.close()?;
    
    Ok(())
}
```

### Using Protocol Macros

```rust
use sessrums::protocol;
use sessrums::protocol_pair;
use sessrums::chan::Chan;
use sessrums::error::Result;

// Define protocol types using the macro
type MyClient = protocol!(send(String) >> recv(i32) >> end);
type MyServer = protocol!(recv(String) >> send(i32) >> end);

// Define a protocol pair using the protocol_pair macro
protocol_pair! {
    pub MyProtocol<Req, Resp> {
        client: send(Req) >> recv(Resp) >> end,
        server: recv(Req) >> send(Resp) >> end
    }
}

// Use the generated type aliases
type ClientProto = MyProtocol::Client<String, i32>;
type ServerProto = MyProtocol::Server<String, i32>;

// Create channels
let client_chan = Chan::<ClientProto, _>::new(());
let server_chan = Chan::<ServerProto, _>::new(());
```

### Using Offer and Select for Protocol Branching

```rust
use sessrums_types::{
    session_types::{
        binary::{Offer, Select, Either},
        End, Send, Receive, Session,
    },
    transport::MockChannelEnd,
    error::SessionError,
};
use serde::{Serialize, Deserialize};

// Define message types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Proposal(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Acceptance(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Rejection(String, String); // Rejection with reason

// Define protocol types for a proposal that can be accepted or rejected
// Client sends a proposal, then server can either:
// - Accept (left branch): Server sends an acceptance message
// - Reject (right branch): Server sends a rejection message with reason
type ClientProtocol = Send<Proposal,
    Offer<
        Receive<Acceptance, End>,  // Left branch - proposal accepted
        Receive<Rejection, End>    // Right branch - proposal rejected
    >
>;

// Server's dual protocol
type ServerProtocol = Receive<Proposal,
    Select<
        Send<Acceptance, End>,     // Left branch - accept proposal
        Send<Rejection, End>       // Right branch - reject proposal
    >
>;

// Example usage
fn main() -> Result<(), SessionError> {
    // Create mock transport
    let (client_chan, server_chan) = MockChannelEnd::new_pair();
    
    // Initialize sessions
    let client = Session::<ClientProtocol, _>::new(client_chan);
    let server = Session::<ServerProtocol, _>::new(server_chan);
    
    // Client sends proposal
    let proposal = Proposal("Let's meet at 2pm".to_string());
    let client = client.send(proposal)?;
    
    // Server receives proposal
    let (received_proposal, server) = server.receive()?;
    println!("Server received proposal: {:?}", received_proposal);
    
    // Server decides whether to accept or reject
    // For this example, let's say the server rejects
    let server = server.select_right()?;
    
    // Server sends rejection with reason
    let rejection = Rejection(
        received_proposal.0,
        "I'm not available at that time".to_string()
    );
    let server = server.send(rejection)?;
    
    // Client waits for server's decision
    let client_branch = client.offer()?;
    
    // Client handles the server's decision
    match client_branch {
        Either::Left(client) => {
            // Proposal was accepted
            let (acceptance, client) = client.receive()?;
            println!("Proposal accepted: {:?}", acceptance);
            client.close(); // Close session
        },
        Either::Right(client) => {
            // Proposal was rejected
            let (rejection, client) = client.receive()?;
            println!("Proposal rejected: {:?}", rejection);
            client.close(); // Close session
        }
    }
    
    // Close server session
    server.close();
    
    Ok(())
}
```

This example demonstrates:
1. How to define protocols with branching using `Offer` and `Select`
2. How the client offers choices and the server selects a branch
3. How to handle different branches with pattern matching
4. How the typestate system ensures protocol adherence
5. A practical use case for protocol branching (proposal acceptance/rejection)

### Using Recursion for Repeated Interactions

```rust
use sessrums_types::{
    session_types::{Rec, Var, Send, Receive, Select, Offer, End, Either, Session},
    transport::MockChannelEnd,
    error::SessionError,
};
use serde::{Serialize, Deserialize};
use std::marker::PhantomData;

// Define message types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CounterMsg {
    count: u32,
}

// Define recursive protocol types with a choice to continue or end
fn client_body(_: Var) -> Send<CounterMsg, Receive<CounterMsg, Select<Var, End>>> {
    Send(PhantomData)
}

fn server_body(_: Var) -> Receive<CounterMsg, Send<CounterMsg, Offer<Var, End>>> {
    Receive(PhantomData)
}

type ClientProtocol = Rec<fn(Var) -> Send<CounterMsg, Receive<CounterMsg, Select<Var, End>>>>;
type ServerProtocol = Rec<fn(Var) -> Receive<CounterMsg, Send<CounterMsg, Offer<Var, End>>>>;

// Example usage
fn main() -> Result<(), SessionError> {
    // Create mock transport
    let (client_chan, server_chan) = MockChannelEnd::new_pair();
    
    // Initialize sessions
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
        println!("Server received: count = {}", received_client_msg.count);
        
        // Server sends counter back
        let server_msg = CounterMsg { count };
        let server = server.send(server_msg)?;
        
        // Client receives counter
        let (received_server_msg, client) = client.receive()?;
        println!("Client received: count = {}", received_server_msg.count);

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
    
    Ok(())
}
```

This example demonstrates a recursive protocol where:
1. The client sends a counter message to the server
2. The server receives the counter and sends it back
3. The client receives the counter from the server
4. The client decides whether to continue (recursion) or end the protocol
5. If continuing, both parties loop back to step 1 using `recurse` and `enter_rec`

## Visual Protocol Representation

```
Client                                Server
  |                                     |
  |------- Send(String) - Query ------->|
  |                                     |
  |<------ Recv(String) - Response -----|
  |                                     |
  |-------------- End ---------------->|
```

For more visual diagrams illustrating session types concepts, see [Session Types Diagrams](docs/session-types-diagrams.md).

## Multiparty Session Types Examples

### Basic MPST Example

```rust
// Define the roles in our protocol
struct Client;
struct Server;
struct Logger;

impl Role for Client {
    fn name(&self) -> &'static str { "Client" }
}

impl Role for Server {
    fn name(&self) -> &'static str { "Server" }
}

impl Role for Logger {
    fn name(&self) -> &'static str { "Logger" }
}

// Define the global protocol
// Client sends a Request to Server
// Server sends a Response to Client
// Server sends a LogMessage to Logger
type GlobalProtocol = GSend<Request, Client, Server,
    GSend<Response, Server, Client,
        GSend<LogMessage, Server, Logger, GEnd>
    >
>;

// Project the global protocol to local protocols for each role
type ClientProtocol = <GlobalProtocol as Project<Client>>::LocalProtocol;
type ServerProtocol = <GlobalProtocol as Project<Server>>::LocalProtocol;
type LoggerProtocol = <GlobalProtocol as Project<Logger>>::LocalProtocol;

// Create channels for each role
let client_chan = Chan::<ClientProtocol, Client, _>::new(client_io);
let server_chan = Chan::<ServerProtocol, Server, _>::new(server_io);
let logger_chan = Chan::<LoggerProtocol, Logger, _>::new(logger_io);
```

### Using the Global Protocol Macro

```rust
// Define the global protocol using the macro
global_protocol! {
    protocol OnlineStore {
        // Client sends a request to Server
        Client -> Server: Request;
        
        // Server makes a choice
        choice at Server {
            // Success branch
            option Success {
                // Server sends a response to Client
                Server -> Client: Response;
                // Server logs the successful transaction
                Server -> Logger: LogMessage;
            }
            // Error branch
            option Error {
                // Server sends an error response to Client
                Server -> Client: Response;
                // Server logs the error
                Server -> Logger: LogMessage;
            }
        }
    }
}

// Project the global protocol to local protocols for each role
type ClientProtocol = <OnlineStore as Project<Client>>::LocalProtocol;
type ServerProtocol = <OnlineStore as Project<Server>>::LocalProtocol;
type LoggerProtocol = <OnlineStore as Project<Logger>>::LocalProtocol;
```

For more examples of multiparty session types, see the [MPST Examples](examples/mpst/) directory.

## Testing Framework

sessrums provides a comprehensive testing framework for verifying both compile-time and runtime properties of session types:

### Compile-Time Tests

Compile-time tests verify that the type system correctly enforces protocol adherence:

```rust
// Verify that a type implements the Protocol trait
fn assert_protocol<P: Protocol>() {}

// Verify that two types have the correct duality relationship
fn assert_dual<P: Protocol, Q: Protocol>()
where
    P::Dual: Protocol,
    Q: Protocol<Dual = P>,
    P: Protocol<Dual = Q>,
{}

// Verify that a type is its own dual
fn assert_self_dual<P: Protocol>()
where
    P::Dual: Protocol<Dual = P>,
    P: Protocol<Dual = P::Dual>,
{}
```

### Runtime Tests

Runtime tests verify the behavior of protocols during execution:

```rust
// Create a mock channel for testing
fn mock_channel<P: Protocol, IO>() -> Chan<P, IO>
where
    IO: Default,
{
    Chan::new(IO::default())
}

// Test sending and receiving messages
#[tokio::test]
async fn test_send_recv() {
    let client_chan = Chan::<Send<i32, End>, _>::new(());
    let client_chan = client_chan.send(42).await.unwrap();
    client_chan.close().unwrap();
    
    let server_chan = Chan::<Recv<i32, End>, _>::new(());
    let (value, server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(value, 42);
    server_chan.close().unwrap();
}
```

### Compile-Fail Tests

sessrums uses the `trybuild` crate to verify that invalid protocols fail to compile with the expected error messages:

```rust
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
```

## Current Implementation Status

The project is being developed in stages as outlined in the implementation plan. Here's the current status:

### Stage 0: Foundational Binary Session Types (Completed)

- Core session type structures: `End`, `Send<M, NextP>`, `Receive<M, NextP>`
- `Session<CurrentState, T: Transport>` struct with typestate pattern
- Transport abstraction with `MockChannelEnd` implementation
- Basic error handling
- Tests for simple ping-pong protocols

### Stage 1: Binary Session Types with External Choice (Completed)

- External choice structures: `Select<L, R>` and `Offer<L, R>`
- `ChoiceSignal` enum for signaling choice over transport
- `Either<L, R>` enum for handling the result of an offer
- Tests for choice protocols, including nested choices

### Stage 2: Binary Session Types with Recursion (Completed)

- Recursion structures: `Rec<F>` and `Var`
- Fixed-point combinator style for recursive protocol definitions
- Session methods for handling recursion (`enter_rec`, `recurse`)
- Tests for recursive protocols, including bounded recursion with choice

## Next Development Steps

Based on the implementation plan, the following stages are planned:

### Stage 3: Basic Multiparty Primitives & Manual Global/Local Types (Completed)

- Define `GlobalInteraction` (Message, End) and `LocalProtocol` (Send, Receive, End) enums
- Implement `RoleIdentifier` and `Participant<R: Role>` structures
- Create `MultipartyTransport` trait and `MockMultipartyBroker` for managing multiple channels
- Manually write global and local protocols for 3+ party interactions

### Stage 4: Automated Projection (Message, End, Choice) (Completed)

- Implement `Project<R: Role>` trait for automated projection from global to local protocols
- Extend `GlobalInteraction` with `Choice` variant for branching protocols
- Extend `LocalProtocol` with `Select` and `Offer` variants for role-specific branching
- Implement projection algorithm for message, end, and choice constructs
- Add comprehensive tests for basic projection and choice projection
- Add comprehensive tests for basic projection and choice projection

### Stage 5: Projection for Recursion & Full Multiparty Types (Completed)

- Extend `GlobalInteraction` and `LocalProtocol` with `Rec` and `Var` variants for recursion
- Implement projection algorithm for recursive protocols
- Add well-formedness checking for recursive protocols
- Create builder methods for recursive protocols
- Add comprehensive tests for recursive protocols, including with choice

### Stage 6: Multiparty Session Runtime

- Develop `MultipartySession<MyRole, CurrentLocalState, AllChannels>` struct
- Implement methods for multiparty communication
- Test with various projected protocols

### Stage 7-10: DSL Development and Integration

#### Stage 7: DSL Macro System (Completed)

The DSL macro system provides a concise, readable syntax for defining multiparty session type protocols. It includes:

- The `mpst!` macro for defining global protocols using a Mermaid-like syntax
- The `project!` macro for projecting global protocols to local protocols
- Comprehensive error reporting for syntax and semantic errors
- Support for complex protocol patterns including choice and recursion

For detailed documentation on the DSL macro system, see:
- [MPST DSL Documentation](docs/mpst-dsl.md)
- [Example: Simple Ping-Pong Protocol](docs/examples/ping-pong.md)
- [Example: File Transfer with Choice](docs/examples/file-transfer.md)
- [Example: Data Streaming with Recursion](docs/examples/data-stream.md)

#### Future Stages

- Connect all components for end-to-end integration
- Add advanced features like parallel composition
- Enhance error reporting and diagnostics
- Optimize protocol validation and code generation

For more details on the implementation plan, see the [MPST_DSL-Review AndImplementation.md](docs/chats/MPST_DSL-Review%20AndImplementation.md) document.

## Documentation

For a complete list of all documentation resources, see the [Documentation Index](docs/index.md).

- [Detailed Documentation](docs/session-types-documentation.md) - Comprehensive guide to the library
- [Quick Reference Guide](docs/quick-reference.md) - Concise summary of key concepts and API methods
- [Visual Diagrams](docs/session-types-diagrams.md) - Visual representations of session types concepts
- [Error Handling Guide](docs/error-handling.md) - Detailed information about error handling
- [Testing Protocols Guide](docs/testing-protocols.md) - Examples and best practices for testing protocols
- [Offer and Choose Guide](docs/offer-choose.md) - Detailed information about the Offer and Choose protocol types
- [API Ergonomics Guide](docs/api-ergonomics.md) - Guide to using the API ergonomics improvements
- [MPST Concepts](docs/mpst-concepts.md) - Introduction to Multiparty Session Types concepts
- [MPST Design](docs/mpst-design.md) - Design and architecture of MPST support
- [MPST Macro](docs/mpst-macro.md) - Guide to using the global protocol macro
- [MPST DSL](docs/mpst-dsl.md) - Comprehensive documentation for the DSL macro system

### DSL Macro System

The DSL macro system provides a concise, readable syntax for defining multiparty session type protocols. It transforms textual protocol definitions into Rust code at compile time, ensuring type safety while offering an intuitive interface for protocol specification.

#### Key Features

- **Mermaid-like Syntax**: Define protocols using a familiar sequence diagram-like syntax
- **Compile-time Verification**: Protocol errors are caught during compilation
- **Comprehensive Error Reporting**: Clear, actionable error messages for syntax and semantic issues
- **Support for Complex Patterns**: Define protocols with choice, recursion, and nested structures
- **Seamless Integration**: Works with the existing MPST type system and projection mechanism

#### Basic Example

```rust
use sessrums_macro::{mpst, project};
use sessrums_types::roles::{Client, Server};

// Define a simple protocol
mpst! {
    protocol PingPong {
        participant Client;
        participant Server;
        
        Client -> Server: String;
        Server -> Client: String;
        end;
    }
}

// Project the global protocol to local protocols
type ClientProtocol = project!(PingPong, Client, String);
type ServerProtocol = project!(PingPong, Server, String);
```

#### Supported Constructs

The DSL supports the following protocol constructs:

- **Message Passing**: `A -> B: T;` - Role A sends a message of type T to role B
- **Choice**: `choice at A { option B1 { ... } or { ... } }` - Role A makes a choice between different branches
- **Recursion**: `rec Loop { ... continue Loop; }` - Define recursive protocols with labeled blocks
- **End**: `end;` - Mark the end of a protocol path

#### Advanced Example with Choice and Recursion

```rust
mpst! {
    protocol FileTransfer {
        participant Client;
        participant Server;
        
        rec Loop {
            Client -> Server: FileName;
            
            choice at Server {
                option FileExists {
                    Server -> Client: FileContent;
                    continue Loop;
                }
                or {
                    Server -> Client: FileNotFound;
                    end;
                }
            }
        }
    }
}
```

For more examples and detailed documentation, see the [MPST DSL Documentation](docs/mpst-dsl.md).

## License

This project is licensed under the MIT License - see the LICENSE file for details.