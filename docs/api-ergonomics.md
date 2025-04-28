# API Ergonomics Guide

This guide explains the API ergonomics improvements provided by the sessrums library, including type aliases, helper functions, and macros that make the session type API more ergonomic and easier to use.

## Type Aliases

sessrums provides several type aliases for common protocol patterns:

### Request-Response Pattern

```rust
// Client side: Send a request, receive a response, then end
pub type RequestClient<Req, Resp> = Send<Req, Recv<Resp, End>>;

// Server side: Receive a request, send a response, then end
pub type RequestServer<Req, Resp> = Recv<Req, Send<Resp, End>>;
```

Example usage:

```rust
use sessrums::api::{RequestClient, RequestServer};
use sessrums::proto::{Send, Recv, End}; // Ensure base protocols are imported

// Define a client that sends a String request and receives an i32 response
type MyClient = RequestClient<String, i32>;

// Define a server that receives a String request and sends an i32 response
type MyServer = RequestServer<String, i32>;
```

### Ping-Pong Pattern

```rust
// Client side: Send a ping, receive a pong, then end
pub type PingClient<Ping, Pong> = Send<Ping, Recv<Pong, End>>;

// Server side: Receive a ping, send a pong, then end
pub type PingServer<Ping, Pong> = Recv<Ping, Send<Pong, End>>;
```

Example usage:

```rust
use sessrums::api::{PingClient, PingServer};
use sessrums::proto::{Send, Recv, End}; // Ensure base protocols are imported

// Define a client that sends an i32 ping and receives a String pong
type MyClient = PingClient<i32, String>;

// Define a server that receives an i32 ping and sends a String pong
type MyServer = PingServer<i32, String>;
```

### Choice Pattern

```rust
// Client side: Choose between two protocols
pub type ChoiceClient<P1, P2> = Choose<P1, P2>;

// Server side: Offer a choice between two protocols
pub type OfferServer<P1, P2> = Offer<P1, P2>;
```

Example usage:

```rust
use sessrums::api::{ChoiceClient, OfferServer};
use sessrums::proto::{Send, Recv, End, Choose, Offer}; // Ensure base protocols are imported

// Define a client that chooses between sending an i32 or a String
type MyClient = ChoiceClient<Send<i32, End>, Send<String, End>>;

// Define a server that offers to receive either an i32 or a String
type MyServer = OfferServer<Recv<i32, End>, Recv<String, End>>;
```

## Helper Functions

sessrums provides several helper functions for common operations:

### Channel Pair Creation

```rust
// Create a pair of channels with dual protocols, suitable for local communication
// or testing. The `IO` type parameter determines the underlying channel mechanism.
pub fn channel_pair<P, IO>() -> (Chan<P, IO>, Chan<P::Dual, IO>)
where
    P: Protocol,
    IO: Default + Clone; // IO typically represents the channel type (e.g., in-memory, TCP)
```

Example usage:

```rust
use sessrums::api::{channel_pair, RequestClient};
use sessrums::proto::{Protocol, Chan}; // Import necessary traits/types

// Create a pair of in-memory channels (using `()` for IO) for a request-response protocol
let (client, server) = channel_pair::<RequestClient<String, i32>, ()>();
```

### Request-Response Pair Creation

```rust
// Create a pair of channels specifically for a request-response protocol.
pub fn request_response_pair<Req, Resp, IO>() -> (Chan<RequestClient<Req, Resp>, IO>, Chan<RequestServer<Req, Resp>, IO>)
where
    IO: Default + Clone; // IO determines the channel type
```

Example usage:

```rust
use sessrums::api::{request_response_pair, RequestClient, RequestServer};
use sessrums::proto::Chan; // Import necessary traits/types

// Create a pair of in-memory channels for a request-response protocol
let (client, server) = request_response_pair::<String, i32, ()>();
```

### Ping-Pong Pair Creation

```rust
// Create a pair of channels specifically for a ping-pong protocol.
pub fn ping_pong_pair<Ping, Pong, IO>() -> (Chan<PingClient<Ping, Pong>, IO>, Chan<PingServer<Ping, Pong>, IO>)
where
    IO: Default + Clone; // IO determines the channel type
```

Example usage:

```rust
use sessrums::api::{ping_pong_pair, PingClient, PingServer};
use sessrums::proto::Chan; // Import necessary traits/types

// Create a pair of in-memory channels for a ping-pong protocol
let (client, server) = ping_pong_pair::<i32, String, ()>();
```

### Connection Establishment

```rust
// Establish a connection (e.g., TCP) using a specific connection info object
// and associate it with a session protocol `P`.
pub async fn connect_with_protocol<P, IO, C>(conn_info: C) -> Result<Chan<P, IO>>
where
    P: Protocol,
    IO: Default, // IO represents the resulting channel type (e.g., TcpChannel)
    C: sessrums::connect::ConnectInfo<IO>; // ConnectInfo provides connection details
```

Example usage:

```rust
use sessrums::api::{connect_with_protocol, RequestClient};
use sessrums::connect::TcpConnectInfo; // Example connection type
use sessrums::proto::{Protocol, Chan}; // Import necessary traits/types
use sessrums::error::Result; // Import Result type
use std::net::SocketAddr;

async fn connect_example() -> Result<()> {
    // Connect to a server using TCP with a request-response protocol
    let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
    let conn_info = TcpConnectInfo::new(addr);
    // The `IO` type is often inferred based on the ConnectInfo (`_`)
    let client: Chan<RequestClient<String, i32>, _> = connect_with_protocol(conn_info).await?;
    // Use the client channel...
    Ok(())
}
```

