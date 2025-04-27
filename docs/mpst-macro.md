# MPST Macro Syntax Design

This document defines the syntax for a macro to create global protocols in the sessrums library. The macro syntax is inspired by sequence diagrams, making it intuitive and easy to read.

## Motivation

Creating global protocols using the current API requires verbose type definitions:

```rust
// Current approach using the builder pattern
let builder = GlobalProtocolBuilder::new();
let protocol = builder.send::<String, Client, Server, _>(
    builder.recv::<i32, Server, Client, _>(
        builder.end()
    )
);

// Or using type definitions directly
type Protocol = GSend<String, Client, Server, GRecv<i32, Server, Client, GEnd>>;
```

This approach becomes increasingly complex with branching, recursion, and composition. A macro would make it easier to define global protocols using a more intuitive syntax inspired by sequence diagrams.

## Macro Syntax

The proposed macro syntax, `mpst!`, uses a sequence diagram-like notation to define global protocols:

```rust
mpst! {
    protocol RequestResponse {
        Client -> Server: String;
        Server -> Client: i32;
    }
}
```

This would generate the equivalent of:

```rust
type RequestResponse = GSend<String, Client, Server, GRecv<i32, Server, Client, GEnd>>;
```

## Syntax Elements

### Protocol Declaration

A protocol is declared with a name and a block of interactions:

```rust
mpst! {
    protocol ProtocolName {
        // interactions
    }
}
```

### Message Passing

Message passing is represented using arrows between roles, followed by the message type:

```rust
Role1 -> Role2: Type;
```

This translates to `GSend<Type, Role1, Role2, ...>`.

### Branching and Choice

Branching is represented using labeled blocks with the `choice at Role` syntax:

```rust
choice at Client {
    option Success {
        Client -> Server: SuccessData;
        // continuation for Success
    }
    option Failure {
        Client -> Server: ErrorData;
        // continuation for Failure
    }
}
```

This translates to `GChoice<Client, (GSend<SuccessData, Client, Server, ...>, GSend<ErrorData, Client, Server, ...>)>`.

### Recursion

Recursion is represented using labeled blocks with the `rec Label` syntax and `continue Label` to reference the recursion point:

```rust
rec Loop {
    Client -> Server: Request;
    Server -> Client: Response;
    choice at Client {
        option Continue {
            continue Loop;
        }
        option Exit {
            Client -> Server: Done;
        }
    }
}
```

This translates to `GRec<LoopLabel, GSend<Request, Client, Server, GRecv<Response, Server, Client, GChoice<Client, (GVar<LoopLabel>, GSend<Done, Client, Server, GEnd>)>>>>`.

### Sequential Composition

Sequential composition is implicit in the syntax, as interactions are listed in sequence:

```rust
protocol Sequential {
    Client -> Server: String;
    Server -> Client: i32;
}
```

For explicit sequential composition of protocols:

```rust
protocol Combined {
    seq {
        include Protocol1;
        include Protocol2;
    }
}
```

### Parallel Composition

Parallel composition is represented using the `par` keyword:

```rust
protocol Parallel {
    par {
        Client -> Server: String;
    } and {
        Logger -> Monitor: LogEntry;
    }
}
```

This translates to `GPar<GSend<String, Client, Server, GEnd>, GSend<LogEntry, Logger, Monitor, GEnd>>`.

## Complete Examples

### Simple Message Passing

```rust
mpst! {
    protocol PingPong {
        Client -> Server: String;
        Server -> Client: String;
    }
}
```

Equivalent to:

```rust
type PingPong = GSend<String, Client, Server, GRecv<String, Server, Client, GEnd>>;
```

### Branching and Choice

```rust
mpst! {
    protocol Authentication {
        Client -> Server: Credentials;
        choice at Server {
            option Success {
                Server -> Client: Token;
                Client -> Server: Request;
                Server -> Client: Response;
            }
            option Failure {
                Server -> Client: ErrorMessage;
            }
        }
    }
}
```

Equivalent to:

```rust
type Authentication = GSend<Credentials, Client, Server, 
    GChoice<Server, (
        GSend<Token, Server, Client, 
            GRecv<Request, Client, Server, 
                GSend<Response, Server, Client, GEnd>
            >
        >,
        GSend<ErrorMessage, Server, Client, GEnd>
    )>
>;
```

