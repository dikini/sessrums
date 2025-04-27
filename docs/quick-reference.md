# SEZ: Session Types Quick Reference

This document provides a concise reference for the SEZ session types library.

## Protocol Types

| Type | Description | Dual |
|------|-------------|------|
| `Send<T, P>` | Send a value of type `T`, then continue with protocol `P` | `Recv<T, P::Dual>` |
| `Recv<T, P>` | Receive a value of type `T`, then continue with protocol `P` | `Send<T, P::Dual>` |
| `End` | End the communication | `End` |
| `Offer<L, R>` | Offer a choice between protocols `L` and `R` | `Choose<L::Dual, R::Dual>` |
| `Choose<L, R>` | Make a choice between protocols `L` and `R` | `Offer<L::Dual, R::Dual>` |

## Channel API

### Creating a Channel

```rust
// Create a channel with a specific protocol and IO implementation
let chan = Chan::<MyProtocol, MyIO>::new(io);
```

### Send Method

```rust
// For Chan<Send<T, P>, IO>
let chan: Chan<Send<i32, P>, IO> = /* ... */;
let chan: Chan<P, IO> = chan.send(42).await?;
```

### Receive Method

```rust
// For Chan<Recv<T, P>, IO>
let chan: Chan<Recv<String, P>, IO> = /* ... */;
let (value, chan): (String, Chan<P, IO>) = chan.recv().await?;
```

### Close Method

```rust
// For Chan<End, IO>
let chan: Chan<End, IO> = /* ... */;
chan.close()?;
```

## Error Handling

```rust
match chan.send(42).await {
    Ok(chan) => {
        // Continue with the protocol
    },
    Err(e) => match e {
        Error::Io(io_err) => {
            // Handle IO error
        },
        Error::Protocol(msg) => {
            // Handle protocol error
        },
        Error::Connection(msg) => {
            // Handle connection error
        },
        Error::Serialization(msg) => {
            // Handle serialization error
        },
        Error::Deserialization(msg) => {
            // Handle deserialization error
        },
        Error::ChannelClosed => {
            // Handle closed channel
        },
    }
}
```

## Common Protocol Patterns

### Simple Request-Response

```rust
// Client sends a request and receives a response
type ClientProtocol = Send<Request, Recv<Response, End>>;

// Server receives a request and sends a response
type ServerProtocol = Recv<Request, Send<Response, End>>;
```

### Authentication with Success/Failure

```rust
// Client sends credentials and receives success or failure
type ClientProtocol = Send<Credentials, Recv<AuthResult, End>>;

// Server receives credentials and sends success or failure
type ServerProtocol = Recv<Credentials, Send<AuthResult, End>>;
```

### Multiple Exchanges

```rust
// Client sends multiple messages and receives responses
type ClientProtocol = Send<Msg1, Recv<Resp1, Send<Msg2, Recv<Resp2, End>>>>;

// Server receives multiple messages and sends responses
type ServerProtocol = Recv<Msg1, Send<Resp1, Recv<Msg2, Send<Resp2, End>>>>;
```

## IO Implementation

To create a custom IO implementation, implement the `Sender<T>` and `Receiver<T>` traits:

```rust
impl<T> Sender<T> for MyIO {
    type Error = MyError;
    
    fn send(&mut self, value: T) -> Result<(), Self::Error> {
        // Implementation
    }
}

impl<T> Receiver<T> for MyIO {
    type Error = MyError;
    
    fn recv(&mut self) -> Result<T, Self::Error> {
        // Implementation
    }
}
```

## Type Safety Examples

```rust
// Correct: Following the protocol
async fn correct(chan: Chan<Send<i32, Recv<String, End>>, IO>) {
    let chan = chan.send(42).await?;
    let (response, chan) = chan.recv().await?;
    chan.close()?;
}

// Incorrect: Would not compile (wrong order)
async fn incorrect1(chan: Chan<Send<i32, Recv<String, End>>, IO>) {
    let (response, chan) = chan.recv().await?; // Error!
    let chan = chan.send(42).await?;
    chan.close()?;
}

// Incorrect: Would not compile (wrong type)
async fn incorrect2(chan: Chan<Send<i32, Recv<String, End>>, IO>) {
    let chan = chan.send("hello").await?; // Error!
    let (response, chan) = chan.recv().await?;
    chan.close()?;
}
```

For more detailed documentation, see [Session Types Documentation](session-types-documentation.md).