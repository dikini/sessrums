# Error Handling in sessrums

This document explains how to handle errors when using the sessrums session types library.

## Error Type

The library uses a single `Error` enum for all potential issues:

```rust
use std::io; // Required for Error::Io variant

#[derive(Debug)]
pub enum Error {
    Io(io::Error), // Errors from the underlying transport (network, file, etc.)
    Protocol(&'static str), // Mismatch between expected and actual protocol steps
    Connection(&'static str), // Issues during connection setup or teardown
    Serialization(&'static str), // Problems encoding data for sending
    Deserialization(&'static str), // Problems decoding received data
    ChannelClosed, // Attempted operation on a closed channel
}

// Define a Result type alias for convenience
pub type Result<T> = std::result::Result<T, Error>;
```

## Error Variants Explained

- **`Error::Io(io::Error)`**: Wraps standard I/O errors (e.g., network connection refused, broken pipe, file not found). These originate from the underlying communication layer.
- **`Error::Protocol(&'static str)`**: Indicates a violation of the session type rules (e.g., trying to `send` when a `recv` is expected, receiving the wrong data type, offering/choosing incorrectly). The message provides context. This usually points to a logic error in one of the communicating parties.
- **`Error::Connection(&'static str)`**: Specific to connection phases (e.g., failure to connect, authentication issues).
- **`Error::Serialization(&'static str)`**: Occurs if data cannot be serialized before sending (e.g., using an unsupported format or data structure).
- **`Error::Deserialization(&'static str)`**: Occurs if incoming data cannot be deserialized into the expected type (e.g., corrupted data, format mismatch between sender and receiver).
- **`Error::ChannelClosed`**: Happens when you try to use a channel that has already been explicitly closed or has terminated its protocol (`End`).

## Handling Errors

Most channel operations (`send`, `recv`, `choose`, `offer`, `close`) return `sessrums::error::Result<T>`. You should handle potential errors using standard Rust patterns.

### Using `match`

You can explicitly match on the `Result` to handle success and error cases, potentially inspecting the specific `Error` variant:

```rust
use sessrums::proto::{Chan, Send, End};
use sessrums::error::{Error, Result};
use std::io; // For io::Error matching

async fn send_data<T, P, IO>(chan: Chan<Send<T, P>, IO>, data: T)
where
    // Simplified bounds for example clarity
    P: sessrums::proto::Protocol,
    IO: sessrums::io::Sender<T> + Send + 'static,
    <IO as sessrums::io::Sender<T>>::Error: std::fmt::Debug + Send,
    T: Send + 'static,
{
    match chan.send(data).await {
        Ok(next_chan) => {
            println!("Data sent successfully.");
            // Proceed with next_chan...
        }
        Err(e) => match e {
            Error::Io(io_err) => {
                eprintln!("Network/IO error during send: {}", io_err);
                // Example: Log the error and potentially signal failure upstream
                // log::error!("Send failed due to IO error: {}", io_err);
                // Consider specific handling based on io_err.kind()
            }
            Error::Protocol(msg) => {
                // This indicates a logic error if types match Send<T, P>.
                // The channel state expected a different operation.
                eprintln!("Protocol error during send: {}. Check protocol logic.", msg);
            }
            Error::Serialization(msg) => {
                eprintln!("Serialization error during send: {}", msg);
                // Check data format or serializer implementation compatibility.
            }
            Error::ChannelClosed => {
                eprintln!("Attempted to send on a closed channel.");
            }
            // Connection errors less likely during send/recv after setup
            Error::Connection(msg) => {
                 eprintln!("Unexpected connection error during send: {}", msg);
            }
            // Deserialization errors only happen on recv
            Error::Deserialization(msg) => {
                 eprintln!("Unexpected deserialization error during send: {}", msg);
            }
        },
    }
}
```

### Using the `?` Operator

For propagating errors up the call stack, the `?` operator is concise. Ensure your function returns `sessrums::error::Result<T>` or a type that `sessrums::error::Error` can be converted into.

