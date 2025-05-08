# Simple Ping-Pong Protocol Example

This example demonstrates a simple ping-pong protocol between a client and a server using the MPST DSL.

## Protocol Definition

```rust
use sessrums_macro::{mpst, project};
use sessrums_types::roles::{Client, Server};
use sessrums_types::transport::MockChannelEnd;
use sessrums_types::session_types::Session;
use sessrums_types::error::SessionError;

// Define a simple protocol with two participants: Client and Server
// The protocol consists of a simple message exchange where:
// 1. Client sends a String message to Server
// 2. Server sends a String message back to Client
mpst! {
    protocol PingPong {
        // Define the participants
        participant Client;
        participant Server;

        // Define the message exchange
        Client -> Server: String;  // Client sends a message to Server
        Server -> Client: String;  // Server responds with a message to Client
        end;                       // End of the protocol
    }
}
```

## Projection to Local Protocols

Once we have defined the global protocol, we can project it to local protocols for each participant:

```rust
// Project the global protocol to local protocols for each role
type ClientProtocol = project!(PingPong, Client, String);
type ServerProtocol = project!(PingPong, Server, String);
```

The projection results in the following local protocols:

- `ClientProtocol`: Send a String to Server, then receive a String from Server, then end
- `ServerProtocol`: Receive a String from Client, then send a String to Client, then end

## Implementation

Here's how we can implement the client and server behaviors using the projected protocols:

```rust
// Client implementation
async fn run_client(session: Session<ClientProtocol, MockChannelEnd>) -> Result<(), SessionError> {
    // Send a message to the server
    println!("Client: Sending 'Ping' to server");
    let session = session.send("Ping".to_string()).await?;
    
    // Receive a message from the server
    let (message, session) = session.receive().await?;
    println!("Client: Received '{}' from server", message);
    
    // End the session
    session.close().await?;
    
    Ok(())
}

// Server implementation
async fn run_server(session: Session<ServerProtocol, MockChannelEnd>) -> Result<(), SessionError> {
    // Receive a message from the client
    let (message, session) = session.receive().await?;
    println!("Server: Received '{}' from client", message);
    
    // Send a message to the client
    println!("Server: Sending 'Pong' to client");
    let session = session.send("Pong".to_string()).await?;
    
    // End the session
    session.close().await?;
    
    Ok(())
}
```

## Running the Example

To run the example, we need to create a pair of channels and spawn the client and server tasks:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a pair of channels
    let (client_channel, server_channel) = MockChannelEnd::new_pair();
    
    // Create sessions for client and server
    let client_session = Session::<ClientProtocol, _>::new(client_channel);
    let server_session = Session::<ServerProtocol, _>::new(server_channel);
    
    // Spawn client and server tasks
    let client_task = tokio::spawn(run_client(client_session));
    let server_task = tokio::spawn(run_server(server_session));
    
    // Wait for both tasks to complete
    let _ = tokio::try_join!(client_task, server_task)?;
    
    Ok(())
}
```

## Output

When running this example, you should see output similar to:

```
Client: Sending 'Ping' to server
Server: Received 'Ping' from client
Server: Sending 'Pong' to client
Client: Received 'Pong' from server
```

## Key Points

This example demonstrates several key concepts:

1. **Global Protocol Definition**: Using the `mpst!` macro to define a global protocol
2. **Projection**: Using the `project!` macro to project the global protocol to local protocols
3. **Session Types**: Using the projected local protocols to implement type-safe communication
4. **Type Safety**: The compiler ensures that the client and server implementations follow the protocol

The ping-pong protocol is the simplest possible MPST protocol, but it demonstrates the core concepts of the MPST system. More complex protocols can be built by combining message passing with choice and recursion.