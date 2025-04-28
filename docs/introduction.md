# Introduction to Session Types

Session types provide a way to define and verify communication protocols at compile time. They ensure that interacting parties follow an agreed-upon sequence of operations, preventing common runtime errors like deadlocks, unexpected messages, or type mismatches. This library implements session types in Rust, leveraging the type system for protocol safety.

## Binary Session Types

Binary session types describe protocols between exactly two participants.

### Core Concepts

- **`Send<T, P>`**: Represents sending a value of type `T` and then continuing with protocol `P`.
- **`Recv<T, P>`**: Represents receiving a value of type `T` and then continuing with protocol `P`.
- **`Offer<L, R>`**: Offers a choice between two protocols, `L` and `R`. The peer decides which branch to take.
- **`Choose<L, R>`**: Chooses between two protocols, `L` and `R`. This party decides which branch to take.
- **`Rec<P>`**: Defines the start of a recursive protocol `P`. Allows defining protocols with loops or repeated interactions.
- **`Var<N>`**: Represents a jump back to the N-th enclosing `Rec` (0-indexed). Used inside recursive protocols.
- **`End`**: Represents the successful termination of a protocol session. No further communication is expected.

### Duality

Every protocol `P` has a dual `P::Dual`. If one party follows `P`, the other must follow `P::Dual` for communication to be safe.

- `Send<T, P>::Dual` is `Recv<T, P::Dual>`
- `Recv<T, P>::Dual` is `Send<T, P::Dual>`
- `Offer<L, R>::Dual` is `Choose<L::Dual, R::Dual>`
- `Choose<L, R>::Dual` is `Offer<L::Dual, R::Dual>`
- `Rec<P>::Dual` is `Rec<P::Dual>`
- `Var<N>::Dual` is `Var<N>`
- `End::Dual` is `End`

### Example: Simple Request-Response

```rust
use sessrums::prelude::*; // Assuming a prelude exists

// Protocol Definition
// Client sends a String query, receives a u64 response, then ends.
type ClientProto = Send<String, Recv<u64, End>>;

// Server receives a String query, sends a u64 response, then ends.
// Note: ServerProto is automatically the dual of ClientProto
type ServerProto = <ClientProto as Protocol>::Dual; // Recv<String, Send<u64, End>>

// Example Usage (Conceptual - requires IO setup)
async fn client_logic(chan: Chan<ClientProto, /* IO Type */>) -> Result<(), Error> {
    let chan = chan.send("QUERY".to_string()).await?;
    let (response, chan) = chan.recv().await?;
    println!("Client received: {}", response);
    chan.close()?;
    Ok(())
}

async fn server_logic(chan: Chan<ServerProto, /* IO Type */>) -> Result<(), Error> {
    let (query, chan) = chan.recv().await?;
    println!("Server received: {}", query);
    let chan = chan.send(12345u64).await?;
    chan.close()?;
    Ok(())
}
```

### Example: Branching (Offer/Choose)

```rust
use sessrums::prelude::*;
use sessrums::either::Either; // For handling offer result

// Protocol Definition
// Client sends a command (String). Server responds with either:
// Left Branch ('Ok'): Send String confirmation, End.
// Right Branch ('Err'): Send i32 error code, End.
type ClientProto = Send<String, Choose<Recv<String, End>, Recv<i32, End>>>;
type ServerProto = Recv<String, Offer<Send<String, End>, Send<i32, End>>>;

// Example Usage (Conceptual)
async fn client_logic_branch(chan: Chan<ClientProto, /* IO Type */>, cmd: String) -> Result<(), Error> {
    let chan = chan.send(cmd).await?;
    // Assume server logic decides which branch based on cmd
    // Client needs to know which branch to expect (or receive it first)
    // For simplicity, let's assume client *chooses* based on some logic
    if should_expect_ok() {
        let chan = chan.choose_left().await?; // Choose 'Ok' branch
        let (confirmation, chan) = chan.recv().await?;
        println!("Client received confirmation: {}", confirmation);
        chan.close()?;
    } else {
        let chan = chan.choose_right().await?; // Choose 'Err' branch
        let (error_code, chan) = chan.recv().await?;
        println!("Client received error code: {}", error_code);
        chan.close()?;
    }
    Ok(())
}

async fn server_logic_branch(chan: Chan<ServerProto, /* IO Type */>) -> Result<(), Error> {
    let (cmd, chan) = chan.recv().await?;
    if cmd == "VALID" {
        // Offer the 'Ok' branch
        match chan.offer().await? {
            Either::Left(chan) => { // Client chose 'Ok'
                let chan = chan.send("Command OK".to_string()).await?;
                chan.close()?;
            },
            Either::Right(_) => { /* Protocol error if client chose wrong branch */ panic!("Protocol mismatch"); }
        }
    } else {
        // Offer the 'Err' branch
        match chan.offer().await? {
            Either::Left(_) => { /* Protocol error */ panic!("Protocol mismatch"); }
            Either::Right(chan) => { // Client chose 'Err'
                let chan = chan.send(-1i32).await?;
                chan.close()?;
            }
        }
    }
    Ok(())
}

fn should_expect_ok() -> bool { /* Client-side logic */ true }
```

### Example: Recursion

