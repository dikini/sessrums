# Testing Session Type Protocols

This document provides guidance and examples for testing session type protocols built with sessrums.

## Testing Approaches

Testing ensures your communication protocols behave correctly. sessrums supports three main approaches:

1.  **Type-Level Testing**: Verifying protocol structure and relationships using the type system *before* runtime.
2.  **Runtime Testing**: Executing protocol steps using actual channels (often in-memory for tests) to check interaction logic.
3.  **Compile-Fail Testing**: Ensuring that code violating protocol rules fails to compile, leveraging the core safety guarantee of session types.

### Type-Level Testing

Use helper functions to assert properties about your protocol types directly in your tests. This catches structural errors early.

```rust
use sessrums::proto::{Send, Recv, End, Protocol};
// Assuming helpers are in a testing module or accessible path
use sessrums::testing::{assert_protocol, assert_dual};

// Define protocol types
type ClientProto = Send<i32, Recv<String, End>>;
type ServerProto = Recv<i32, Send<String, End>>;

#[test]
fn test_protocol_structure() {
    // Verify that types implement the Protocol trait
    assert_protocol::<ClientProto>();
    assert_protocol::<ServerProto>();

    // Verify that ServerProto is the dual of ClientProto
    assert_dual::<ClientProto, ServerProto>();
}
```

### Runtime Testing

Test the actual send/receive logic by running participants concurrently using in-memory channels. The `channel_pair::<_, ()>()` function is ideal for this.

```rust
use sessrums::proto::{Chan, Send, Recv, End, Protocol};
use sessrums::api::channel_pair; // Use helper for in-memory channels
use sessrums::error::Result;

// Define protocol types
type PingClient = Send<i32, Recv<String, End>>;
type PongServer = Recv<i32, Send<String, End>>;

async fn run_ping_client(chan: Chan<PingClient, ()>) -> Result<()> {
    println!("Client: Sending ping (42)");
    let chan = chan.send(42).await?;
    println!("Client: Receiving pong...");
    let (response, chan) = chan.recv().await?;
    assert_eq!(response, "pong");
    println!("Client: Received '{}', closing.", response);
    chan.close()?;
    Ok(())
}

async fn run_pong_server(chan: Chan<PongServer, ()>) -> Result<()> {
    println!("Server: Receiving ping...");
    let (ping_val, chan) = chan.recv().await?;
    assert_eq!(ping_val, 42);
    println!("Server: Received {}, sending pong...", ping_val);
    let chan = chan.send("pong".to_string()).await?;
    println!("Server: Pong sent, closing.");
    chan.close()?;
    Ok(())
}

#[tokio::test] // Requires an async runtime like tokio
async fn test_ping_pong_runtime() {
    // Create a pair of connected in-memory channels (IO = ())
    let (client_chan, server_chan) = channel_pair::<PingClient, ()>();

    // Run client and server concurrently
    let client_handle = tokio::spawn(run_ping_client(client_chan));
    let server_handle = tokio::spawn(run_pong_server(server_chan));

    // Wait for both tasks to complete and check results
    let client_res = client_handle.await.unwrap();
    let server_res = server_handle.await.unwrap();

    assert!(client_res.is_ok(), "Client failed: {:?}", client_res);
    assert!(server_res.is_ok(), "Server failed: {:?}", server_res);
}
```

### Compile-Fail Testing

Use the `trybuild` crate to create tests that *expect* compilation errors for specific code snippets that misuse protocols. This confirms that the type system prevents invalid sequences of operations.

Setup (`Cargo.toml`):
```toml
[dev-dependencies]
trybuild = "1.0"
```

Test function (`tests/my_tests.rs`):
```rust
#[test]
fn test_compile_failures() {
    let t = trybuild::TestCases::new();
    // Point to files containing code that should fail to compile
    t.compile_fail("tests/compile_fail/wrong_order.rs");
    t.compile_fail("tests/compile_fail/wrong_type.rs");
    // Add more compile-fail tests as needed
}
```

Example compile-fail test file (`tests/compile_fail/wrong_order.rs`):
```rust,ignore
// This file is expected to fail compilation by trybuild.

use sessrums::proto::{Chan, Send, Recv, End};
use sessrums::error::Result;

// Protocol: Send i32, then Recv String
type ClientProto = Send<i32, Recv<String, End>>;

async fn incorrect_protocol_usage(chan: Chan<ClientProto, ()>) -> Result<()> {
    // Error: Protocol expects Send first, but we try to Recv.
    // The type system should prevent this call.
    let (response, chan) = chan.recv().await?; // COMPILE ERROR EXPECTED HERE

    // ... rest of code (unreachable due to expected compile error)
    // let chan = chan.send(123).await?;
    // chan.close()?;
    Ok(())
}

// Dummy main needed for trybuild to attempt compilation
fn main() {}
```

