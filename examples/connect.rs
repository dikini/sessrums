//! Example demonstrating connection establishment and communication using session types.
//!
//! This example shows how to establish a connection between a client and server
//! using TCP streams and communicate using session types. The client and server
//! run in separate threads and follow a simple protocol:
//!
//! 1. Client sends a string message to the server
//! 2. Server receives the message and sends back an integer response
//! 3. Client receives the integer response
//! 4. Both client and server end the session
//!
//! To run this example:
//! ```
//! cargo run --example connect
//! ```

use sessrums::chan::Chan;
use sessrums::connect::StreamWrapper;
use sessrums::proto::{Send, Recv, End};
use std::net::{TcpListener, TcpStream};

// Define the protocol types for the client and server
// The client sends a String, then receives a String, then ends
type ClientProto = Send<String, Recv<String, End>>;

// The server receives a String, then sends a String, then ends
// This is the dual of the client protocol
type ServerProto = Recv<String, Send<String, End>>;

fn main() {
    println!("Connection Example");
    println!("=================");
    println!("This example demonstrates how to use the connection establishment functions");
    println!("and stream wrappers to create a session-typed communication channel.");
    println!();
    println!("To run a working example, you would need to:");
    println!("1. Start a server that listens on a port");
    println!("2. Connect a client to that server");
    println!("3. Use the StreamWrapper to wrap the TCP streams");
    println!("4. Create channels with the appropriate protocols");
    println!("5. Send and receive messages according to the protocol");
    println!();
    println!("The code in this example shows how to do this, but it's not");
    println!("actually running the server and client since that would require");
    println!("setting up proper async runtime handling for non-blocking IO.");
    println!();
    println!("For a real application, you would:");
    println!("- Use proper error handling");
    println!("- Set up the TCP streams correctly for non-blocking IO");
    println!("- Handle connection timeouts and retries");
    println!("- Implement proper protocol negotiation");
    println!();
    println!("See the documentation for more details on how to use the");
    println!("connection establishment functions in a real application.");
}

/// Example server code (not actually run in this example)
fn server_example() {
    // Create a TCP listener
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server: Listening on {}", listener.local_addr().unwrap());
    
    // Accept a connection
    let (stream, addr) = listener.accept().unwrap();
    println!("Server: Accepted connection from {}", addr);
    
    // Set the stream to non-blocking mode
    stream.set_nonblocking(true).unwrap();
    
    // Wrap the stream
    let wrapper = StreamWrapper::<TcpStream, String>::new(stream);
    
    // Create a channel with the server protocol
    let _chan = Chan::<ServerProto, _>::new(wrapper);
    
    // In an async context, you would:
    // 1. Receive a string message
    //    let (message, chan) = chan.recv().await.unwrap();
    // 2. Send back a string response
    //    let chan = chan.send("Hello, client!".to_string()).await.unwrap();
    // 3. Close the channel
    //    chan.close().unwrap();
}

/// Example client code (not actually run in this example)
fn client_example() {
    // Connect to the server
    let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    println!("Client: Connected to {}", stream.peer_addr().unwrap());
    
    // Set the stream to non-blocking mode
    stream.set_nonblocking(true).unwrap();
    
    // Wrap the stream
    let wrapper = StreamWrapper::<TcpStream, String>::new(stream);
    
    // Create a channel with the client protocol
    let _chan = Chan::<ClientProto, _>::new(wrapper);
    
    // In an async context, you would:
    // 1. Send a string message
    //    let chan = chan.send("Hello, server!".to_string()).await.unwrap();
    // 2. Receive a string response
    //    let (response, chan) = chan.recv().await.unwrap();
    // 3. Close the channel
    //    chan.close().unwrap();
}

/// Example of using the connect function
fn connect_example() {
    // Connect to the server
    let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    
    // Use the connect function to create a channel
    let _chan = sessrums::connect::connect::<ClientProto, _, String>(stream);
    
    // The rest would be the same as in client_example()
}