# SEZ: Session Types EZ

A Rust library for asynchronous session types with minimal dependencies, focusing on expressing the process calculus in the types using Rust's type system features, including `const generics`.

## Overview

SEZ implements session types, a type discipline for communication protocols that allows compile-time verification of protocol adherence. This library ensures that communicating parties follow the agreed-upon protocol without runtime errors or deadlocks.

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

SEZ provides the following protocol types:

- **Send\<T, P\>**: Sends a value of type `T` and then continues with protocol `P`
- **Recv\<T, P\>**: Receives a value of type `T` and then continues with protocol `P`
- **End**: Represents the end of a communication session
- **Offer\<L, R\>**: Offers a choice between continuing with protocol `L` or protocol `R`
- **Choose\<L, R\>**: Makes a choice between continuing with protocol `L` or protocol `R`

## Dependencies

SEZ is designed with minimal dependencies:

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

SEZ defines an `Error` enum that represents various error conditions:

```rust
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Protocol(&'static str),
    Connection(&'static str),
    Serialization(&'static str),
    Deserialization(&'static str),
    ChannelClosed,
}
```

## API Reference

### send Method

```rust
// For Chan<Send<T, P>, IO>
pub async fn send(mut self, value: T) -> Result<Chan<P, IO>, Error>
```

Sends a value of type `T` over the channel and advances the protocol from `Send<T, P>` to `P`.

### recv Method

```rust
// For Chan<Recv<T, P>, IO>
pub async fn recv(mut self) -> Result<(T, Chan<P, IO>), Error>
```

Receives a value of type `T` from the channel and advances the protocol from `Recv<T, P>` to `P`.

### close Method

```rust
// For Chan<End, IO>
pub fn close(self) -> Result<(), Error>
```

Closes the channel, indicating that the communication session has ended.

## Example Usage

```rust
// Define the client's protocol: Send a query, receive a response, then end
type ClientProtocol = Send<String, Recv<String, End>>;

// Define the server's protocol: Receive a query, send a response, then end
type ServerProtocol = <ClientProtocol as Protocol>::Dual;

// Client implementation
async fn run_client(chan: Chan<ClientProtocol, BiChannel<String>>) -> Result<(), Error> {
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
async fn run_server(chan: Chan<ServerProtocol, BiChannel<String>>) -> Result<(), Error> {
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

## Documentation

For a complete list of all documentation resources, see the [Documentation Index](docs/index.md).

- [Detailed Documentation](docs/session-types-documentation.md) - Comprehensive guide to the library
- [Quick Reference Guide](docs/quick-reference.md) - Concise summary of key concepts and API methods
- [Visual Diagrams](docs/session-types-diagrams.md) - Visual representations of session types concepts
- [Error Handling Guide](docs/error-handling.md) - Detailed information about error handling
- [Testing Protocols Guide](docs/testing-protocols.md) - Examples and best practices for testing protocols
- [Offer and Choose Guide](docs/offer-choose.md) - Detailed information about the Offer and Choose protocol types

## License

This project is licensed under the MIT License - see the LICENSE file for details.