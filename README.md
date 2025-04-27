# sessrums: Session Types EZ

A Rust library for asynchronous session types with minimal dependencies, focusing on expressing the process calculus in the types using Rust's type system features, including `const generics`.

## Overview

sessrums implements session types, a type discipline for communication protocols that allows compile-time verification of protocol adherence. This library ensures that communicating parties follow the agreed-upon protocol without runtime errors or deadlocks.

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

- **Send\<T, P\>**: Sends a value of type `T` and then continues with protocol `P`
- **Recv\<T, P\>**: Receives a value of type `T` and then continues with protocol `P`
- **End**: Represents the end of a communication session
- **Offer\<L, R\>**: Offers a choice between continuing with protocol `L` or protocol `R`
- **Choose\<L, R\>**: Makes a choice between continuing with protocol `L` or protocol `R`

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

The `Chan<P, IO>` type represents a communication channel that follows protocol `P` using the underlying IO implementation `IO`:

```rust
pub struct Chan<P: Protocol, IO> {
    io: IO,
    _marker: PhantomData<P>,
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

## Documentation

For a complete list of all documentation resources, see the [Documentation Index](docs/index.md).

- [Detailed Documentation](docs/session-types-documentation.md) - Comprehensive guide to the library
- [Quick Reference Guide](docs/quick-reference.md) - Concise summary of key concepts and API methods
- [Visual Diagrams](docs/session-types-diagrams.md) - Visual representations of session types concepts
- [Error Handling Guide](docs/error-handling.md) - Detailed information about error handling
- [Testing Protocols Guide](docs/testing-protocols.md) - Examples and best practices for testing protocols
- [Offer and Choose Guide](docs/offer-choose.md) - Detailed information about the Offer and Choose protocol types
- [API Ergonomics Guide](docs/api-ergonomics.md) - Guide to using the API ergonomics improvements

## License

This project is licensed under the MIT License - see the LICENSE file for details.