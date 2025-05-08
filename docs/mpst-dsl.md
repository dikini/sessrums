# MPST DSL: Multiparty Session Type Domain-Specific Language

This document provides comprehensive documentation for the Multiparty Session Type (MPST) Domain-Specific Language (DSL) implemented in the sessrums project. The DSL provides a concise, readable syntax for defining multiparty session type protocols, making them more intuitive to understand while maintaining the strong type guarantees of the underlying system.

## Table of Contents

1. [Introduction](#introduction)
2. [DSL Syntax and Grammar](#dsl-syntax-and-grammar)
3. [Defining Protocols](#defining-protocols)
4. [Common Protocol Patterns](#common-protocol-patterns)
5. [Integration with the MPST System](#integration-with-the-mpst-system)
6. [Projection: From Global to Local Protocols](#projection-from-global-to-local-protocols)
7. [Error Handling and Troubleshooting](#error-handling-and-troubleshooting)
8. [Examples](#examples)

## Introduction

The MPST DSL is a procedural macro system that allows you to define multiparty session type protocols using a Mermaid-like syntax. This approach offers several advantages over manually defining protocols using the underlying Rust types:

- **Readability**: The DSL syntax resembles sequence diagrams, making protocols easier to understand at a glance
- **Conciseness**: Complex protocols can be defined with minimal boilerplate
- **Compile-time Verification**: Protocol errors are caught during compilation rather than at runtime
- **Type Safety**: Direct integration with Rust's type system ensures type-safe protocol definitions
- **IDE Support**: Better integration with Rust's tooling ecosystem (syntax highlighting, error reporting)

The DSL is processed at compile time by the `mpst!` procedural macro, which transforms the textual protocol definitions into Rust code that constructs the `GlobalInteraction` enum structure. This enables you to define complex protocols using a familiar, diagram-like syntax while maintaining the strong type guarantees of the underlying system.

## DSL Syntax and Grammar

### Lexical Elements

#### Identifiers

Identifiers are used for role names, message types, and recursion labels.

```ebnf
Identifier ::= [a-zA-Z_][a-zA-Z0-9_]*
```

Identifiers must start with a letter or underscore, followed by any number of letters, digits, or underscores.

#### Keywords

The following keywords are reserved and cannot be used as identifiers:

```
protocol, participant, as, choice, at, option, or, rec, continue, end
```

#### Symbols

The following symbols have special meaning in the DSL:

```
{ } ; : -> ,
```

#### Whitespace and Comments

Whitespace (spaces, tabs, newlines) is ignored except as a separator between tokens.

Comments can be:
- Line comments: `// comment text`
- Block comments: `/* comment text */`

### Grammar

The grammar is defined in Extended Backus-Naur Form (EBNF).

#### Protocol Definition

```ebnf
Protocol ::= 'protocol' Identifier '{' ParticipantList InteractionList '}'

ParticipantList ::= Participant*

Participant ::= 'participant' Identifier ('as' Identifier)? ';'

InteractionList ::= Interaction*

Interaction ::= MessageInteraction
              | ChoiceInteraction
              | RecursionInteraction
              | ContinueInteraction
              | EndInteraction
```

#### Message Interaction

```ebnf
MessageInteraction ::= Identifier '->' Identifier ':' MessageType ';'

MessageType ::= Identifier ('::' Identifier)* ('<' GenericParams '>')?

GenericParams ::= MessageType (',' MessageType)*
```

#### Choice Interaction

```ebnf
ChoiceInteraction ::= 'choice' 'at' Identifier '{' BranchList '}'

BranchList ::= Branch ('or' Branch)*

Branch ::= ('option' Identifier)? '{' InteractionList '}'
```

#### Recursion Interaction

```ebnf
RecursionInteraction ::= 'rec' Identifier '{' InteractionList '}'

ContinueInteraction ::= 'continue' Identifier ';'
```

#### End Interaction

```ebnf
EndInteraction ::= 'end' ';'
```

## Defining Protocols

### Basic Protocol Structure

A protocol definition consists of a protocol name, a list of participants, and a sequence of interactions:

```rust
mpst! {
    protocol ProtocolName {
        // Participant declarations
        participant Role1;
        participant Role2;
        participant Role3;
        
        // Interactions
        Role1 -> Role2: MessageType;
        Role2 -> Role3: AnotherMessageType;
        // ...
    }
}
```

### Participant Declaration

Participants are declared at the beginning of the protocol using the `participant` keyword:

```rust
participant Role;
```

You can also provide an alias for a role:

```rust
participant Role as Alias;
```

This is useful when you want to use a different name for a role in the protocol definition than the actual role type in your code.

### Message Passing

Message passing is represented using arrows between roles, followed by the message type:

```rust
Role1 -> Role2: MessageType;
```

This indicates that `Role1` sends a message of type `MessageType` to `Role2`.

### Choice and Branching

Branching is represented using the `choice at Role` syntax, where `Role` is the participant that makes the choice:

```rust
choice at Role1 {
    option Branch1 {
        // Interactions for Branch1
    }
    option Branch2 {
        // Interactions for Branch2
    }
}
```

The `option` keyword is optional, and you can use `or` to separate branches:

```rust
choice at Role1 {
    option Branch1 {
        // Interactions for Branch1
    }
    or {
        // Interactions for Branch2
    }
}
```

### Recursion

Recursion is represented using labeled blocks with the `rec Label` syntax and `continue Label` to reference the recursion point:

```rust
rec Loop {
    // Interactions
    continue Loop;
}
```

### End

The `end` keyword indicates the end of a protocol path:

```rust
end;
```

## Common Protocol Patterns

### Request-Response

A simple request-response pattern involves a client sending a request to a server, and the server responding:

```rust
mpst! {
    protocol RequestResponse {
        participant Client;
        participant Server;
        
        Client -> Server: Request;
        Server -> Client: Response;
        end;
    }
}
```

### Choice-Based Protocol

A choice-based protocol involves a participant making a choice between different branches:

```rust
mpst! {
    protocol LoginProtocol {
        participant Client;
        participant Server;
        
        Client -> Server: Credentials;
        
        choice at Server {
            option Success {
                Server -> Client: LoginSuccess;
                end;
            }
            or {
                Server -> Client: LoginFailure;
                end;
            }
        }
    }
}
```

### Recursive Protocol

A recursive protocol involves repeating a sequence of interactions:

```rust
mpst! {
    protocol ChatProtocol {
        participant Client;
        participant Server;
        
        rec ChatLoop {
            choice at Client {
                option SendMessage {
                    Client -> Server: Message;
                    Server -> Client: Acknowledgment;
                    continue ChatLoop;
                }
                or {
                    Client -> Server: Disconnect;
                    end;
                }
            }
        }
    }
}
```

## Integration with the MPST System

The DSL integrates seamlessly with the rest of the MPST system in sessrums. The `mpst!` macro generates Rust code that constructs the `GlobalInteraction` enum structure, which is the core representation of global protocols in the sessrums library.

### From DSL to Global Protocol Types

When you define a protocol using the DSL, the `mpst!` macro parses the syntax and generates the corresponding global protocol types. The translation follows these rules:

| DSL Construct | Rust Type |
|---------------|-----------|
| `A -> B: T;` | `GlobalInteraction::Message { from: "A", to: "B", msg: PhantomData<T>, cont: ... }` |
| `choice at A { ... }` | `GlobalInteraction::Choice { decider: "A", branches: [...] }` |
| `rec X { ... }` | `GlobalInteraction::Rec { label: "X", body: ... }` |
| `continue X;` | `GlobalInteraction::Var { label: "X" }` |
| `end;` | `GlobalInteraction::End` |

For example, the following DSL protocol:

```rust
mpst! {
    protocol PingPong {
        participant Client;
        participant Server;
        
        Client -> Server: String;
        Server -> Client: String;
        end;
    }
}
```

Is translated to:

```rust
type PingPong = GlobalInteraction<String>;

impl PingPong {
    pub fn new() -> Self {
        GlobalInteraction::message(
            "Client",
            "Server",
            GlobalInteraction::message(
                "Server",
                "Client",
                GlobalInteraction::end()
            )
        )
    }
}
```

### Using Global Protocols

Once you've defined a global protocol using the DSL, you can use it like any other global protocol in the sessrums library:

```rust
// Define the protocol using the DSL
mpst! {
    protocol PingPong {
        participant Client;
        participant Server;
        
        Client -> Server: String;
        Server -> Client: String;
        end;
    }
}

// Create an instance of the global protocol
let protocol = PingPong::new();

// Use the protocol with the rest of the MPST system
// ...
```

## Projection: From Global to Local Protocols

One of the key features of the MPST system is the ability to project a global protocol to local protocols for each participant. This is done using the `project!` macro, which takes a global protocol type and a role type, and returns the projected local protocol type for that role.

### The `project!` Macro

The `project!` macro has the following syntax:

```rust
project!(GlobalProtocol, Role, [MessageType])
```

Where:
- `GlobalProtocol` is the global protocol type
- `Role` is the role type to project for
- `MessageType` is an optional message type parameter (if not provided, it will be inferred)

For example:

```rust
// Define the protocol using the DSL
mpst! {
    protocol PingPong {
        participant Client;
        participant Server;
        
        Client -> Server: String;
        Server -> Client: String;
        end;
    }
}

// Project the global protocol to local protocols for each role
type ClientProtocol = project!(PingPong, Client, String);
type ServerProtocol = project!(PingPong, Server, String);
```

### How Projection Works

Projection is the process of extracting the local behavior of a specific participant from a global protocol. The projection algorithm follows these rules:

1. **Message Passing**:
   - If the participant is the sender, the projection is a `Send` operation
   - If the participant is the receiver, the projection is a `Receive` operation
   - If the participant is neither the sender nor the receiver, the projection is the projection of the continuation

2. **Choice**:
   - If the participant is the decider, the projection is a `Choose` operation
   - If the participant is not the decider but participates in the branches, the projection is an `Offer` operation
   - If the participant does not participate in any branch, the projection is `End`

3. **Recursion**:
   - The projection of a recursion is a recursion with the same label and the projection of the body
   - The projection of a continue is a continue with the same label

4. **End**:
   - The projection of `End` is `End`

For example, the projection of the `PingPong` protocol for the `Client` role is:

```rust
type ClientProtocol = Send<String, Receive<String, End>>;
```

And for the `Server` role:

```rust
type ServerProtocol = Receive<String, Send<String, End>>;
```

### Using Projected Protocols

Once you've projected a global protocol to local protocols for each role, you can use these local protocols to implement the behavior of each participant:

```rust
// Define the protocol using the DSL
mpst! {
    protocol PingPong {
        participant Client;
        participant Server;
        
        Client -> Server: String;
        Server -> Client: String;
        end;
    }
}

// Project the global protocol to local protocols for each role
type ClientProtocol = project!(PingPong, Client, String);
type ServerProtocol = project!(PingPong, Server, String);

// Implement the client behavior
async fn run_client(session: Session<ClientProtocol>) -> Result<(), Error> {
    // Send a message to the server
    let session = session.send("Hello, server!".to_string()).await?;
    
    // Receive a message from the server
    let (message, session) = session.receive().await?;
    println!("Client received: {}", message);
    
    // End the session
    session.close().await?;
    
    Ok(())
}

// Implement the server behavior
async fn run_server(session: Session<ServerProtocol>) -> Result<(), Error> {
    // Receive a message from the client
    let (message, session) = session.receive().await?;
    println!("Server received: {}", message);
    
    // Send a message to the client
    let session = session.send("Hello, client!".to_string()).await?;
    
    // End the session
    session.close().await?;
    
    Ok(())
}
```

### Projection with Complex Protocols

Projection becomes more interesting with complex protocols involving choice and recursion. Let's consider a more complex example:

```rust
mpst! {
    protocol FileTransfer {
        participant Client;
        participant Server;
        
        rec Loop {
            Client -> Server: FileName;
            
            choice at Server {
                option FileExists {
                    Server -> Client: FileSize;
                    Client -> Server: Ready;
                    
                    rec Transfer {
                        Server -> Client: FileChunk;
                        
                        choice at Client {
                            option Continue {
                                Client -> Server: Ack;
                                continue Transfer;
                            }
                            or {
                                Client -> Server: Done;
                                
                                choice at Client {
                                    option RequestMore {
                                        continue Loop;
                                    }
                                    or {
                                        Client -> Server: Quit;
                                        end;
                                    }
                                }
                            }
                        }
                    }
                }
                or {
                    Server -> Client: FileNotFound;
                    continue Loop;
                }
            }
        }
    }
}
```

The projection of this protocol for the `Client` role would involve:
- Sending a `FileName` to the server
- Offering a choice from the server (either `FileExists` or `FileNotFound`)
- If `FileExists`, receiving a `FileSize`, sending `Ready`, and entering a loop to receive file chunks
- In the file chunk loop, choosing whether to continue receiving chunks or finish
- If finished, choosing whether to request more files or quit

The projection for the `Server` role would involve:
- Receiving a `FileName` from the client
- Choosing whether the file exists or not
- If the file exists, sending a `FileSize`, receiving `Ready`, and entering a loop to send file chunks
- In the file chunk loop, offering a choice from the client (either continue or done)
- If done, offering another choice from the client (either request more files or quit)

This example demonstrates how projection handles complex nested structures like recursion within choice branches.

## Error Handling and Troubleshooting

The DSL parser provides detailed error messages for syntax and semantic errors, including:

1. **Syntax Errors**: Missing semicolons, unmatched braces, invalid arrows, etc.
2. **Participant Errors**: Undefined participants, duplicate participants, invalid participant names
3. **Recursion Errors**: Undefined recursion labels, duplicate recursion labels, continue outside recursion
4. **Type Errors**: Invalid message types, unsupported generic types
5. **Semantic Errors**: Invalid choice role, empty choice, unreachable code

### Common Error Messages

#### Syntax Errors

```
error: Invalid choice syntax. Expected 'choice at Role { option Label { ... } ... }'.
  --> protocol.rs:15:5
   |
15 |     choice Client {
   |     ^^^^^^^^^^^^^ Expected 'at' keyword after 'choice'
   |
   = help: Use 'choice at Client { ... }' instead
```

#### Participant Errors

```
error: Undefined participant 'Database'. All participants must be declared at the beginning of the protocol.
  --> protocol.rs:12:5
   |
12 |     Client -> Database: Query;
   |              ^^^^^^^^ Participant not declared
   |
   = help: Add 'participant Database;' to the beginning of the protocol
```

#### Recursion Errors

```
error: Undefined recursion label 'Loop'. Labels must be defined with 'rec Loop' before they can be referenced with 'continue Loop'.
  --> protocol.rs:18:9
   |
18 |         continue Loop;
   |                 ^^^^ Label not defined
   |
   = help: Define the recursion label with 'rec Loop { ... }' before using 'continue Loop'
```

#### Type Errors

```
error: Invalid message type 'Map<String, Value>'. Message types must be valid Rust types.
  --> protocol.rs:10:24
   |
10 |     Client -> Server: Map<String, Value>;
   |                        ^^^^^^^^^^^^^^^^^ Unknown type
   |
   = help: Use a fully qualified path or import the type with 'use'
```

#### Semantic Errors

```
error: Role 'Client' cannot make a choice in a branch where 'Server' is the deciding role.
  --> protocol.rs:25:13
   |
25 |             Client -> Server: Continue;
   |             ^^^^^^ Invalid role in choice branch
   |
   = help: In a choice block, the first message in each branch must be sent by the deciding role
```

### Troubleshooting Tips

1. **Check Participant Declarations**: Ensure all participants are declared at the beginning of the protocol.
2. **Check Message Types**: Ensure all message types are valid Rust types.
3. **Check Recursion Labels**: Ensure all recursion labels are defined before they are referenced.
4. **Check Choice Syntax**: Ensure the choice syntax is correct, with the deciding role specified after the `at` keyword.
5. **Check Branch Consistency**: Ensure all branches in a choice are consistent with the deciding role.

## Examples

### Simple Ping-Pong Protocol

```rust
use sessrums_macro::mpst;
use sessrums_types::roles::{Client, Server};

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
        Client -> Server: String;
        Server -> Client: String;
        end;
    }
}

fn main() {
    // Create an instance of the global protocol
    let protocol = PingPong::new();
    
    // Project the global protocol to local protocols for each role
    type ClientProtocol = project!(PingPong, Client, String);
    type ServerProtocol = project!(PingPong, Server, String);
    
    // Now you can use these local protocols to implement the behavior of each participant
    // ...
}
```

### Protocol with Choice (File Transfer)

```rust
use sessrums_macro::mpst;
use sessrums_types::roles::{Client, Server};

// Define custom types for the protocol
struct FileRequest {
    filename: String,
}

struct FileContent {
    content: Vec<u8>,
}

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

fn main() {
    // Create an instance of the global protocol
    let protocol = FileTransfer::new();
    
    // Project the global protocol to local protocols for each role
    type ClientProtocol = project!(FileTransfer, Client);
    type ServerProtocol = project!(FileTransfer, Server);
    
    // Now you can use these local protocols to implement the behavior of each participant
    // ...
}
```

### Recursive Protocol (Streaming Data)

```rust
use sessrums_macro::mpst;
use sessrums_types::roles::{Producer, Consumer};

// Define custom types for the protocol
struct DataChunk {
    data: Vec<u8>,
    sequence_number: u32,
}

struct Ack {
    sequence_number: u32,
}

struct EndOfStream;

// Define a protocol for streaming data with termination
mpst! {
    protocol DataStream {
        // Define the participants
        participant Producer;
        participant Consumer;

        // Define the recursive interaction
        rec Stream {
            // Producer decides whether to send more data or end the stream
            choice at Producer {
                // Send more data
                option SendData {
                    Producer -> Consumer: DataChunk;
                    Consumer -> Producer: Ack;
                    continue Stream;
                }
                
                // End the stream
                option EndStream {
                    Producer -> Consumer: EndOfStream;
                    end;
                }
            }
        }
    }
}

fn main() {
    // Create an instance of the global protocol
    let protocol = DataStream::new();
    
    // Project the global protocol to local protocols for each role
    type ProducerProtocol = project!(DataStream, Producer);
    type ConsumerProtocol = project!(DataStream, Consumer);
    
    // Now you can use these local protocols to implement the behavior of each participant
    // ...
}
```

These examples demonstrate the key features of the MPST DSL, including message passing, choice, and recursion. They also show how to integrate the DSL with the rest of the MPST system through projection.