```rust
use sessrums::api::{RequestClient, request_response_pair};
use sessrums::proto::{Chan, Protocol};
use sessrums::error::{Result, Error};

// Define a simple request-response protocol
type ClientProto = RequestClient<String, i32>;

async fn run_client_protocol<IO>(chan: Chan<ClientProto, IO>) -> Result<()>
where
    // Simplified bounds for example clarity
    IO: sessrums::io::Sender<String> + sessrums::io::Receiver<i32> + Send + 'static,
    <IO as sessrums::io::Sender<String>>::Error: std::fmt::Debug + Send,
    <IO as sessrums::io::Receiver<i32>>::Error: std::fmt::Debug + Send,
{
    println!("Client: Sending request...");
    let chan = chan.send("GetStatus".to_string()).await?; // Propagates error if send fails

    println!("Client: Receiving response...");
    let (response, chan) = chan.recv().await?; // Propagates error if recv fails
    println!("Client: Received status code: {}", response);

    println!("Client: Closing channel...");
    chan.close()?; // Propagates error if close fails (e.g., already closed)

    println!("Client: Protocol finished successfully.");
    Ok(())
}

async fn main_example() {
    // Example setup with in-memory channel (`()`)
    let (client_chan, _server_chan) = request_response_pair::<String, i32, ()>();

    if let Err(e) = run_client_protocol(client_chan).await {
        eprintln!("Client protocol failed: {:?}", e);
        // Handle the error from the top level
    }
}
```

### Handling Receive Errors

Receiving data (`recv`) is particularly prone to specific errors like `Deserialization` or `Protocol` mismatches, in addition to `Io` errors or `ChannelClosed`.

```rust
use sessrums::proto::{Chan, Recv, End};
use sessrums::error::{Error, Result};
use std::io;

async fn receive_data<T, P, IO>(chan: Chan<Recv<T, P>, IO>) -> Result<(T, Chan<P, IO>)>
where
    // Simplified bounds for example clarity
    P: sessrums::proto::Protocol,
    IO: sessrums::io::Receiver<T> + Send + 'static,
    <IO as sessrums::io::Receiver<T>>::Error: std::fmt::Debug + Send,
    T: serde::de::DeserializeOwned + Send + 'static, // Ensure T is deserializable
{
    match chan.recv().await {
        Ok((data, next_chan)) => {
            println!("Data received successfully.");
            Ok((data, next_chan))
        }
        Err(e) => match e {
            Error::Io(io_err) => {
                eprintln!("Network/IO error during receive: {}", io_err);
                Err(Error::Io(io_err)) // Propagate or handle
            }
            Error::Protocol(msg) => {
                // This indicates a logic error - expecting Recv<T, P> but
                // the other end performed an incompatible action (e.g., sent wrong type, closed early).
                eprintln!("Protocol error during receive: {}. Check protocol logic.", msg);
                Err(Error::Protocol(msg))
            }
            Error::Deserialization(msg) => {
                eprintln!("Deserialization error during receive: {}. Data format mismatch?", msg);
                // Could indicate corrupted data, incompatible types/serializers between peers,
                // or receiving data not matching the expected type T.
                Err(Error::Deserialization(msg))
            }
            Error::ChannelClosed => {
                eprintln!("Attempted to receive on a closed channel.");
                Err(Error::ChannelClosed)
            }
            // Connection errors less likely here after setup
            Error::Connection(msg) => {
                 eprintln!("Unexpected connection error during receive: {}", msg);
                 Err(Error::Connection(msg))
            }
            // Serialization errors only happen on send
            Error::Serialization(msg) => {
                 eprintln!("Unexpected serialization error during receive: {}", msg);
                 Err(Error::Serialization(msg))
            }
        },
    }
}
```

### Custom IO Error Conversion