## Macros

sessrums provides macros for defining protocols with a more concise syntax:

### Protocol Macro

The `protocol!` macro allows defining complex protocol types with a more readable syntax:

```rust
use sessrums::protocol; // Import the macro
use sessrums::proto::{Send, Recv, End, Choose, Offer}; // Import base protocols

// Define a protocol using the macro
type ClientProto = protocol!(send(i32) >> recv(String) >> end);
type ServerProto = protocol!(recv(i32) >> send(String) >> end);
```

The macro supports the following protocol combinators:

- `send(T)` - Send a value of type T
- `recv(T)` - Receive a value of type T
- `choose(P1, P2)` - Choose between protocols P1 and P2
- `offer(P1, P2)` - Offer a choice between protocols P1 and P2
- `end` - End the protocol

Example with choice:

```rust
use sessrums::protocol; // Import the macro
use sessrums::proto::{Send, Recv, End, Choose, Offer}; // Import base protocols

// Define a more complex protocol with choice
type ComplexClient = protocol!(send(i32) >> choose(
    send(String) >> recv(bool) >> end,
    send(f64) >> recv(char) >> end
));
```

### Protocol Pair Macro

The `protocol_pair!` macro creates a pair of protocol types for a client and server, ensuring that they are duals of each other:

```rust
use sessrums::protocol_pair; // Import the macro
use sessrums::proto::{Send, Recv, End}; // Import base protocols

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

The macro generates a module with two type aliases:

- `Client<...>` - The client side of the protocol
- `Server<...>` - The server side of the protocol

It also includes a compile-time verification that the protocols are duals of each other.

## Complete Example

Here's a complete example demonstrating the use of these API improvements with in-memory channels:

```rust
use sessrums::api::{RequestClient, RequestServer, request_response_pair};
use sessrums::protocol; // Import macros
use sessrums::protocol_pair;
use sessrums::proto::{Chan, End, Protocol, Recv, Send}; // Import necessary types/traits
use sessrums::error::Result;

// Define protocol types using type aliases
type MyClientAlias = RequestClient<String, i32>;
type MyServerAlias = RequestServer<String, i32>;

// Define protocol types using the protocol macro
type MacroClient = protocol!(send(String) >> recv(i32) >> end);
type MacroServer = protocol!(recv(String) >> send(i32) >> end);

// Define a protocol pair using the protocol_pair macro
protocol_pair! {
    pub MyPairProto<Req, Resp> {
        client: send(Req) >> recv(Resp) >> end,
        server: recv(Req) >> send(Resp) >> end
    }
}

// Use the generated type aliases
type PairClient = MyPairProto::Client<String, i32>;
type PairServer = MyPairProto::Server<String, i32>;

// Client implementation (generic over IO type, typically `()` for in-memory)
async fn run_client<IO>(chan: Chan<MyClientAlias, IO>) -> Result<()>
where
    IO: Default + Clone + Send + 'static, // Add necessary bounds for async task
{
    // Send a request
    let request = "Hello, server!".to_string();
    let chan = chan.send(request).await?;

    // Receive the response
    let (response, chan) = chan.recv().await?;
    println!("Client received: {}", response);

    // Close the channel
    chan.close()?; // Use close for End protocol

    Ok(())
}

// Server implementation (generic over IO type, typically `()` for in-memory)
async fn run_server<IO>(chan: Chan<MyServerAlias, IO>) -> Result<()>
where
    IO: Default + Clone + Send + 'static, // Add necessary bounds for async task
{
    // Receive the request
    let (request, chan) = chan.recv().await?;
    println!("Server received: {}", request);

    // Process and send response
    let response = 42;
    let chan = chan.send(response).await?;

    // Close the channel
    chan.close()?; // Use close for End protocol

    Ok(())
}

async fn main_example() -> Result<()> {
    // Create a pair of in-memory channels using the helper function
    // Here, IO is specified as `()`
    let (client_chan, server_chan) = request_response_pair::<String, i32, ()>();

    // Spawn tasks (requires an async runtime like tokio or async-std)
    let client_handle = tokio::spawn(run_client(client_chan));
    let server_handle = tokio::spawn(run_server(server_chan));

    // Wait for tasks to complete
    client_handle.await.unwrap()?;
    server_handle.await.unwrap()?;

    Ok(())
}
```

## Benefits of API Ergonomics Improvements

The API ergonomics improvements provide several benefits:

- **Reduced Boilerplate**: Type aliases and helper functions reduce the amount of code needed to define and use protocols.
- **Improved Readability**: The `protocol!` macro provides a more concise and readable syntax for defining protocols.
- **Type Safety**: The `protocol_pair!` macro ensures that client and server protocols are duals of each other at compile time.
- **Simplified Setup**: Helper functions for creating channel pairs and establishing connections simplify setting up communication, especially for common patterns and local testing.
- **Common Protocol Patterns**: Type aliases for common protocol patterns (Request/Response, Ping/Pong) make it easier to define and use them consistently.

By using these API ergonomics improvements, you can write more concise, readable, and maintainable code when working with session types.