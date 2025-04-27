# Error Examples

This document provides an overview of all the error examples implemented in the sessrums library tests. Each example demonstrates how the session type system prevents different kinds of protocol errors at compile time.

## Error 1: Deadlock (Recv/Recv)

### Description

This example demonstrates a protocol that fails to compile because both client and server try to receive an i32 first, which would cause a deadlock.

### Type-Level Representation

```rust
// Client: Receive an i32, then end
type ClientProto = Recv<i32, End>;
// Server: Receive an i32, then end (Not the dual of ClientProto!)
type ServerProto = Recv<i32, End>; // Should be Send<i32, End>
```

### Visual Diagram

```text
Client                 Server
  |                      |
  |  ?i32                |  ?i32
  | <---- DEADLOCK ----> |
  |                      |
 End                    End
```

Legend:
- ?T: Receive a value of type T
- !T: Send a value of type T
- DEADLOCK: Both parties waiting to receive, neither sending

### Why This Protocol Fails

In session types, communication between two parties must be complementary to avoid deadlocks. When one party receives a message, the other party must send a message. This complementary relationship is formalized through the concept of "duality".

In this example, both parties are waiting to receive a message, but neither is sending one. This creates a deadlock situation where both parties are blocked forever waiting for messages that will never arrive.

### How Session Types Prevent This Error

The session type system prevents this error at compile time by enforcing duality between communicating parties. For any protocol P, there must exist a dual protocol P::Dual that represents the complementary behavior.

For Recv<T, P>, the dual is Send<T, P::Dual>. This means:
- The dual of Recv<i32, End> is Send<i32, End>
- The dual of Send<i32, End> is Recv<i32, End>
- The dual of End is End (termination is symmetric)

When we try to create a channel pair with non-dual protocols, the type system rejects it, preventing the deadlock at compile time.

### Correct Version

A correct version would have complementary actions:

```text
Client                 Server
  |                      |
  |  ?i32                |  !i32
  | <------------------- |
  |                      |
 End                    End
```

## Error 2: Deadlock (Send/Send)

### Description

This example demonstrates a protocol that fails to compile because both client and server try to send an i32 first, which would cause a deadlock.

### Type-Level Representation

```rust
// Client: Send an i32, then end
type ClientProto = Send<i32, End>;
// Server: Send an i32, then end (Not the dual of ClientProto!)
type ServerProto = Send<i32, End>; // Should be Recv<i32, End>
```

### Visual Diagram

```text
Client                 Server
  |                      |
  |  !i32                |  !i32
  | <---- DEADLOCK ----> |
  |                      |
 End                    End
```

Legend:
- ?T: Receive a value of type T
- !T: Send a value of type T
- DEADLOCK: Both parties trying to send, neither receiving

### Why This Protocol Fails

In session types, communication between two parties must be complementary to avoid deadlocks. When one party sends a message, the other party must receive a message. This complementary relationship is formalized through the concept of "duality".

In this example, both parties are trying to send a message, but neither is receiving one. This creates a deadlock situation where both parties might block trying to send messages that will never be received.

### How Session Types Prevent This Error

The session type system prevents this error at compile time by enforcing duality between communicating parties. For any protocol P, there must exist a dual protocol P::Dual that represents the complementary behavior.

For Send<T, P>, the dual is Recv<T, P::Dual>. This means:
- The dual of Send<i32, End> is Recv<i32, End>
- The dual of Recv<i32, End> is Send<i32, End>
- The dual of End is End (termination is symmetric)

When we try to create a channel pair with non-dual protocols, the type system rejects it, preventing the deadlock at compile time.

### Correct Version

A correct version would have complementary actions:

```text
Client                 Server
  |                      |
  |  !i32                |  ?i32
  | -------------------> |
  |                      |
 End                    End
```

## Error 3: Type Mismatch

### Description

This example demonstrates a protocol that fails to compile because the client sends an i32, but the server expects to receive a String.

### Type-Level Representation

```rust
// Client: Send an i32, then end
type ClientProto = Send<i32, End>;
// Server: Receive a String, then end (Not the dual of ClientProto!)
type ServerProto = Recv<String, End>; // Should be Recv<i32, End>
```

### Visual Diagram