### Recursion

```rust
mpst! {
    protocol ChatSession {
        rec ChatLoop {
            choice at Client {
                option SendMessage {
                    Client -> Server: Message;
                    Server -> Client: Confirmation;
                    continue ChatLoop;
                }
                option Quit {
                    Client -> Server: Disconnect;
                }
            }
        }
    }
}
```

Equivalent to:

```rust
type ChatSession = GRec<ChatLoopLabel, 
    GChoice<Client, (
        GSend<Message, Client, Server, 
            GRecv<Confirmation, Server, Client, 
                GVar<ChatLoopLabel>
            >
        >,
        GSend<Disconnect, Client, Server, GEnd>
    )>
>;
```

### Composition

```rust
mpst! {
    protocol Login {
        Client -> Server: Credentials;
        Server -> Client: Token;
    }

    protocol DataExchange {
        Client -> Server: Request;
        Server -> Client: Response;
    }

    protocol ComposedProtocol {
        seq {
            include Login;
            include DataExchange;
        }
    }
}
```

Equivalent to:

```rust
type Login = GSend<Credentials, Client, Server, GRecv<Token, Server, Client, GEnd>>;
type DataExchange = GSend<Request, Client, Server, GRecv<Response, Server, Client, GEnd>>;
type ComposedProtocol = GSeq<Login, DataExchange>;
```

### Parallel Composition

```rust
mpst! {
    protocol ParallelOperations {
        par {
            Client -> Server: Request;
            Server -> Client: Response;
        } and {
            Client -> Logger: LogEntry;
            Logger -> Monitor: Notification;
        }
    }
}
```

Equivalent to:

```rust
type ParallelOperations = GPar<
    GSend<Request, Client, Server, GRecv<Response, Server, Client, GEnd>>,
    GSend<LogEntry, Client, Logger, GSend<Notification, Logger, Monitor, GEnd>>
>;
```

## Translation to Global Protocol Types

The macro will parse the syntax and generate the corresponding global protocol types. The translation follows these rules:

1. **Message Passing**: `Role1 -> Role2: Type;` → `GSend<Type, Role1, Role2, ...>`
2. **Branching**: `choice at Role { ... }` → `GChoice<Role, (...)>` or `GOffer<Role, (...)>`
3. **Recursion**: `rec Label { ... }` → `GRec<LabelType, ...>` and `continue Label;` → `GVar<LabelType>`
4. **Sequential Composition**: `seq { ... }` → `GSeq<...>`
5. **Parallel Composition**: `par { ... } and { ... }` → `GPar<...>`
6. **End**: Implicit at the end of each branch → `GEnd`

## Edge Cases and Limitations

1. **Type Inference**: The macro may have limitations with type inference, requiring explicit type annotations in some cases.

2. **Nested Recursion**: Complex nested recursion patterns might require careful labeling to ensure correct references.

3. **Role Consistency**: The macro should validate that roles are used consistently throughout the protocol.

4. **Protocol Reuse**: When including other protocols, the macro needs to handle potential name conflicts and ensure proper composition.

5. **Validation**: The macro should perform validation similar to the `validate` method of `GlobalProtocol` to catch errors at compile time.

6. **Syntax Errors**: Clear error messages should be provided for syntax errors in the macro usage.

7. **Rust Limitations**: The macro implementation will be constrained by Rust's macro system limitations.

## Implementation Strategy

The implementation of the `mpst!` macro will involve:

1. Parsing the sequence diagram-like syntax
2. Building an abstract syntax tree (AST) representing the protocol
3. Validating the protocol structure
4. Generating Rust code that constructs the equivalent global protocol types

The macro will be implemented using Rust's procedural macro system, which provides the flexibility needed for this complex syntax transformation.

## Conclusion

The proposed `mpst!` macro syntax provides an intuitive, sequence diagram-inspired way to define global protocols in the sessrums library. It significantly reduces the verbosity of protocol definitions while maintaining the type safety and expressiveness of the underlying MPST system.

By making protocol definitions more readable and maintainable, this macro will improve the usability of the sessrums library for multiparty session types.