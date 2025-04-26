# Protocol Examples

This document provides an overview of all the protocol examples implemented in the sez library tests. Each example demonstrates different aspects of session types and how they can be used to model various communication patterns.

## Protocol 1: Simple Ping-Pong

### Description

A simple ping-pong protocol where:
- Client sends an i32 value
- Server receives the i32 value
- Server sends a String response
- Client receives the String response
- Both sides close the connection

### Type-Level Representation

```rust
// Client: Send an i32, then receive a String, then end
type PingPongClient = Send<i32, Recv<String, End>>;
// Server: Receive an i32, then send a String, then end
type PingPongServer = Recv<i32, Send<String, End>>;
```

### Visual Diagram

```text
                  PingPongClient                 PingPongServer
                  --------------                 --------------
                        |                              |
                        |        Send(i32)            |
                        | ---------------------------> |
                        |                              |
                        |        Recv(String)          |
                        | <--------------------------- |
                        |                              |
                        |           End                |
                        | - - - - - - - - - - - - - - -|
                        |                              |
```

### Key Concepts Demonstrated

- **Type-level Protocol Definition**: Using `Send<T, P>`, `Recv<T, P>`, and `End` types to define a protocol
- **Duality**: The client and server protocols are duals of each other
- **Type Safety**: Ensuring that the correct types are sent and received at each step
- **Protocol Completion**: Both sides follow the protocol to completion

## Protocol 2: Request/Response

### Description

A request/response protocol where:
- Client sends a String request
- Server receives the String request
- Server sends a boolean response
- Client receives the boolean response
- Both sides close the connection

### Type-Level Representation

```rust
// Client: Send a String request, then receive a bool response, then end
type ReqResClient = Send<String, Recv<bool, End>>;
// Server: Receive a String request, then send a bool response, then end
type ReqResServer = Recv<String, Send<bool, End>>;
```

### Visual Diagram

```text
                  ReqResClient                    ReqResServer
                  ------------                    ------------
                        |                              |
                        |        Send(String)          |
                        | ---------------------------> |
                        |                              |
                        |        Recv(bool)            |
                        | <--------------------------- |
                        |                              |
                        |           End                |
                        | - - - - - - - - - - - - - - -|
                        |                              |
```

### Key Concepts Demonstrated

- **Different Message Types**: Using different types (String and bool) for request and response
- **API-like Interaction**: Modeling a typical request/response pattern common in APIs
- **Protocol Composition**: Building on the foundation of Protocol 1 with different message types

## Protocol 3: Simple Choice

### Description

A simple choice protocol where:
- Client chooses between two options:
  - Option 1: Client sends a u64 value and ends
  - Option 2: Client receives an f32 value and ends
- Server offers these two options:
  - Option 1: Server receives a u64 value and ends
  - Option 2: Server sends an f32 value and ends

### Type-Level Representation

```rust
// Client: Choose between sending a u64 or receiving an f32, then end
type ChoiceClient = Choose<Send<u64, End>, Recv<f32, End>>;
// Server: Offer to either receive a u64 or send an f32, then end
type ChoiceServer = Offer<Recv<u64, End>, Send<f32, End>>;
```

### Visual Diagram

```text
                  ChoiceClient                    ChoiceServer
                  ------------                    ------------
                        |                              |
                        |        Choose                |
                        | ---------------------------> |
                        |                              |
                        |        Offer                 |
                        | <--------------------------- |
                        |                              |
                 +------+------+              +--------+-------+
                 |             |              |                |
                 |             |              |                |
           Option 1       Option 2      Option 1         Option 2
                 |             |              |                |
                 |             |              |                |
                 v             v              v                v
              Send(u64)     Recv(f32)     Recv(u64)        Send(f32)
                 |             |              |                |
                 v             v              v                v
                End           End            End              End
```

### Key Concepts Demonstrated

- **Branching Protocols**: Using `Choose<L, R>` and `Offer<L, R>` to represent decision points
- **Client-side Choice**: The client decides which branch of the protocol to follow
- **Type-level Branching**: The type system ensures that both parties agree on the available options
- **Protocol Composition with Branching**: Building complex protocols by composing choices with other protocol types

## Protocol 4: Simple Authentication

### Description

A simple authentication protocol where:
- Client sends a username (String)
- Client sends a password (String)
- Server receives the username
- Server receives the password
- Server sends an authentication token (u128)
- Client receives the authentication token
- Both sides close the connection

### Type-Level Representation

```rust
// Client: Send username, send password, receive token, then end
type AuthClient = Send<String, Send<String, Recv<u128, End>>>;
// Server: Receive username, receive password, send token, then end
type AuthServer = Recv<String, Recv<String, Send<u128, End>>>;
```

### Visual Diagram

```text
                  AuthClient                      AuthServer
                  ----------                      ----------
                       |                               |
                       |                               |
                       |        Send(username)         |
                       | ----------------------------> |
                       |                               |
                       |        Recv(username)         |
                       | <---------------------------- |
                       |                               |
                       |        Send(password)         |
                       | ----------------------------> |
                       |                               |
                       |        Recv(password)         |
                       | <---------------------------- |
                       |                               |
                       |        Recv(token)            |
                       | <---------------------------- |
                       |                               |
                       |        Send(token)            |
                       | ----------------------------> |
                       |                               |
                       v                               v
                      End                             End
```

### Key Concepts Demonstrated

- **Multi-step Communication**: A sequence of multiple send/receive operations
- **Security Protocol Modeling**: Modeling a typical authentication flow
- **Sequential Composition**: Chaining multiple protocol steps together
- **Different Message Types**: Using different types (String for credentials, u128 for token)

## Protocol 5: Data Query with Options

### Description

A data query protocol with options where:
- Client sends a query string
- Server receives the query string
- Server chooses between two options:
  - Option 1: Server sends binary data (Vec<u8>) and ends
  - Option 2: Server sends an error code (i16) and ends
- Client offers these two options:
  - Option 1: Client receives binary data (Vec<u8>) and ends
  - Option 2: Client receives an error code (i16) and ends

### Type-Level Representation

```rust
// Client: Send query, offer to receive data or error, then end
type QueryClient = Send<String, Offer<Recv<Vec<u8>, End>, Recv<i16, End>>>;
// Server: Receive query, choose to send data or error, then end
type QueryServer = Recv<String, Choose<Send<Vec<u8>, End>, Send<i16, End>>>;
```

### Visual Diagram

```text
                  QueryClient                      QueryServer
                  -----------                      -----------
                       |                               |
                       |                               |
                       |        Send(query)            |
                       | ----------------------------> |
                       |                               |
                       |        Recv(query)            |
                       | <---------------------------- |
                       |                               |
                       |                               | Server chooses:
                       |                               |
                       |                               |--------+
                       |                               |        |
                       |                               |        |
                       |                               V        V
                       |                          Option 1   Option 2
                       |                               |        |
                       |        Recv(data)             |        |
                       | <---------------------------- |        |
                       |                               |        |
                       |                               |        |
                       |        Recv(error)            |        |
                       | <------------------------------------ |
                       |                               |        |
                       |                               |        |
                       V                               V        V
                      End                             End      End
```

### Key Concepts Demonstrated

- **Server-side Choice**: The server decides which branch of the protocol to follow
- **Error Handling**: Modeling success and error paths in a protocol
- **Complex Branching**: Combining sequential and branching communication patterns
- **Different Message Types**: Using different types for query (String), data (Vec<u8>), and error (i16)
- **API-like Interaction with Error Handling**: Modeling a typical query/response pattern with error handling