```text
Client                 Server
  |                      |
  |  !i32                |  ?String
  | <---- MISMATCH ----> |
  |                      |
 End                    End
```

Legend:
- ?T: Receive a value of type T
- !T: Send a value of type T
- MISMATCH: Type mismatch between sent and received values

### Why This Protocol Fails

In session types, communication between two parties must have matching types to ensure type safety. When one party sends a message of type T, the other party must receive a message of the same type T. This type matching is enforced through the concept of "duality".

In this example, the client is sending an i32, but the server is expecting a String. This creates a type mismatch that would cause runtime errors if allowed to compile.

### How Session Types Prevent This Error

The session type system prevents this error at compile time by enforcing duality between communicating parties. For any protocol P, there must exist a dual protocol P::Dual that represents the complementary behavior with matching types.

For Send<T, P>, the dual is Recv<T, P::Dual> (with the same type T). This means:
- The dual of Send<i32, End> is Recv<i32, End> (not Recv<String, End>)
- The dual of Recv<String, End> is Send<String, End> (not Send<i32, End>)

When we try to create a channel pair with non-dual protocols that have mismatched types, the type system rejects it, preventing type errors at compile time.

### Correct Version

A correct version would have matching types:

```text
Client                 Server
  |                      |
  |  !i32                |  ?i32
  | -------------------> |
  |                      |
 End                    End
```

## Error 4: Unexpected End

### Description

This example demonstrates a protocol that fails to compile because the client sends an i32 and terminates, but the server expects to send a bool after receiving the i32.

### Type-Level Representation

```rust
// Client: Send an i32, then end
type ClientProto = Send<i32, End>;
// Server: Receive an i32, send a bool, then end (Not the dual of ClientProto!)
type ServerProto = Recv<i32, Send<bool, End>>; // Should be Recv<i32, End>
```

### Visual Diagram

```text
Client                 Server
  |                      |
  |  !i32                |  ?i32
  | -------------------> |
  |                      |
 End                     |  !bool
  |                      |
  |       MISMATCH       |
  |                      |
  X                     End
```

Legend:
- ?T: Receive a value of type T
- !T: Send a value of type T
- MISMATCH: Protocol continuation mismatch
- X: Client has ended the session

### Why This Protocol Fails

In session types, both parties must agree on the entire communication sequence. When one party expects the session to end but the other expects it to continue, this creates a protocol mismatch that would cause runtime errors if allowed to compile.

In this example, after the initial i32 exchange, the client expects to terminate the session, but the server expects to continue by sending a bool. This mismatch in continuation protocols (End vs Send<bool, End>) would lead to a runtime error where the server attempts to send data to a client that has already closed the connection.

### How Session Types Prevent This Error

The session type system prevents this error at compile time by enforcing duality between communicating parties. For any protocol P, there must exist a dual protocol P::Dual that represents the complementary behavior with matching continuations.

For Send<T, P>, the dual is Recv<T, P::Dual>. This means:
- The dual of Send<i32, End> is Recv<i32, End>
- The dual of Recv<i32, Send<bool, End>> is Send<i32, Recv<bool, End>>

When we try to create a channel pair with non-dual protocols that have mismatched continuations, the type system rejects it, preventing protocol errors at compile time.

### Correct Version

A correct version would have matching continuations:

```text
Client                 Server
  |                      |
  |  !i32                |  ?i32
  | -------------------> |
  |                      |
  |  ?bool               |  !bool
  | <------------------- |
  |                      |
 End                    End
```

## Summary of Error Prevention

The session type system prevents several kinds of errors at compile time:

1. **Deadlock Prevention**: By enforcing duality between communicating parties, the session type system ensures that when one party sends, the other receives, and vice versa. This prevents deadlocks where both parties are waiting to send or both are waiting to receive.

2. **Type Safety**: By ensuring that the types match between sending and receiving parties, the session type system prevents type errors that would occur if, for example, one party sends an i32 but the other expects a String.

3. **Protocol Completion**: By enforcing that both parties agree on the entire communication sequence, the session type system prevents errors where one party expects the session to end but the other expects it to continue.

These compile-time guarantees make session types a powerful tool for designing and implementing communication protocols that are free from common errors.