If you implement custom IO layers (`Sender`/`Receiver` traits), your associated `Error` type needs to be convertible into `sessrums::error::Error`. The library typically expects your error to implement `Into<io::Error>` or you might need a specific `From` implementation if wrapping is complex. The goal is usually to map your custom transport errors to `SessrumsError::Io`.

```rust
use std::io;
use sessrums::error::Error as SessrumsError;

// Example custom error for a hypothetical transport
#[derive(Debug)]
pub enum MyTransportError {
    Timeout,
    BufferFull,
    Internal(String),
}

// Implement conversion into std::io::Error (common pattern)
impl From<MyTransportError> for io::Error {
    fn from(err: MyTransportError) -> Self {
        let kind = match err {
            MyTransportError::Timeout => io::ErrorKind::TimedOut,
            MyTransportError::BufferFull => io::ErrorKind::WriteZero,
            MyTransportError::Internal(_) => io::ErrorKind::Other,
        };
        io::Error::new(kind, format!("{:?}", err))
    }
}

// If your Sender/Receiver uses MyTransportError and returns it,
// sessrums operations will typically wrap it automatically:
// MyTransportError -> io::Error -> SessrumsError::Io(io::Error)
// You generally don't need `impl From<MyTransportError> for SessrumsError`.
```

## Example: Handling Connection Errors

Connection errors typically occur during the initial setup phase using functions like `connect_with_protocol`.

```rust
use sessrums::api::{connect_with_protocol, RequestClient};
use sessrums::connect::TcpConnectInfo; // Example: TCP connection
use sessrums::error::{Result, Error};
use std::net::SocketAddr;

async fn connect_and_run() -> Result<()> {
    let addr: SocketAddr = "127.0.0.1:12345".parse().expect("Invalid address");
    let conn_info = TcpConnectInfo::new(addr);

    println!("Attempting to connect to {}...", addr);

    // connect_with_protocol returns Result<Chan<P, IO>>
    let client_chan = match connect_with_protocol::<RequestClient<String, i32>, _, _>(conn_info).await {
        Ok(chan) => {
            println!("Connection successful.");
            chan
        }
        Err(e) => {
            eprintln!("Connection failed: {:?}", e);
            // Handle specific connection errors if needed
            match e {
                Error::Io(ref io_err) if io_err.kind() == std::io::ErrorKind::ConnectionRefused => {
                    eprintln!("Server is not running or refused the connection.");
                }
                Error::Connection(msg) => {
                     eprintln!("Specific connection protocol error: {}", msg);
                }
                _ => {
                    eprintln!("An unexpected error occurred during connection: {:?}", e);
                }
            }
            return Err(e); // Propagate the error
        }
    };

    // Proceed with the protocol using client_chan...
    // Example: run_client_protocol(client_chan).await?;

    Ok(())
}
```

## Best Practices

1.  **Handle `Result`**: Always check the `Result` returned by channel operations. Unhandled errors often lead to panics or incorrect program state.
2.  **Use `?` for Propagation**: Use the `?` operator in functions returning `Result` for cleaner error propagation up the call stack.
3.  **Match Specific Errors**: When necessary, `match` on the `Error` enum to handle different error conditions appropriately (e.g., retrying specific `Io` errors, logging `Protocol` errors which indicate bugs, handling `Deserialization` errors gracefully).
4.  **Log Errors**: Log errors with sufficient context (e.g., current protocol step, peer information if available) to aid debugging.
5.  **Consider Retries**: Transient errors (some `Io` or `Connection` errors like timeouts or temporary network issues) might be suitable for retries, potentially with backoff delays. `Protocol` or `Deserialization` errors are usually not retryable.
6.  **Clean Up**: Ensure resources (like connections) are handled correctly, even on error paths. Rust's ownership and RAII help, but be mindful in complex async scenarios or when using external resources.
7.  **Test Error Paths**: Write tests specifically for error conditions (e.g., simulating connection failures, sending incorrectly typed/formatted data, closing channels prematurely) to verify your error handling logic.