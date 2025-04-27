# Error Handling in SEZ

This document provides detailed information about error handling in the SEZ session types library.

## Error Type

The library defines an `Error` enum that represents the various error conditions that might arise when using session-typed channels for communication:

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

## Error Variants

### Io Error

```rust
Error::Io(io::Error)
```

This variant represents errors that occur in the underlying IO implementation. These errors are typically related to the actual sending or receiving of data through the underlying communication mechanism.

Examples:
- Network connection failures
- File system errors
- Timeout errors

### Protocol Error

```rust
Error::Protocol(&'static str)
```

This variant represents errors related to protocol violations, such as unexpected messages or type mismatches. These errors indicate that the communication protocol was not followed correctly.

Examples:
- Receiving an unexpected message type
- Protocol state mismatch
- Invalid protocol transition

### Connection Error

```rust
Error::Connection(&'static str)
```

This variant represents errors related to connection establishment or termination. These errors indicate issues with setting up or tearing down the communication channel.

Examples:
- Connection refused
- Connection reset
- Authentication failure

### Serialization Error

```rust
Error::Serialization(&'static str)
```

This variant represents errors that occur when serializing data to be sent over the channel. These errors indicate issues with converting data to a format that can be transmitted.

Examples:
- Invalid data format
- Unsupported data type
- Serialization buffer overflow

### Deserialization Error

```rust
Error::Deserialization(&'static str)
```

This variant represents errors that occur when deserializing received data. These errors indicate issues with converting received data back to the expected type.

Examples:
- Corrupted data
- Incompatible data format
- Missing required fields

### Channel Closed Error

```rust
Error::ChannelClosed
```

This variant represents the error that occurs when attempting to communicate on a closed channel. This error indicates that the channel has been closed and can no longer be used for communication.

Examples:
- Sending on a closed channel
- Receiving from a closed channel

## Error Handling Patterns

### Basic Error Handling

```rust
match chan.send(42).await {
    Ok(chan) => {
        // Continue with the protocol
        println!("Value sent successfully");
    },
    Err(e) => {
        // Handle the error
        eprintln!("Error sending value: {}", e);
    }
}
```

### Detailed Error Handling

```rust
match chan.send(42).await {
    Ok(chan) => {
        // Continue with the protocol
        println!("Value sent successfully");
    },
    Err(e) => match e {
        Error::Io(io_err) => {
            eprintln!("IO error: {}", io_err);
            // Handle IO error specifically
        },
        Error::Protocol(msg) => {
            eprintln!("Protocol error: {}", msg);
            // Handle protocol error specifically
        },
        Error::Connection(msg) => {
            eprintln!("Connection error: {}", msg);
            // Handle connection error specifically
        },
        Error::Serialization(msg) => {
            eprintln!("Serialization error: {}", msg);
            // Handle serialization error specifically
        },
        Error::Deserialization(msg) => {
            eprintln!("Deserialization error: {}", msg);
            // Handle deserialization error specifically
        },
        Error::ChannelClosed => {
            eprintln!("Channel closed");
            // Handle closed channel specifically
        },
    }
}
```

### Using the `?` Operator

```rust
async fn run_protocol(chan: Chan<MyProtocol, MyIO>) -> Result<(), Error> {
    // Send a value
    let chan = chan.send(42).await?;
    
    // Receive a response
    let (response, chan) = chan.recv().await?;
    
    // Close the channel
    chan.close()?;
    
    Ok(())
}
```

### Custom Error Conversion

You can implement custom error conversion for your IO implementation's error types:

```rust
impl From<MyCustomError> for Error {
    fn from(err: MyCustomError) -> Self {
        Error::Io(io::Error::new(
            io::ErrorKind::Other,
            format!("Custom error: {:?}", err),
        ))
    }
}
```

## Error Propagation in IO Implementations

When implementing the `Sender<T>` and `Receiver<T>` traits for your custom IO type, you need to define an associated `Error` type:

```rust
impl<T> Sender<T> for MyIO {
    type Error = MyError;
    
    fn send(&mut self, value: T) -> Result<(), Self::Error> {
        // Implementation that can return MyError
    }
}
```

The library will convert your custom error type to the library's `Error` type when calling the `send` and `recv` methods on a `Chan`:

```rust
// In the Chan implementation
self.io_mut().send(value).map_err(|e| {
    // Convert the IO-specific error to our Error type
    crate::error::Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Send error: {:?}", e),
    ))
})?;
```

## Example: Handling Connection Errors

```rust
async fn connect_and_communicate() -> Result<(), Error> {
    // Attempt to establish a connection
    let io = match establish_connection() {
        Ok(io) => io,
        Err(e) => {
            return Err(Error::Connection("Failed to establish connection"));
        }
    };
    
    // Create a channel with the established connection
    let chan = Chan::<ClientProtocol, _>::new(io);
    
    // Send a message
    let chan = match chan.send("Hello").await {
        Ok(chan) => chan,
        Err(e) => {
            // Log the error and return
            eprintln!("Failed to send message: {}", e);
            return Err(e);
        }
    };
    
    // Receive a response
    let (response, chan) = chan.recv().await?;
    
    // Close the channel
    chan.close()?;
    
    Ok(())
}
```

## Example: Custom Error Handling with Retries

```rust
async fn send_with_retry<T, P, IO>(
    mut chan: Chan<Send<T, P>, IO>,
    value: T,
    max_retries: usize,
) -> Result<Chan<P, IO>, Error>
where
    T: Clone,
    P: Protocol,
    IO: Sender<T>,
    <IO as Sender<T>>::Error: std::fmt::Debug,
{
    let mut retries = 0;
    
    loop {
        match chan.send(value.clone()).await {
            Ok(new_chan) => return Ok(new_chan),
            Err(e) => {
                if retries >= max_retries {
                    return Err(e);
                }
                
                match e {
                    Error::Io(_) => {
                        // Retry on IO errors
                        retries += 1;
                        eprintln!("IO error, retrying ({}/{})", retries, max_retries);
                        // In a real implementation, you might want to add a delay here
                    },
                    _ => {
                        // Don't retry on other types of errors
                        return Err(e);
                    }
                }
            }
        }
    }
}
```

## Best Practices for Error Handling

1. **Be specific about error types**: Use the appropriate error variant for each type of error to provide clear information about what went wrong.

2. **Provide meaningful error messages**: Include specific details in error messages to help diagnose and fix issues.

3. **Handle errors at the appropriate level**: Some errors should be handled locally, while others should be propagated up the call stack.

4. **Consider retries for transient errors**: IO errors and connection errors may be transient and worth retrying.

5. **Log errors for debugging**: Log errors with appropriate context to help diagnose issues in production.

6. **Clean up resources on error**: Ensure that resources are properly cleaned up when errors occur, especially when using the `?` operator for early returns.

7. **Test error handling code**: Write tests that specifically exercise error handling paths to ensure they work correctly.

By following these best practices, you can create robust applications that handle errors gracefully and provide a good user experience even when things go wrong.