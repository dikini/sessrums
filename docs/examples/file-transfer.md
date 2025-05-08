# File Transfer Protocol with Choice Example

This example demonstrates a file transfer protocol with success and failure paths using the MPST DSL. It shows how to use the `choice` construct to handle different scenarios in a protocol.

## Protocol Definition

```rust
use sessrums_macro::{mpst, project};
use sessrums_types::roles::{Client, Server};
use sessrums_types::transport::MockChannelEnd;
use sessrums_types::session_types::{Session, Either};
use sessrums_types::error::SessionError;
use serde::{Serialize, Deserialize};

// Define custom message types for the protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileRequest {
    filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileContent {
    content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileNotFound {
    filename: String,
    reason: String,
}

// Define a protocol for file transfer with success/failure paths
mpst! {
    protocol FileTransfer {
        // Define the participants
        participant Client;
        participant Server;

        // Client requests a file
        Client -> Server: FileRequest;
        
        // Server decides whether the file exists or not
        choice at Server {
            // Success path: Server sends the file content
            option FileFound {
                Server -> Client: FileContent;
                end;
            }
            
            // Failure path: Server sends a file not found error
            option FileNotFound {
                Server -> Client: FileNotFound;
                end;
            }
        }
    }
}
```

## Projection to Local Protocols

Once we have defined the global protocol, we can project it to local protocols for each participant:

```rust
// Project the global protocol to local protocols for each role
type ClientProtocol = project!(FileTransfer, Client);
type ServerProtocol = project!(FileTransfer, Server);
```

The projection results in the following local protocols:

- `ClientProtocol`: Send a FileRequest to Server, then offer a choice from Server (either receive FileContent or receive FileNotFound), then end
- `ServerProtocol`: Receive a FileRequest from Client, then choose between sending FileContent or sending FileNotFound, then end

## Implementation

Here's how we can implement the client and server behaviors using the projected protocols:

```rust
// Client implementation
async fn run_client(
    session: Session<ClientProtocol, MockChannelEnd>,
    filename: String
) -> Result<(), SessionError> {
    // Send a file request to the server
    println!("Client: Requesting file '{}'", filename);
    let request = FileRequest { filename };
    let session = session.send(request).await?;
    
    // Offer a choice from the server
    let session_branch = session.offer().await?;
    
    match session_branch {
        // File found path
        Either::Left(session) => {
            // Receive the file content
            let (content, session) = session.receive().await?;
            println!("Client: Received file with {} bytes", content.content.len());
            
            // End the session
            session.close().await?;
        },
        
        // File not found path
        Either::Right(session) => {
            // Receive the file not found error
            let (error, session) = session.receive().await?;
            println!("Client: File not found: {} ({})", error.filename, error.reason);
            
            // End the session
            session.close().await?;
        }
    }
    
    Ok(())
}

// Server implementation
async fn run_server(
    session: Session<ServerProtocol, MockChannelEnd>,
    available_files: &[String]
) -> Result<(), SessionError> {
    // Receive a file request from the client
    let (request, session) = session.receive().await?;
    println!("Server: Received request for file '{}'", request.filename);
    
    // Check if the file exists
    if available_files.contains(&request.filename) {
        // File exists, send the content
        println!("Server: File found, sending content");
        
        // Choose the file found path
        let session = session.select_left().await?;
        
        // Create some dummy content
        let content = FileContent {
            content: format!("Content of file {}", request.filename).into_bytes(),
        };
        
        // Send the file content
        let session = session.send(content).await?;
        
        // End the session
        session.close().await?;
    } else {
        // File doesn't exist, send an error
        println!("Server: File not found, sending error");
        
        // Choose the file not found path
        let session = session.select_right().await?;
        
        // Create the error message
        let error = FileNotFound {
            filename: request.filename,
            reason: "File does not exist on the server".to_string(),
        };
        
        // Send the error
        let session = session.send(error).await?;
        
        // End the session
        session.close().await?;
    }
    
    Ok(())
}
```

## Running the Example

To run the example, we need to create a pair of channels and spawn the client and server tasks:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define available files on the server
    let available_files = vec![
        "file1.txt".to_string(),
        "file2.txt".to_string(),
        "file3.txt".to_string(),
    ];
    
    // Test with a file that exists
    {
        // Create a pair of channels
        let (client_channel, server_channel) = MockChannelEnd::new_pair();
        
        // Create sessions for client and server
        let client_session = Session::<ClientProtocol, _>::new(client_channel);
        let server_session = Session::<ServerProtocol, _>::new(server_channel);
        
        // Spawn client and server tasks
        let client_task = tokio::spawn(run_client(client_session, "file1.txt".to_string()));
        let server_task = tokio::spawn(run_server(server_session, &available_files));
        
        // Wait for both tasks to complete
        let _ = tokio::try_join!(client_task, server_task)?;
    }
    
    println!("---");
    
    // Test with a file that doesn't exist
    {
        // Create a pair of channels
        let (client_channel, server_channel) = MockChannelEnd::new_pair();
        
        // Create sessions for client and server
        let client_session = Session::<ClientProtocol, _>::new(client_channel);
        let server_session = Session::<ServerProtocol, _>::new(server_channel);
        
        // Spawn client and server tasks
        let client_task = tokio::spawn(run_client(client_session, "nonexistent.txt".to_string()));
        let server_task = tokio::spawn(run_server(server_session, &available_files));
        
        // Wait for both tasks to complete
        let _ = tokio::try_join!(client_task, server_task)?;
    }
    
    Ok(())
}
```

## Output

When running this example, you should see output similar to:

```
Client: Requesting file 'file1.txt'
Server: Received request for file 'file1.txt'
Server: File found, sending content
Client: Received file with 22 bytes
---
Client: Requesting file 'nonexistent.txt'
Server: Received request for file 'nonexistent.txt'
Server: File not found, sending error
Client: File not found: nonexistent.txt (File does not exist on the server)
```

## Key Points

This example demonstrates several key concepts:

1. **Choice Construct**: Using the `choice at Role` construct to represent different paths in the protocol
2. **Option Labels**: Using the `option Label` syntax to label different branches
3. **Projection of Choice**: How choice is projected to different roles:
   - The deciding role (Server) gets a `Select` operation
   - The other role (Client) gets an `Offer` operation
4. **Type Safety**: The compiler ensures that both roles handle all possible branches of the protocol

The choice construct is a powerful feature of the MPST system that allows you to model protocols with different possible paths. It ensures that all participants are aware of the possible paths and handle them correctly.