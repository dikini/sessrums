# SEZ: Session Types EZ

A Rust library for asynchronous session types with minimal dependencies, focusing on expressing the process calculus in the types using Rust's type system features, including `const generics`.

## Table of Contents

1. [Introduction](#introduction)
2. [Core Concepts](#core-concepts)
   - [What are Session Types?](#what-are-session-types)
   - [Duality](#duality)
   - [Type Safety](#type-safety)
   - [Protocol Composition](#protocol-composition)
3. [Visual Diagrams](#visual-diagrams)
4. [Quick Reference](#quick-reference)
5. [Protocol Types](#protocol-types)
   - [Send](#send)
   - [Recv](#recv)
   - [End](#end)
   - [Offer & Choose](#offer--choose)
6. [Channel Implementation](#channel-implementation)
   - [Chan Type](#chan-type)
   - [IO Abstraction](#io-abstraction)
7. [Error Handling](#error-handling)
   - [Error Type](#error-type)
   - [Error Variants](#error-variants)
8. [API Reference](#api-reference)
   - [send Method](#send-method)
   - [recv Method](#recv-method)
   - [close Method](#close-method)
9. [Usage Examples](#usage-examples)
   - [Simple Client-Server Protocol](#simple-client-server-protocol)
   - [Error Handling](#error-handling-example)
   - [Type Safety Examples](#type-safety-examples)
10. [Visual Protocol Representation](#visual-protocol-representation)
11. [Advanced Topics](#advanced-topics)
    - [Custom IO Implementations](#custom-io-implementations)
    - [Protocol Testing](#protocol-testing)

## Introduction

SEZ (Session Types EZ) is a Rust library that implements session types, a type discipline for communication protocols that allows compile-time verification of protocol adherence. This library focuses on expressing the process calculus in the types using Rust's type system features, with minimal dependencies.

Session types provide a way to specify and verify communication protocols at compile time, ensuring that communicating parties follow the agreed-upon protocol without runtime errors or deadlocks.

## Core Concepts

### What are Session Types?

Session types are a formal method for describing communication protocols at the type level. They allow you to specify the sequence and types of messages exchanged between communicating parties, ensuring that:

1. Messages are sent and received in the correct order
2. Messages have the expected types
3. The protocol is followed to completion
4. Communication is free from deadlocks and race conditions

In SEZ, session types are represented as Rust types that describe the communication behavior of a channel. These types are composed of primitive protocol types like `Send<T, P>`, `Recv<T, P>`, and `End`.

### Duality

Duality is a fundamental concept in session types. For every protocol `P`, there exists a dual protocol `P::Dual` that represents the complementary behavior. For example:

- If `P` sends a message of type `T`, then `P::Dual` receives a message of type `T`.
- If `P` offers a choice between protocols, then `P::Dual` makes a choice between the dual protocols.

This duality ensures that when two parties follow dual protocols, their communication is guaranteed to be compatible and free from communication errors like deadlocks or protocol violations.

```rust
// A protocol that sends an i32, then receives a String, then ends
type ClientProtocol = Send<i32, Recv<String, End>>;

// The dual protocol receives an i32, then sends a String, then ends
type ServerProtocol = <ClientProtocol as Protocol>::Dual;
// Equivalent to: Recv<i32, Send<String, End>>
```

### Type Safety

Session types leverage Rust's type system to ensure protocol adherence at compile time. This means that protocol violations are caught as type errors during compilation, preventing runtime communication errors.

For example, if a protocol specifies that a channel should first send an `i32` and then receive a `String`, attempting to receive before sending or sending a value of the wrong type will result in a compile-time error.

### Protocol Composition

Session types can be composed to build complex communication patterns. The primitive protocol types can be nested to create protocols of arbitrary complexity:

```rust
// A protocol that sends an i32, receives a bool, then sends a String, then ends
type ComplexProtocol = Send<i32, Recv<bool, Send<String, End>>>;
```

## Visual Diagrams

Visual representations of session types concepts are available in the [Session Types Diagrams](session-types-diagrams.md) document. These diagrams illustrate:

- Protocol communication flow
- Protocol type composition
- Duality relationships
- Channel state transitions
- Client-server interaction patterns
- Complex protocols with choices

The diagrams provide a visual way to understand how session types work and how they ensure type-safe communication.

## Quick Reference

A concise summary of the key concepts and API methods is available in the [Quick Reference Guide](quick-reference.md). This guide provides a quick overview of:

- Protocol types and their duals
- Channel API methods
- Error handling
- Common protocol patterns
- IO implementation
- Type safety examples

This is particularly useful for developers who are already familiar with the library and just need a quick reminder of how to use it.

## Protocol Types

### Send

The `Send<T, P>` type represents a protocol that sends a value of type `T` and then continues with protocol `P`.

```rust
pub struct Send<T, P> {
    _marker: PhantomData<(T, P)>,
}

impl<T, P: Protocol> Protocol for Send<T, P> {
    type Dual = Recv<T, P::Dual>;
}
```

The dual of `Send<T, P>` is `Recv<T, P::Dual>`, which represents receiving a value of type `T` and then continuing with the dual of protocol `P`.

### Recv

The `Recv<T, P>` type represents a protocol that receives a value of type `T` and then continues with protocol `P`.

```rust
pub struct Recv<T, P> {
    _marker: PhantomData<(T, P)>,
}

impl<T, P: Protocol> Protocol for Recv<T, P> {
    type Dual = Send<T, P::Dual>;
}
```

The dual of `Recv<T, P>` is `Send<T, P::Dual>`, which represents sending a value of type `T` and then continuing with the dual of protocol `P`.

### End

The `End` type represents the end of a communication session. It is a terminal protocol that indicates no further communication will occur.

```rust
pub struct End;

impl Protocol for End {
    type Dual = End;
}
```

The dual of `End` is `End` itself, as ending a session is symmetric for both parties involved in the communication.

### Offer & Choose

The `Offer<L, R>` and `Choose<L, R>` types represent protocols for making and offering choices between two possible continuations.

- `Offer<L, R>` represents a protocol that offers a choice between continuing with protocol `L` or protocol `R`.
- `Choose<L, R>` represents a protocol that makes a choice between continuing with protocol `L` or protocol `R`.

The dual of `Offer<L, R>` is `Choose<L::Dual, R::Dual>`, and the dual of `Choose<L, R>` is `Offer<L::Dual, R::Dual>`.

For detailed information about the Offer and Choose protocol types, including API methods and usage examples, see the [Offer and Choose Guide](offer-choose.md).

## Channel Implementation

### Chan Type

The `Chan<P, IO>` type represents a communication channel that follows protocol `P` using the underlying IO implementation `IO`.

```rust
pub struct Chan<P: Protocol, IO> {
    io: IO,
    _marker: PhantomData<P>,
}
```

The `Chan` type is parameterized by:
- `P`: The protocol type that this channel follows. Must implement the `Protocol` trait.
- `IO`: The underlying IO implementation that handles the actual communication.

### IO Abstraction

The library provides a set of traits that abstract over different IO implementations:

- `Sender<T>`: A trait for sending values of type `T`.
- `Receiver<T>`: A trait for receiving values of type `T`.

These traits allow the session type system to work with various IO implementations, from simple in-memory channels to network sockets.

```rust
pub trait Sender<T> {
    type Error;
    fn send(&mut self, value: T) -> Result<(), Self::Error>;
}

pub trait Receiver<T> {
    type Error;
    fn recv(&mut self) -> Result<T, Self::Error>;
}
```

## Error Handling

The library provides a comprehensive error handling system through the `Error` enum. For detailed information about error handling, including error variants, handling patterns, and best practices, see the [Error Handling Guide](error-handling.md).

### Error Type

The library defines an `Error` enum that represents the various error conditions that might arise when using session-typed channels for communication.

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

### Error Variants

- **Io**: An error occurred in the underlying IO implementation.
- **Protocol**: A protocol violation occurred, such as unexpected messages or type mismatches.
- **Connection**: A connection error occurred during establishment or termination.
- **Serialization**: An error occurred when serializing data to be sent over the channel.
- **Deserialization**: An error occurred when deserializing received data.
- **ChannelClosed**: The channel was closed when attempting to communicate.

For more detailed information about error handling, including examples and best practices, see the [Error Handling Guide](error-handling.md).

## API Reference

### send Method

The `send` method is implemented for `Chan<Send<T, P>, IO>` and allows sending a value of type `T` over the channel.

```rust
impl<T, P: Protocol, IO> Chan<crate::proto::Send<T, P>, IO>
where
    IO: crate::io::Sender<T>,
    <IO as crate::io::Sender<T>>::Error: std::fmt::Debug,
{
    pub async fn send(mut self, value: T) -> Result<Chan<P, IO>, crate::error::Error> {
        // Send the value using the underlying IO implementation
        self.io_mut().send(value).map_err(|e| {
            // Convert the IO-specific error to our Error type
            crate::error::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Send error: {:?}", e),
            ))
        })?;

        // Return a new channel with the advanced protocol
        Ok(Chan {
            io: self.io,
            _marker: std::marker::PhantomData,
        })
    }
}
```

This method consumes the channel and returns a new channel with the advanced protocol. The protocol advances from `Send<T, P>` to `P` after sending the value.

#### Example

```rust
// Create a channel with a Send<i32, End> protocol
let chan = Chan::<Send<i32, End>, _>::new(io);

// Send a value and advance the protocol to End
let chan = chan.send(42).await?;
```

### recv Method

The `recv` method is implemented for `Chan<Recv<T, P>, IO>` and allows receiving a value of type `T` from the channel.

```rust
impl<T, P: Protocol, IO> Chan<crate::proto::Recv<T, P>, IO>
where
    IO: crate::io::Receiver<T>,
    <IO as crate::io::Receiver<T>>::Error: std::fmt::Debug,
{
    pub async fn recv(mut self) -> Result<(T, Chan<P, IO>), crate::error::Error> {
        // Receive the value using the underlying IO implementation
        let value = self.io_mut().recv().map_err(|e| {
            // Convert the IO-specific error to our Error type
            crate::error::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Receive error: {:?}", e),
            ))
        })?;

        // Return the received value and a new channel with the advanced protocol
        Ok((
            value,
            Chan {
                io: self.io,
                _marker: std::marker::PhantomData,
            },
        ))
    }
}
```

This method consumes the channel and returns the received value along with a new channel with the advanced protocol. The protocol advances from `Recv<T, P>` to `P` after receiving the value.

#### Example

```rust
// Create a channel with a Recv<i32, End> protocol
let chan = Chan::<Recv<i32, End>, _>::new(io);

// Receive a value and advance the protocol to End
let (value, chan) = chan.recv().await?;
assert_eq!(value, 42);
```

### close Method

The `close` method is implemented for `Chan<End, IO>` and closes the channel, indicating that the communication session has ended.

```rust
impl<IO> Chan<crate::proto::End, IO> {
    pub fn close(self) -> Result<(), crate::error::Error> {
        // The End protocol doesn't require any specific action to close
        // We just consume the channel and return Ok(())
        Ok(())
    }
}
```

This method consumes the channel and returns nothing on success, indicating that the protocol has been completed successfully.

#### Example

```rust
// Create a channel with an End protocol
let chan = Chan::<End, _>::new(io);

// Close the channel
chan.close()?;
```

## Usage Examples

### Simple Client-Server Protocol

This example demonstrates a simple client-server interaction using session types:

```rust
// Define the client's protocol: Send a query, receive a response, then end
type ClientProtocol = Send<String, Recv<String, End>>;

// Define the server's protocol: Receive a query, send a response, then end
// Note: This is the dual of the client's protocol
type ServerProtocol = <ClientProtocol as Protocol>::Dual;

// Implements the client side of the protocol
async fn run_client(chan: Chan<ClientProtocol, BiChannel<String>>) -> Result<(), Error> {
    println!("Client: Starting protocol");
    
    // Step 1: Send a query to the server
    println!("Client: Sending query");
    let query = "What is the meaning of life?".to_string();
    let chan = chan.send(query).await?;
    println!("Client: Query sent successfully");
    
    // Step 2: Receive the response from the server
    println!("Client: Waiting for response");
    let (response, chan) = chan.recv().await?;
    println!("Client: Received response: {}", response);
    
    // Step 3: Close the channel (end the protocol)
    println!("Client: Closing channel");
    chan.close()?;
    println!("Client: Protocol completed successfully");
    
    Ok(())
}

// Implements the server side of the protocol
async fn run_server(chan: Chan<ServerProtocol, BiChannel<String>>) -> Result<(), Error> {
    println!("Server: Starting protocol");
    
    // Step 1: Receive a query from the client
    println!("Server: Waiting for query");
    let (query, chan) = chan.recv().await?;
    println!("Server: Received query: {}", query);
    
    // Step 2: Process the query and send a response
    println!("Server: Processing query");
    let response = process_query(&query);
    println!("Server: Sending response");
    let chan = chan.send(response).await?;
    println!("Server: Response sent successfully");
    
    // Step 3: Close the channel (end the protocol)
    println!("Server: Closing channel");
    chan.close()?;
    println!("Server: Protocol completed successfully");
    
    Ok(())
}
```

### Error Handling Example

This example demonstrates how to handle errors when using session-typed channels:

```rust
// Create a custom IO implementation that fails on recv
struct FailingIO;

impl Sender<String> for FailingIO {
    type Error = ();
    
    fn send(&mut self, _value: String) -> Result<(), Self::Error> {
        println!("Send operation succeeded");
        Ok(())
    }
}

impl Receiver<String> for FailingIO {
    type Error = ();
    
    fn recv(&mut self) -> Result<String, Self::Error> {
        println!("Receive operation failed");
        Err(())
    }
}

// Create a client channel with our failing IO
let chan = Chan::<ClientProtocol, _>::new(FailingIO);

// Send should work
println!("Attempting to send a message...");
let chan = chan.send("Hello".to_string()).await.map_err(|_| {
    Error::Protocol("Unexpected send error")
})?;

// Receive should fail
println!("Attempting to receive a response (should fail)...");
match chan.recv().await {
    Ok(_) => println!("Unexpected success!"),
    Err(e) => {
        println!("Received expected error: {}", e);
        return Ok(());
    }
}
```

### Type Safety Examples

This example demonstrates how the type system ensures protocol adherence:

```rust
// Define a protocol: Send an i32, then receive a String, then end
type MyProtocol = Send<i32, Recv<String, End>>;

// The following function compiles because it follows the protocol correctly
async fn correct_protocol_usage(chan: Chan<MyProtocol, DummyIO>) {
    // First send an i32 as required by the protocol
    let chan = chan.send(42).await.unwrap();
    
    // Then receive a String as required by the protocol
    let (response, chan) = chan.recv().await.unwrap();
    let _: String = response; // Type check
    
    // Finally close the channel as required by the protocol
    chan.close().unwrap();
}

// The following function would not compile because it violates the protocol
// by trying to receive before sending
/*
async fn incorrect_protocol_usage(chan: Chan<MyProtocol, DummyIO>) {
    // Error: The protocol requires sending an i32 first, but we're trying to receive
    let (response, chan) = chan.recv().await.unwrap();
    
    // Send an i32
    let chan = chan.send(42).await.unwrap();
    
    // Close the channel
    chan.close().unwrap();
}
*/

// The following function would not compile because it violates the protocol
// by trying to send a String instead of an i32
/*
async fn incorrect_type_usage(chan: Chan<MyProtocol, DummyIO>) {
    // Error: The protocol requires sending an i32, but we're trying to send a String
    let chan = chan.send("hello".to_string()).await.unwrap();
    
    // Receive a String
    let (response, chan) = chan.recv().await.unwrap();
    
    // Close the channel
    chan.close().unwrap();
}
*/
```

## Visual Protocol Representation

### Simple Ping-Pong Protocol

```
                  PingPongClient                 PingPongServer
                  --------------                 --------------
                        |                              |
                        |        Send(i32)            |
                        | ---------------------------> |
                        |                              |
                        |        Recv(String)          |
                        | <--------------------------- |
                        |                              |
                        |           End                |
                        | - - - - - - - - - - - - - - -|
                        |                              |
```

Type-Level Representation:
```
Client: Send<i32, Recv<String, End>>
Server: Recv<i32, Send<String, End>>
```

### Query-Response Protocol

```
Client                                Server
  |                                     |
  |------- Send(String) - Query ------->|
  |                                     |
  |<------ Recv(String) - Response -----|
  |                                     |
  |-------------- End ---------------->|
```

Type-Level Representation:
```
Client: Send<String, Recv<String, End>>
Server: Recv<String, Send<String, End>>
```

## Advanced Topics

### Custom IO Implementations

The library allows you to create custom IO implementations by implementing the `Sender<T>` and `Receiver<T>` traits:

```rust
// Define a custom IO implementation
struct MyIO<T> {
    value: Option<T>,
}

// Define a custom error type
#[derive(Debug)]
struct MyError;

// Implement Sender for our custom IO
impl<T> Sender<T> for MyIO<T> {
    type Error = MyError;
    
    fn send(&mut self, value: T) -> Result<(), Self::Error> {
        self.value = Some(value);
        Ok(())
    }
}

// Implement Receiver for our custom IO
impl<T> Receiver<T> for MyIO<T> {
    type Error = MyError;
    
    fn recv(&mut self) -> Result<T, Self::Error> {
        self.value.take().ok_or(MyError)
    }
}

// Create a channel with our custom IO implementation
let io = MyIO { value: None };
let chan = Chan::<Send<i32, End>, _>::new(io);
```

### Protocol Testing

The library provides helper functions for testing protocol types:

```rust
// Assert that a type implements the Protocol trait
assert_protocol::<PingPongClient>();

// Assert that two types have the correct duality relationship
assert_dual::<PingPongClient, PingPongServer>();

// Create a mock channel for testing
let _client_chan: Chan<PingPongClient, ()> = mock_channel::<PingPongClient, ()>();
```

These functions help verify that protocol types are correctly defined and have the expected duality relationships.

For detailed information about testing session type protocols, including examples and best practices, see the [Testing Protocols Guide](testing-protocols.md).