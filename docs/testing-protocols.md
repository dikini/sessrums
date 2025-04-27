# Testing Session Type Protocols

This document provides guidance and examples for testing session type protocols in the sessrums library.

## Table of Contents

1. [Introduction](#introduction)
2. [Testing Approaches](#testing-approaches)
   - [Type-Level Testing](#type-level-testing)
   - [Runtime Testing](#runtime-testing)
   - [Compile-Fail Testing](#compile-fail-testing)
3. [Helper Functions](#helper-functions)
4. [Example Tests](#example-tests)
   - [Simple Protocol Test](#simple-protocol-test)
   - [Complex Protocol Test](#complex-protocol-test)
   - [Error Handling Test](#error-handling-test)
5. [Best Practices](#best-practices)

## Introduction

Testing session type protocols is essential to ensure that your communication patterns work as expected. The sessrums library provides several approaches to testing protocols:

1. **Type-level testing**: Verifying that protocol types have the expected properties
2. **Runtime testing**: Testing the actual communication behavior at runtime
3. **Compile-fail testing**: Ensuring that invalid protocol usage fails to compile

## Testing Approaches

### Type-Level Testing

Type-level testing verifies the properties of protocol types without executing any code. This approach uses Rust's type system to check that protocols have the expected structure and duality relationships.

```rust
// Define protocol types
type ClientProtocol = Send<i32, Recv<String, End>>;
type ServerProtocol = Recv<i32, Send<String, End>>;

// Verify that ClientProtocol and ServerProtocol implement the Protocol trait
assert_protocol::<ClientProtocol>();
assert_protocol::<ServerProtocol>();

// Verify that ServerProtocol is the dual of ClientProtocol
assert_dual::<ClientProtocol, ServerProtocol>();
```

### Runtime Testing

Runtime testing verifies the actual communication behavior of protocols. This approach creates channels with specific protocols and tests sending and receiving values.

```rust
#[tokio::test]
async fn test_ping_pong_protocol() {
    // Create a channel with a Send<i32, Recv<String, End>> protocol
    let chan = Chan::<Send<i32, Recv<String, End>>, TestIO>::new(test_io);
    
    // Send a value
    let chan = chan.send(42).await.unwrap();
    
    // Receive a value
    let (response, chan) = chan.recv().await.unwrap();
    assert_eq!(response, "Hello");
    
    // Close the channel
    chan.close().unwrap();
}
```

### Compile-Fail Testing

Compile-fail testing ensures that invalid protocol usage fails to compile. This approach uses the `trybuild` crate to verify that code with protocol violations produces the expected compilation errors.

```rust
#[test]
fn test_compile_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
```

Example compile-fail test file:

```rust
// tests/compile_fail/wrong_order.rs
use sessrums::chan::Chan;
use sessrums::proto::{Send, Recv, End};

async fn incorrect_protocol_usage(chan: Chan<Send<i32, Recv<String, End>>, ()>) {
    // Error: The protocol requires sending an i32 first, but we're trying to receive
    let (response, chan) = chan.recv().await.unwrap();
}

fn main() {}
```

## Helper Functions

The sessrums library provides several helper functions for testing protocols:

```rust
// Assert that a type implements the Protocol trait
pub fn assert_protocol<P: Protocol>() {}

// Assert that two types have the correct duality relationship
pub fn assert_dual<P: Protocol, Q: Protocol>()
where
    P::Dual: Protocol,
{
    fn assert_same_type<T, U>() where T: Protocol, U: Protocol {}
    assert_same_type::<P::Dual, Q>();
}

// Assert that a type is its own dual
pub fn assert_self_dual<P: Protocol>()
where
    P::Dual: Protocol,
{
    fn assert_same_type<T, U>() where T: Protocol, U: Protocol {}
    assert_same_type::<P, P::Dual>();
}

// Create a mock channel for testing
pub fn mock_channel<P: Protocol, IO>() -> Chan<P, IO>
where
    IO: Default,
{
    Chan::new(IO::default())
}
```

## Example Tests

### Simple Protocol Test

This example tests a simple ping-pong protocol:

```rust
// Define the protocol types
type PingPongClient = Send<i32, Recv<String, End>>;
type PingPongServer = Recv<i32, Send<String, End>>;

#[tokio::test]
async fn test_ping_pong_protocol() {
    // Verify that PingPongClient and PingPongServer implement the Protocol trait
    assert_protocol::<PingPongClient>();
    assert_protocol::<PingPongServer>();
    
    // Verify that PingPongServer is the dual of PingPongClient
    assert_dual::<PingPongClient, PingPongServer>();
    
    // Create a test IO implementation
    struct TestIO {
        client_to_server: Option<i32>,
        server_to_client: Option<String>,
    }
    
    // Implement Sender<i32> for TestIO
    impl Sender<i32> for TestIO {
        type Error = ();
        
        fn send(&mut self, value: i32) -> Result<(), Self::Error> {
            self.client_to_server = Some(value);
            Ok(())
        }
    }
    
    // Implement Receiver<i32> for TestIO
    impl Receiver<i32> for TestIO {
        type Error = ();
        
        fn recv(&mut self) -> Result<i32, Self::Error> {
            self.client_to_server.take().ok_or(())
        }
    }
    
    // Implement Sender<String> for TestIO
    impl Sender<String> for TestIO {
        type Error = ();
        
        fn send(&mut self, value: String) -> Result<(), Self::Error> {
            self.server_to_client = Some(value);
            Ok(())
        }
    }
    
    // Implement Receiver<String> for TestIO
    impl Receiver<String> for TestIO {
        type Error = ();
        
        fn recv(&mut self) -> Result<String, Self::Error> {
            self.server_to_client.take().ok_or(())
        }
    }
    
    // Create the IO implementation
    let io = TestIO {
        client_to_server: None,
        server_to_client: None,
    };
    
    // Create client channel
    let client_chan = Chan::<PingPongClient, _>::new(io);
    
    // Client sends an i32 value
    let client_chan = client_chan.send(42).await.unwrap();
    
    // Create server channel with the client's message
    let io = TestIO {
        client_to_server: Some(42),
        server_to_client: None,
    };
    let server_chan = Chan::<PingPongServer, _>::new(io);
    
    // Server receives the i32 value
    let (value, server_chan) = server_chan.recv().await.unwrap();
    assert_eq!(value, 42);
    
    // Server sends a String response
    let server_chan = server_chan.send("Hello".to_string()).await.unwrap();
    
    // Create client channel with the server's response
    let io = TestIO {
        client_to_server: None,
        server_to_client: Some("Hello".to_string()),
    };
    let client_chan = Chan::<Recv<String, End>, _>::new(io);
    
    // Client receives the String response
    let (response, client_chan) = client_chan.recv().await.unwrap();
    assert_eq!(response, "Hello");
    
    // Both sides close the connection
    client_chan.close().unwrap();
    server_chan.close().unwrap();
}
```

### Complex Protocol Test

This example tests a more complex protocol with choices:

```rust
// Define a protocol with choices
type ClientProtocol = Send<Auth, Recv<AuthResult, Choose<Success, Failure>>>;
type ServerProtocol = Recv<Auth, Send<AuthResult, Offer<Success, Failure>>>;

// Define the authentication types
struct Auth { username: String, password: String }
enum AuthResult { Success, Failure }

// Define the success and failure protocols
type Success = Send<String, End>;
type Failure = End;

#[tokio::test]
async fn test_auth_protocol() {
    // Verify that the protocols implement the Protocol trait
    assert_protocol::<ClientProtocol>();
    assert_protocol::<ServerProtocol>();
    
    // Verify that ServerProtocol is the dual of ClientProtocol
    assert_dual::<ClientProtocol, ServerProtocol>();
    
    // Create a test IO implementation
    // (implementation details omitted for brevity)
    
    // Test the successful authentication path
    // 1. Client sends authentication credentials
    // 2. Server receives credentials and sends success result
    // 3. Client receives success result and chooses the success branch
    // 4. Client sends a message on the success branch
    // 5. Server receives the message
    // 6. Both sides close the connection
    
    // Test the failed authentication path
    // 1. Client sends authentication credentials
    // 2. Server receives credentials and sends failure result
    // 3. Client receives failure result and chooses the failure branch
    // 4. Both sides close the connection
}
```

### Error Handling Test

This example tests error handling in protocols:

```rust
#[tokio::test]
async fn test_error_handling() {
    // Create a custom IO implementation that fails on recv
    struct FailingIO;
    
    impl Sender<String> for FailingIO {
        type Error = ();
        
        fn send(&mut self, _value: String) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    
    impl Receiver<String> for FailingIO {
        type Error = ();
        
        fn recv(&mut self) -> Result<String, Self::Error> {
            Err(())
        }
    }
    
    // Create a channel with a Recv<String, End> protocol
    let chan = Chan::<Recv<String, End>, _>::new(FailingIO);
    
    // Attempt to receive a value (should fail)
    let result = chan.recv().await;
    assert!(result.is_err());
    
    // Check that the error is of the expected type
    match result {
        Ok(_) => panic!("Expected an error"),
        Err(e) => {
            match e {
                Error::Io(_) => {
                    // Expected error type
                },
                _ => panic!("Unexpected error type: {:?}", e),
            }
        }
    }
}
```

## Best Practices

1. **Test both type-level and runtime properties**: Verify both the type-level properties of protocols (using `assert_protocol` and `assert_dual`) and their runtime behavior.

2. **Use mock IO implementations**: Create custom IO implementations for testing that allow you to control the flow of data and simulate different scenarios.

3. **Test error handling**: Ensure that your code handles errors correctly by testing error cases explicitly.

4. **Test protocol violations**: Use compile-fail tests to verify that invalid protocol usage is caught at compile time.

5. **Test complex protocols incrementally**: For complex protocols, test each part of the protocol separately before testing the entire protocol.

6. **Use helper functions**: Create helper functions for common testing patterns to reduce duplication and improve readability.

7. **Document test cases**: Clearly document what each test is verifying to make it easier to understand and maintain.

8. **Test edge cases**: Test edge cases such as empty messages, large messages, and concurrent communication.

9. **Test with real IO implementations**: In addition to mock IO implementations, test with real IO implementations (e.g., TCP sockets) to ensure that your protocols work in practice.

10. **Use property-based testing**: Consider using property-based testing frameworks like `proptest` to generate test cases automatically.

By following these best practices, you can ensure that your session type protocols are correct, robust, and maintainable.