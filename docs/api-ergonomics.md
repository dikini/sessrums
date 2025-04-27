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
use sessrums::proto::{Send, Recv, End};

// Define a client that chooses between sending an i32 or a String
type MyClient = ChoiceClient<Send<i32, End>, Send<String, End>>;

// Define a server that offers to receive either an i32 or a String
type MyServer = OfferServer<Recv<i32, End>, Recv<String, End>>;
```

## Helper Functions

sessrums provides several helper functions for common operations:

### Channel Pair Creation

```rust
// Create a pair of channels with dual protocols
pub fn channel_pair<P, IO>() -> (Chan<P, IO>, Chan<P::Dual, IO>)
where
    P: Protocol,
    IO: Default + Clone;
```

Example usage:

```rust
use sessrums::api::{channel_pair, RequestClient, RequestServer};

// Create a pair of channels for a request-response protocol
let (client, server) = channel_pair::<RequestClient<String, i32>, ()>();
```

### Request-Response Pair Creation

```rust
// Create a pair of channels for a request-response protocol
pub fn request_response_pair<Req, Resp, IO>() -> (Chan<RequestClient<Req, Resp>, IO>, Chan<RequestServer<Req, Resp>, IO>)
where
    IO: Default + Clone;
```

Example usage:

```rust
use sessrums::api::request_response_pair;

// Create a pair of channels for a request-response protocol
let (client, server) = request_response_pair::<String, i32, ()>();
```

### Ping-Pong Pair Creation

```rust
// Create a pair of channels for a ping-pong protocol
pub fn ping_pong_pair<Ping, Pong, IO>() -> (Chan<PingClient<Ping, Pong>, IO>, Chan<PingServer<Ping, Pong>, IO>)
where
    IO: Default + Clone;
```

Example usage:

```rust
use sessrums::api::ping_pong_pair;

// Create a pair of channels for a ping-pong protocol
let (client, server) = ping_pong_pair::<i32, String, ()>();
```

### Connection Establishment

```rust
// Establish a connection with a specific protocol
pub async fn connect_with_protocol<P, IO, C>(conn_info: C) -> Result<Chan<P, IO>>
where
    P: Protocol,
    IO: Default,
    C: connect::ConnectInfo<IO>;
```

Example usage:

```rust
use sessrums::api::{connect_with_protocol, RequestClient};
use sessrums::connect::TcpConnectInfo;
use std::net::SocketAddr;

// Connect to a server with a request-response protocol
let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
let conn_info = TcpConnectInfo::new(addr);
let client = connect_with_protocol::<RequestClient<String, i32>, _, _>(conn_info).await.unwrap();
```

## Macros

sessrums provides macros for defining protocols with a more concise syntax:

### Protocol Macro

The `protocol!` macro allows defining complex protocol types with a more readable syntax:

```rust
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
// Define a more complex protocol with choice
type ComplexClient = protocol!(send(i32) >> choose(
    send(String) >> recv(bool) >> end,
    send(f64) >> recv(char) >> end
));
```

### Protocol Pair Macro

The `protocol_pair!` macro creates a pair of protocol types for a client and server, ensuring that they are duals of each other:

```rust
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

Here's a complete example that demonstrates the use of the API ergonomics improvements:

```rust
use sessrums::api::{RequestClient, RequestServer, request_response_pair};
use sessrums::protocol;
use sessrums::protocol_pair;
use sessrums::error::Result;

// Define protocol types using type aliases
type MyClient = RequestClient<String, i32>;
type MyServer = RequestServer<String, i32>;

// Define protocol types using the protocol macro
type MacroClient = protocol!(send(String) >> recv(i32) >> end);
type MacroServer = protocol!(recv(String) >> send(i32) >> end);

// Define a protocol pair using the protocol_pair macro
protocol_pair! {
    pub MyProtocol<Req, Resp> {
        client: send(Req) >> recv(Resp) >> end,
        server: recv(Req) >> send(Resp) >> end
    }
}

// Use the generated type aliases
type PairClient = MyProtocol::Client<String, i32>;
type PairServer = MyProtocol::Server<String, i32>;

// Create a pair of channels using the helper function
let (client, server) = request_response_pair::<String, i32, ()>();

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

## Benefits of API Ergonomics Improvements

The API ergonomics improvements provide several benefits:

1. **Reduced Boilerplate**: Type aliases and helper functions reduce the amount of code needed to define and use protocols.

2. **Improved Readability**: The protocol macro provides a more concise and readable syntax for defining protocols.

3. **Type Safety**: The protocol_pair macro ensures that client and server protocols are duals of each other at compile time.

4. **Simplified Connection Establishment**: Helper functions for creating channel pairs and establishing connections simplify the process of setting up communication.

5. **Common Protocol Patterns**: Type aliases for common protocol patterns make it easier to define and use these patterns.

By using these API ergonomics improvements, you can write more concise, readable, and maintainable code when working with session types.