## Testing Helper Functions

sessrums provides helpers (often in a `testing` module or similar) to simplify assertions:

- **`assert_protocol<P: Protocol>()`**: Asserts that type `P` implements the `Protocol` trait. Useful for basic type checks.
- **`assert_dual<P: Protocol, Q: Protocol>()`**: Asserts that `Q` is the dual of `P` (i.e., `P::Dual == Q`). Essential for verifying client/server pairs match.
- **`assert_self_dual<P: Protocol>()`**: Asserts that `P` is its own dual (i.e., `P::Dual == P`). Less common, but useful for symmetric protocols.

*(Note: The exact location and names of these helpers might vary; check the library's API documentation or source code.)*

## Example Tests

### Simple Protocol Test (Ping-Pong)

The runtime test example above already demonstrates testing a simple ping-pong protocol using `channel_pair` and concurrent tasks.

### Complex Protocol Test (Authentication)

Testing protocols with choices involves testing each branch.

```rust
use sessrums::proto::{Chan, Send, Recv, End, Choose, Offer, Protocol, Either};
use sessrums::api::channel_pair;
use sessrums::error::Result;
use sessrums::testing::{assert_protocol, assert_dual}; // Assuming helpers exist

// Define types (simplified for example)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)] // Add derives for send/recv
struct AuthRequest { user: String }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)] // Add derives for send/recv
enum AuthResponse { Ok, Fail }

type SuccessProto = Send<String, End>; // Send welcome message
type FailureProto = Recv<u32, End>;    // Receive error code

// Client: Send AuthRequest, Recv AuthResponse, Choose based on response
type AuthClient = Send<AuthRequest, Recv<AuthResponse, Choose<SuccessProto, FailureProto>>>;
// Server: Recv AuthRequest, Send AuthResponse, Offer based on response
type AuthServer = Recv<AuthRequest, Send<AuthResponse, Offer<SuccessProto, FailureProto>>>;

#[test]
fn test_auth_protocol_types() {
    assert_protocol::<AuthClient>();
    assert_protocol::<AuthServer>();
    assert_dual::<AuthClient, AuthServer>();
}

// --- Test Success Path ---
async fn run_auth_client_success(chan: Chan<AuthClient, ()>) -> Result<()> {
    let chan = chan.send(AuthRequest { user: "Alice".into() }).await?;
    let (resp, chan) = chan.recv().await?;
    match resp {
        AuthResponse::Ok => {
            println!("Client (Success): Auth OK. Choosing Success branch.");
            let chan = chan.choose_left().await?; // Choose SuccessProto
            let chan = chan.send("Welcome data".into()).await?;
            chan.close()?;
            Ok(())
        }
        AuthResponse::Fail => panic!("Expected Success path, got Fail"),
    }
}
async fn run_auth_server_success(chan: Chan<AuthServer, ()>) -> Result<()> {
    let (req, chan) = chan.recv().await?;
    println!("Server (Success): Received auth for {}", req.user);
    let chan = chan.send(AuthResponse::Ok).await?;
    println!("Server (Success): Sent OK. Offering choice.");
    match chan.offer().await? {
        Either::Left(chan) => { // Offer SuccessProto
            println!("Server (Success): Client chose Success. Receiving data.");
            let (data, chan) = chan.recv().await?;
            assert_eq!(data, "Welcome data");
            println!("Server (Success): Received '{}'", data);
            chan.close()?;
            Ok(())
        }
        Either::Right(_) => panic!("Expected Success path, client chose Fail"),
    }
}

#[tokio::test]
async fn test_auth_success_runtime() {
    let (client_chan, server_chan) = channel_pair::<AuthClient, ()>();
    let client_handle = tokio::spawn(run_auth_client_success(client_chan));
    let server_handle = tokio::spawn(run_auth_server_success(server_chan));
    assert!(client_handle.await.unwrap().is_ok());
    assert!(server_handle.await.unwrap().is_ok());
}

// --- Test Failure Path ---
async fn run_auth_client_fail(chan: Chan<AuthClient, ()>) -> Result<()> {
    let chan = chan.send(AuthRequest { user: "Bob".into() }).await?;
    let (resp, chan) = chan.recv().await?;
    match resp {
        AuthResponse::Ok => panic!("Expected Failure path, got Ok"),
        AuthResponse::Fail => {
            println!("Client (Fail): Auth Failed. Choosing Failure branch.");
            let chan = chan.choose_right().await?; // Choose FailureProto
            let (error_code, chan) = chan.recv().await?;
            assert_eq!(error_code, 403);
            println!("Client (Fail): Received error code {}", error_code);
            chan.close()?;
            Ok(())
        }
    }
}
async fn run_auth_server_fail(chan: Chan<AuthServer, ()>) -> Result<()> {
    let (req, chan) = chan.recv().await?;
    println!("Server (Fail): Received auth for {}", req.user);
    let chan = chan.send(AuthResponse::Fail).await?; // Simulate failure
    println!("Server (Fail): Sent Fail. Offering choice.");
    match chan.offer().await? {
        Either::Left(_) => panic!("Expected Failure path, client chose Success"),
        Either::Right(chan) => { // Offer FailureProto
            println!("Server (Fail): Client chose Failure. Sending error code.");
            let chan = chan.send(403u32).await?;
            chan.close()?;
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_auth_fail_runtime() {
    let (client_chan, server_chan) = channel_pair::<AuthClient, ()>();
    let client_handle = tokio::spawn(run_auth_client_fail(client_chan));
    let server_handle = tokio::spawn(run_auth_server_fail(server_chan));
    assert!(client_handle.await.unwrap().is_ok());
    assert!(server_handle.await.unwrap().is_ok());
}
```

### Error Handling Test

Test how protocols behave when errors occur, like unexpected channel closure.

```rust
use sessrums::proto::{Chan, Send, Recv, End};
use sessrums::api::channel_pair;
use sessrums::error::{Error, Result};

type ClientProto = Send<String, Recv<String, End>>;
type ServerProto = Recv<String, Send<String, End>>;

#[tokio::test]
async fn test_unexpected_closure() {
    let (client_chan, server_chan) = channel_pair::<ClientProto, ()>();

    // Client sends data then expects a reply
    let client_task = tokio::spawn(async move {
        println!("Client: Sending data...");
        let res = client_chan.send("hello".to_string()).await;
        println!("Client: Send result: {:?}", res);

        // Client proceeds to recv, but server will drop early
        if let Ok(chan) = res {
             println!("Client: Attempting to receive...");
             let recv_res = chan.recv().await;
             println!("Client: Recv result: {:?}", recv_res);
             // Expect ChannelClosed or potentially an Io error depending on timing/buffering
             assert!(matches!(recv_res, Err(Error::ChannelClosed) | Err(Error::Io(_))),
                     "Expected ChannelClosed or Io error, got {:?}", recv_res);
        } else {
            panic!("Client send failed unexpectedly: {:?}", res);
        }
    });

    // Server receives data but then drops the channel without sending/closing
    let server_task = tokio::spawn(async move {
        println!("Server: Receiving data...");
        let recv_res = server_chan.recv().await;
        println!("Server: Recv result: {:?}", recv_res);
        if let Ok((data, _chan)) = recv_res {
             println!("Server: Received '{}'. Dropping channel early.", data);
             // Drop _chan implicitly here, simulating unexpected closure before Send/Close
        } else {
             println!("Server: Recv failed unexpectedly: {:?}", recv_res);
             // If recv fails, the test might still pass if client gets expected error
        }
    });

    // Wait for tasks (important to ensure assertions run)
    client_task.await.expect("Client task panicked");
    server_task.await.expect("Server task panicked");
}
```

## Best Practices

1.  **Combine Approaches**: Use type-level tests for structure, runtime tests for logic, and compile-fail tests for misuse prevention.
2.  **Use In-Memory Channels**: Leverage `channel_pair::<_, ()>()` for most runtime tests to isolate protocol logic from network issues.
3.  **Test Concurrently**: Run client and server logic concurrently (e.g., `tokio::spawn`) to simulate real interaction.
4.  **Test All Paths**: For protocols with choices (`Choose`/`Offer`), ensure every branch is tested with separate runtime tests.
5.  **Test Error Conditions**: Explicitly test scenarios like unexpected channel closures, timeouts (if applicable), and potential protocol violations if possible at runtime (e.g., sending wrong data type if serialization allows).
6.  **Keep Tests Focused**: Each test should verify a specific aspect of the protocol or error handling.
7.  **Clear Assertions**: Use `assert_eq!`, `assert!`, and `matches!` to clearly check expected values, states, or errors.
8.  **Consider Integration Tests**: Optionally, add tests using real network I/O (e.g., TCP) to catch integration issues, but keep these separate from unit tests focusing on protocol logic.