```rust
use sessrums::prelude::*;

// Protocol Definition
// Client repeatedly sends a u8, Server receives it and sends back a bool.
// Client can choose to stop (End) or continue (Var<0>).
type ClientRecursiveProto = Rec<Send<u8, Recv<bool, Choose<End, Var<0>>>>>;
type ServerRecursiveProto = <ClientRecursiveProto as Protocol>::Dual; // Rec<Recv<u8, Send<bool, Offer<End, Var<0>>>>>

// Example Usage (Conceptual)
async fn client_recursive(mut chan: Chan<ClientRecursiveProto, /* IO Type */>) -> Result<(), Error> {
    for i in 0..3u8 { // Loop 3 times
        let chan_loop = chan.enter(); // Enter the Rec block
        let chan_loop = chan_loop.send(i).await?;
        let (go_again, chan_loop) = chan_loop.recv().await?;

        if go_again && i < 2 {
            chan = chan_loop.choose_right().await?; // Choose Var<0> (continue)
            chan = chan.zero(); // Jump back to Rec
        } else {
            chan = chan_loop.choose_left().await?; // Choose End
            chan.close()?;
            break;
        }
    }
    Ok(())
}

async fn server_recursive(mut chan: Chan<ServerRecursiveProto, /* IO Type */>) -> Result<(), Error> {
    loop {
        let chan_loop = chan.enter();
        let (val, chan_loop) = chan_loop.recv().await?;
        let should_continue = val < 2; // Server logic to continue
        let chan_loop = chan_loop.send(should_continue).await?;

        match chan_loop.offer().await? {
            Either::Left(chan_end) => { // Client chose End
                chan_end.close()?;
                break;
            },
            Either::Right(chan_cont) => { // Client chose Var<0>
                chan = chan_cont.zero(); // Jump back
            }
        }
    }
    Ok(())
}
```

## Multiparty Session Types (MPST)

Multiparty Session Types (MPST) extend binary session types to protocols involving three or more participants.

### Core Concepts

- **Roles**: Unique identifiers for each participant (e.g., `Client`, `Server`, `Worker`).
- **Global Protocol**: A single definition describing the entire interaction choreography between all roles. Specifies who sends what to whom.
- **Projection**: The process of deriving a *local* protocol (a binary session type) for a specific role from the global protocol. Each participant only needs to implement its projected local protocol.

### Implementation Status

**Note:** Multiparty session type support in this library is currently **experimental and incomplete**.

- The `global_protocol!` macro allows defining global protocols.
- However, the underlying **projection logic** (generating correct local types from global types) and **protocol validation** are not fully implemented or verified.
- Using MPST features may lead to unexpected behavior or compile-time/runtime errors beyond standard session type guarantees. Use with caution and primarily for exploration.

### `global_protocol!` Macro

This macro provides a syntax for defining global protocols.

```rust
use sessrums::prelude::*; // Assuming roles like RoleA, RoleB, RoleC are defined
use sessrums_macro::global_protocol; // Import the macro

global_protocol! {
    protocol ThreePartyProto {
        A -> B: String; // Role A sends String to Role B
        B -> C: i32;   // Role B sends i32 to Role C
        C -> A: bool;  // Role C sends bool to Role A
        // Protocol implicitly ends here
    }
}

// This macro attempts to generate corresponding global type structures.
// Example (Conceptual - actual generated types might differ):
// type ThreePartyProto = GSend<String, RoleA, RoleB,
//                         GSend<i32, RoleB, RoleC,
//                           GSend<bool, RoleC, RoleA, GEnd>>>;

// Using the projected types (Requires functional projection)
// let local_a: <ThreePartyProto as Project<RoleA>>::Local = project();
// let local_b: <ThreePartyProto as Project<RoleB>>::Local = project();
// let local_c: <ThreePartyProto as Project<RoleC>>::Local = project();
```

**Reminder:** Due to the incomplete implementation status, the projection (`Project<Role>`) and subsequent use of these local types are not guaranteed to work correctly or provide full safety.

## Key API Concepts

- **`Chan<P, IO>`**: Represents a communication channel endpoint following protocol `P` over an IO backend `IO`. The `P` type parameter changes as communication progresses.
- **Roles**: Types implementing the `Role` trait identify participants in MPST protocols.
- **Error Handling**: Communication errors (IO, protocol violations) are typically represented by an `Error` enum. Refer to [Error Handling Guide](error-handling.md) for details.

## Further Reading

*   [A Very Gentle Introduction to Multiparty Session Types](http://mrg.doc.ic.ac.uk/publications/a-very-gentle-introduction-to-multiparty-session-types/main.pdf)
*   [Comprehensive Multiparty Session Types](https://arxiv.org/pdf/1902.00544)
*   [Less is more: multiparty session types revisited](https://dl.acm.org/doi/10.1145/3290343)
*   [Implementing Multiparty Session Types in Rust](https://inria.hal.science/hal-03273998v1/document)
*   [A Linear Decomposition of Multiparty Sessions for Safe Distributed Programming](https://drops.dagstuhl.de/storage/00lipics/lipics-vol074-ecoop2017/LIPIcs.ECOOP.2017.24/LIPIcs.ECOOP.2017.24.pdf)
*   [Complete Multiparty Session Type Projection with Automata](https://link.springer.com/chapter/10.1007/978-3-031-37709-9_17)
*   [Composition and Decomposition of Multiparty Sessions](https://inria.hal.science/hal-03338671v1)
*   [Global Types for Asynchronous Multiparty Sessions](https://t-ladies.di.unimi.it/kickoff-slides/giannini,%20paola.pdf)
*   [API Generation for Multiparty Session Types, Revisited and Revised Using Scala 3](https://drops.dagstuhl.de/storage/00lipics/lipics-vol222-ecoop2022/LIPIcs.ECOOP.2022.27/LIPIcs.ECOOP.2022.27.pdf)