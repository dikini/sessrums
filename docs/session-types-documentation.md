# sessrums: Session Types EZ

A Rust library for asynchronous session types with minimal dependencies, focusing on expressing the process calculus in the types using Rust's type system features, including `const generics`.

## Table of Contents

1. [Introduction](#introduction)
2. [Core Concepts](#core-concepts)
   - [What are Session Types?](#what-are-session-types)
   - [Duality](#duality)
   - [Type Safety](#type-safety)
   - [Protocol Composition](#protocol-composition)
3. [Visual Diagrams](#visual-diagrams)
4. [Quick Reference](#quick-reference)
5. [Protocol Types](#protocol-types)
   - [Send](#send)
   - [Recv](#recv)
   - [End](#end)
   - [Offer & Choose](#offer--choose)
6. [Multiparty Session Types](#multiparty-session-types)
   - [Roles](#roles)
   - [Global Protocol](#global-protocol)
   - [Global Protocol Macro](#global-protocol-macro)
   - [Projection](#projection)
7. [Channel Implementation](#channel-implementation)
   - [Chan Type](#chan-type)
   - [IO Abstraction](#io-abstraction)
8. [Error Handling](#error-handling)
   - [Error Type](#error-type)
   - [Error Variants](#error-variants)
9. [API Reference](#api-reference)
   - [send Method](#send-method)
   - [recv Method](#recv-method)
   - [close Method](#close-method)
10. [Usage Examples](#usage-examples)
    - [Simple Client-Server Protocol](#simple-client-server-protocol)
    - [Error Handling](#error-handling-example)
    - [Type Safety Examples](#type-safety-examples)
11. [Visual Protocol Representation](#visual-protocol-representation)
12. [Advanced Topics](#advanced-topics)
     - [Custom IO Implementations](#custom-io-implementations)
     - [Protocol Testing](#protocol-testing)

## Introduction

sessrums (Session Types EZ) is a Rust library that implements session types, a type discipline for communication protocols that allows compile-time verification of protocol adherence. This library focuses on expressing the process calculus in the types using Rust's type system features, with minimal dependencies.

Session types provide a way to specify and verify communication protocols at compile time, ensuring that communicating parties follow the agreed-upon protocol without runtime errors or deadlocks.

## Core Concepts

### Roles

In an MPST protocol, each participant is represented by a unique role. Roles are used to define the structure of the session types from each participant's perspective and ensure that communication is correctly directed between the intended parties.

The `Role` trait defines the basic requirements for a type to be used as a role in sessrums. Concrete role types (e.g., `RoleA`, `RoleB`) implement this trait.

```rust
pub trait Role: Send + 'static {
    /// Returns a string representation of the role.
    fn name(&self) -> &'static str;
}
```

### What are Session Types?

Session types are a formal method for describing communication protocols at the type level. They allow you to specify the sequence and types of messages exchanged between communicating parties, ensuring that:

1. Messages are sent and received in the correct order
2. Messages have the expected types
3. The protocol is followed to completion
4. Communication is free from deadlocks and race conditions

In sessrums, session types are represented as Rust types that describe the communication behavior of a channel. These types are composed of primitive protocol types like `Send<T, P>`, `Recv<T, P>`, and `End`.

### Duality

Duality is a fundamental concept in session types. For every protocol `P`, there exists a dual protocol `P::Dual` that represents the complementary behavior. For example:

- If `P` sends a message of type `T`, then `P::Dual` receives a message of type `T`.
- If `P` offers a choice between protocols, then `P::Dual` makes a choice between the dual protocols.

This duality ensures that when two parties follow dual protocols, their communication is guaranteed to be compatible and free from communication errors like deadlocks or protocol violations.

```rust
// A protocol that sends an i32, then receives a String, then ends
type ClientProtocol = Send<i32, Recv<String, End>>;

// The dual protocol receives an i32, then sends a String, then ends
type ServerProtocol = <ClientProtocol as Protocol>::Dual;
// Equivalent to: Recv<i32, Send<String, End>>
```

### Type Safety

Session types leverage Rust's type system to ensure protocol adherence at compile time. This means that protocol violations are caught as type errors during compilation, preventing runtime communication errors.

For example, if a protocol specifies that a channel should first send an `i32` and then receive a `String`, attempting to receive before sending or sending a value of the wrong type will result in a compile-time error.

### Protocol Composition

Session types can be composed to build complex communication patterns. The primitive protocol types can be nested to create protocols of arbitrary complexity:

```rust
// A protocol that sends an i32, receives a bool, then sends a String, then ends
type ComplexProtocol = Send<i32, Recv<bool, Send<String, End>>>;
```

In multiparty session types, we support several forms of protocol composition:

#### Sequential Composition

Sequential composition (`GSeq<First, Second>`) allows two protocols to be executed one after the other. The `First` protocol is executed to completion before the `Second` protocol begins.

```rust
// A protocol where RoleA sends an i32 to RoleB, then RoleB sends a String to RoleA
type SequentialProtocol = GSeq<
    GSend<i32, RoleA, RoleB, GEnd>,
    GSend<String, RoleB, RoleA, GEnd>
>;
```

Using the builder pattern:

```rust
let builder = GlobalProtocolBuilder::new();
let protocol = builder.seq(
    builder.send::<i32, RoleA, RoleB, GEnd>(),
    builder.send::<String, RoleB, RoleA, GEnd>()
);
```

#### Parallel Composition

Parallel composition (`GPar<First, Second>`) allows two protocols to be executed concurrently. The `First` and `Second` protocols can proceed independently of each other.

```rust
// A protocol where RoleA sends an i32 to RoleB in parallel with RoleC sending a String to RoleD
type ParallelProtocol = GPar<
    GSend<i32, RoleA, RoleB, GEnd>,
    GSend<String, RoleC, RoleD, GEnd>
>;
```

Using the builder pattern:

```rust
let builder = GlobalProtocolBuilder::new();
let protocol = builder.par(
    builder.send::<i32, RoleA, RoleB, GEnd>(),
    builder.send::<String, RoleC, RoleD, GEnd>()
);
```

#### Complex Compositions

These composition operators can be combined to create complex protocols:

```rust
// A protocol where:
// 1. RoleA sends an i32 to RoleB
// 2. Then, in parallel:
//    a. RoleB sends a String to RoleA
//    b. RoleC sends a bool to RoleD
type ComplexProtocol = GSeq<
    GSend<i32, RoleA, RoleB, GEnd>,
    GPar<
        GSend<String, RoleB, RoleA, GEnd>,
        GSend<bool, RoleC, RoleD, GEnd>
    >
>;
```

Using the builder pattern:

```rust
let builder = GlobalProtocolBuilder::new();
let protocol = builder.seq(
    builder.send::<i32, RoleA, RoleB, GEnd>(),
    builder.par(
        builder.send::<String, RoleB, RoleA, GEnd>(),
        builder.send::<bool, RoleC, RoleD, GEnd>()
    )
);
```

## Visual Diagrams

Visual representations of session types concepts are available in the [Session Types Diagrams](session-types-diagrams.md) document. These diagrams illustrate:

- Protocol communication flow
- Protocol type composition
- Duality relationships
- Channel state transitions
- Client-server interaction patterns
- Complex protocols with choices

The diagrams provide a visual way to understand how session types work and how they ensure type-safe communication.

## Quick Reference

A concise summary of the key concepts and API methods is available in the [Quick Reference Guide](quick-reference.md). This guide provides a quick overview of:

- Protocol types and their duals
- Channel API methods
- Error handling
- Common protocol patterns
- IO implementation
- Type safety examples

This is particularly useful for developers who are already familiar with the library and just need a quick reminder of how to use it.

## Protocol Types

### Send

The `Send<T, P>` type represents a protocol that sends a value of type `T` and then continues with protocol `P`.

```rust
pub struct Send<T, P> {
    _marker: PhantomData<(T, P)>,
}

impl<T, P: Protocol> Protocol for Send<T, P> {
    type Dual = Recv<T, P::Dual>;
}
```

The dual of `Send<T, P>` is `Recv<T, P::Dual>`, which represents receiving a value of type `T` and then continuing with the dual of protocol `P`.

### Recv

The `Recv<T, P>` type represents a protocol that receives a value of type `T` and then continues with protocol `P`.

```rust
pub struct Recv<T, P> {
    _marker: PhantomData<(T, P)>,
}

impl<T, P: Protocol> Protocol for Recv<T, P> {
    type Dual = Send<T, P::Dual>;
}
```

The dual of `Recv<T, P>` is `Send<T, P::Dual>`, which represents sending a value of type `T` and then continuing with the dual of protocol `P`.

### End

The `End` type represents the end of a communication session. It is a terminal protocol that indicates no further communication will occur.

```rust
pub struct End;

impl Protocol for End {
    type Dual = End;
}
```

The dual of `End` is `End` itself, as ending a session is symmetric for both parties involved in the communication.

### Offer & Choose

The `Offer<L, R>` and `Choose<L, R>` types represent protocols for making and offering choices between two possible continuations.

- `Offer<L, R>` represents a protocol that offers a choice between continuing with protocol `L` or protocol `R`.
- `Choose<L, R>` represents a protocol that makes a choice between continuing with protocol `L` or protocol `R`.

The dual of `Offer<L, R>` is `Choose<L::Dual, R::Dual>`, and the dual of `Choose<L, R>` is `Offer<L::Dual, R::Dual>`.

For detailed information about the Offer and Choose protocol types, including API methods and usage examples, see the [Offer and Choose Guide](offer-choose.md).

## Multiparty Session Types

Multiparty Session Types (MPST) extend the binary session type system to support communication protocols involving multiple participants. This section describes the core components of MPST in sessrums.

### Roles

In an MPST protocol, each participant is represented by a unique role. Roles are used to define the structure of the session types from each participant's perspective and ensure that communication is correctly directed between the intended parties.

The `Role` trait defines the basic requirements for a type to be used as a role in sessrums. Concrete role types (e.g., `RoleA`, `RoleB`) implement this trait.

```rust
pub trait Role: Send + 'static {
    /// Returns a string representation of the role.
    fn name() -> &'static str;
}
```

### Global Protocol

A global protocol describes the communication behavior between multiple roles in a distributed system. It specifies the sequence and types of messages exchanged between participants, as well as control flow structures like choices and recursion.

The `GlobalProtocol` trait defines the interface for all global protocol types:

```rust
pub trait GlobalProtocol {
    /// Returns a string representation of the protocol for debugging.
    fn protocol_name(&self) -> &'static str;
    
    /// Validates the structure of the global protocol.
    ///
    /// This method checks for structural errors like choices with no branches,
    /// mismatched recursion labels, etc.
    fn validate(&self) -> Result<()>;
    
    /// Returns the roles involved in this protocol.
    ///
    /// This method returns a vector of role names that participate in the protocol.
    fn involved_roles(&self) -> Vec<&'static str>;
}
```

#### Global Protocol Types

sessrums provides several types that implement the `GlobalProtocol` trait, representing different communication patterns:

- **GSend<T, From, To, Next>**: Represents sending a value of type `T` from role `From` to role `To`, then continuing with protocol `Next`.
- **GRecv<T, From, To, Next>**: Represents receiving a value of type `T` by role `To` from role `From`, then continuing with protocol `Next`.
- **GChoice<Chooser, Branches>**: Represents a choice made by role `Chooser` between different protocol branches. The `Branches` parameter is a tuple of global protocols representing the different possible continuations.
- **GOffer<Offeree, Branches>**: Represents an offer received by role `Offeree` with different protocol branches. The `Branches` parameter is a tuple of global protocols representing the different possible continuations.
- **GRec<Label, Protocol>**: Represents a recursive protocol definition with label `Label` and protocol `Protocol`.
- **GVar<Label>**: Represents a reference to a recursive protocol definition with label `Label`.
- **GSeq<First, Second>**: Represents sequential composition of two protocols, where `First` is executed before `Second`.
- **GPar<First, Second>**: Represents parallel composition of two protocols, where `First` and `Second` are executed concurrently.
- **GEnd**: Represents the end of a global protocol path.

#### Global Protocol Macro

The `global_protocol!` macro provides a more intuitive way to define global protocols using a sequence diagram-inspired syntax. This makes it easier to create and understand complex communication protocols.

```rust
global_protocol! {
    protocol PingPong {
        Client -> Server: String;
        Server -> Client: String;
    }
}
```

This generates the equivalent of:

```rust
type PingPong = GSend<String, Client, Server, GRecv<String, Server, Client, GEnd>>;
```

The macro supports all the protocol patterns:

1. **Simple message passing**:
   ```rust
   Role1 -> Role2: Type;
   ```

2. **Branching and choice**:
   ```rust
   choice at Role {
       option Option1 {
           // interactions for Option1
       }
       option Option2 {
           // interactions for Option2
       }
   }
   ```

3. **Recursion**:
   ```rust
   rec Label {
       // interactions
       continue Label;
   }
   ```

4. **Sequential composition**:
   ```rust
   seq {
       include Protocol1;
       include Protocol2;
   }
   ```

5. **Parallel composition**:
   ```rust
   par {
       // first protocol
   } and {
       // second protocol
   }
   ```

The macro handles the translation of this intuitive syntax into the corresponding global protocol types, making it easier to define and understand complex protocols.

#### Example: Defining a Global Protocol

Here's an example of defining a simple global protocol using the provided types:

```rust
// Define roles
struct Client;
struct Server;

impl Role for Client {
    fn name() -> &'static str { "Client" }
}

impl Role for Server {
    fn name() -> &'static str { "Server" }
}

// Define a global protocol: Client sends a String to Server, Server sends an i32 back to Client, then ends
type RequestResponseProtocol = GSend<String, Client, Server, GRecv<i32, Server, Client, GEnd>>;
```

#### GlobalProtocolBuilder

To make it easier to construct global protocols, sessrums provides a `GlobalProtocolBuilder` helper:

```rust
let builder = GlobalProtocolBuilder::new();

// Build a simple protocol: Client sends a String to Server, then ends
let protocol = builder.send::<String, Client, Server, GEnd>();

// Build a more complex protocol: Client sends a String to Server,
// Server sends an i32 back to Client, then ends
let protocol = builder.send::<String, Client, Server, _>(
    builder.recv::<i32, Server, Client, _>(
        builder.end()
    )
);

// Build a protocol with sequential composition
let seq_protocol = builder.seq(
    builder.send::<String, Client, Server, GEnd>(),
    builder.recv::<i32, Server, Client, GEnd>()
);

// Build a protocol with parallel composition
let par_protocol = builder.par(
    builder.send::<String, Client, Server, GEnd>(),
    builder.send::<bool, Client, Logger, GEnd>()
);
```

#### Validating Global Protocols

Global protocols can be validated to ensure they are well-formed:

```rust
let protocol = GSend::<String, Client, Server, GEnd>::new();
match validate_global_protocol(&protocol) {
    Ok(_) => println!("Protocol is valid"),
    Err(e) => println!("Protocol validation error: {:?}", e),
}
```

### Projection

Projection is the process of extracting a local protocol for a specific role from a global protocol. This allows each participant to know exactly what actions they need to perform in the protocol.

The `Project` trait defines the interface for projecting a global protocol to a local protocol:

```rust
pub trait Project<R: Role> {
    /// The resulting local protocol type after projection.
    type LocalProtocol;
}
```

#### Projection Rules

The projection of a global protocol to a local protocol follows these rules:

1. **GSend<T, From, To, Next>**:
   - For the `From` role: `Send<T, <Next as Project<From>>::LocalProtocol>`
   - For the `To` role: `Recv<T, <Next as Project<To>>::LocalProtocol>`
   - For any other role R: `<Next as Project<R>>::LocalProtocol>`

2. **GRecv<T, From, To, Next>**:
   - For the `From` role: `Send<T, <Next as Project<From>>::LocalProtocol>`
   - For the `To` role: `Recv<T, <Next as Project<To>>::LocalProtocol>`
   - For any other role R: `<Next as Project<R>>::LocalProtocol>`

3. **GChoice<Chooser, Branches>**:
   - For the `Chooser` role: `Choose<Branches::LocalProtocolTuple>` where `Branches::LocalProtocolTuple` is the tuple of projected branches
   - For any other role R: `Offer<Branches::LocalProtocolTuple>` where `Branches::LocalProtocolTuple` is the tuple of projected branches

   This reflects the fact that the chooser role makes a choice between different continuations, while all other roles must offer the corresponding branches.

4. **GOffer<Offeree, Branches>**:
   - For the `Offeree` role: `Offer<Branches::LocalProtocolTuple>` where `Branches::LocalProtocolTuple` is the tuple of projected branches
   - For any other role R: `Choose<Branches::LocalProtocolTuple>` where `Branches::LocalProtocolTuple` is the tuple of projected branches

   This reflects the fact that the offeree role offers different continuations, while all other roles must choose between the corresponding branches.

5. **GRec<Label, Protocol>**:
   - For any role R: `Rec<<Protocol as Project<R>>::LocalProtocol>`

6. **GVar<Label>**:
   - For any role R: `Var<N>` where N is the recursion depth

7. **GSeq<First, Second>**:
   - For any role R: The sequential composition of the projection of `First` for role `R` followed by the projection of `Second` for role `R`.
   - In our current implementation, this is simplified to `<First as Project<R>>::LocalProtocol`.

8. **GPar<First, Second>**:
   - For any role R: The parallel composition of the projection of `First` for role `R` and the projection of `Second` for role `R`.
   - In our current implementation, this is simplified to `<First as Project<R>>::LocalProtocol`.

9. **GEnd**:
   - For any role R: `End`

#### Helper Traits

To support projection of complex global protocols, the library provides helper traits:

- `ProjectTuple<R: Role>`: Projects each element of a tuple of global protocols.
- `Into<T>`: Converts a tuple of local protocols to a `Choose` or `Offer` type.

#### The `project` Function

The library provides a `project` function that can be used to project a global protocol for a specific role:

```rust
pub fn project<G: GlobalProtocol + Project<R>, R: Role>() -> <G as Project<R>>::LocalProtocol
```

This function takes a global protocol type `G` and a role type `R` and returns the local protocol type that results from projecting `G` for role `R`.

#### Example: Projecting a Global Protocol

```rust
// Define a global protocol
type GlobalProtocol = GSend<String, Client, Server, GRecv<i32, Server, Client, GEnd>>;

// Project for the Client role
type ClientProtocol = <GlobalProtocol as Project<Client>>::LocalProtocol;
// Equivalent to: Send<String, Recv<i32, End>>

// Project for the Server role
type ServerProtocol = <GlobalProtocol as Project<Server>>::LocalProtocol;
// Equivalent to: Recv<String, Send<i32, End>>
```

#### Example: Using the `project` Function

```rust
use sessrums::proto::{global::*, roles::*, projection::project};

// Define a global protocol
type MyGlobalProtocol = GSend<String, RoleA, RoleB, GEnd>;

// Project it for RoleA
let local_protocol_a = project::<MyGlobalProtocol, RoleA>();

// Project it for RoleB
let local_protocol_b = project::<MyGlobalProtocol, RoleB>();
```

#### Example: Projecting a Complex Protocol with Branching

```rust
// Define a complex global protocol with a choice
type ComplexProtocol = GSend<String, RoleA, RoleB, GChoice<RoleB, (
    GSend<i32, RoleB, RoleA, GEnd>,
    GSend<bool, RoleB, RoleA, GEnd>
)>>;

// Project for RoleA
type RoleAProtocol = <ComplexProtocol as Project<RoleA>>::LocalProtocol;
// Equivalent to: Send<String, Offer<Recv<i32, End>, Recv<bool, End>>>

// Project for RoleB
type RoleBProtocol = <ComplexProtocol as Project<RoleB>>::LocalProtocol;
// Equivalent to: Recv<String, Choose<Send<i32, End>, Send<bool, End>>>
```

In this example:
1. RoleA sends a String to RoleB
2. RoleB then makes a choice between:
   - Sending an i32 to RoleA and ending
   - Sending a bool to RoleA and ending

When projected for RoleA, this becomes:
1. Send a String to RoleB
2. Offer a choice between:
   - Receiving an i32 from RoleB and ending
   - Receiving a bool from RoleB and ending

When projected for RoleB, this becomes:
1. Receive a String from RoleA
2. Choose between:
   - Sending an i32 to RoleA and ending
   - Sending a bool to RoleA and ending

#### Example: Multiparty Branching

```rust
// Define a global protocol with three roles
type MultipartyProtocol = GSend<bool, RoleA, RoleB, GChoice<RoleB, (
    GSend<String, RoleB, RoleC, GEnd>,
    GSend<i32, RoleB, RoleA, GEnd>
)>>;

// Project for RoleA
type RoleAProtocol = <MultipartyProtocol as Project<RoleA>>::LocalProtocol;
// Equivalent to: Send<bool, Offer<End, Recv<i32, End>>>

// Project for RoleB
type RoleBProtocol = <MultipartyProtocol as Project<RoleB>>::LocalProtocol;
// Equivalent to: Recv<bool, Choose<Send<String, End>, Send<i32, End>>>

// Project for RoleC
type RoleCProtocol = <MultipartyProtocol as Project<RoleC>>::LocalProtocol;
// Equivalent to: Offer<Recv<String, End>, End>
```

In this example with three roles:
1. RoleA sends a bool to RoleB
2. RoleB then makes a choice between:
   - Sending a String to RoleC and ending
   - Sending an i32 to RoleA and ending

When projected for each role, the protocol captures exactly what each role needs to do, including which messages to send/receive and which choices to make/offer.

#### Example: Projecting a Recursive Protocol

Recursive protocols allow for expressing repetitive behavior in communication protocols. They are defined using the `GRec<Label, Protocol>` and `GVar<Label>` types, where `Label` is a type used to identify the recursion point and `Protocol` is the body of the recursive protocol.

The `GRec<Label, Protocol>` type defines a recursive protocol with label `Label` and body `Protocol`. The `GVar<Label>` type is used within `Protocol` to refer back to the enclosing `GRec<Label, Protocol>`, creating a loop.

When projecting a recursive protocol, the `GRec<Label, Protocol>` type is projected to a `Rec<P>` type, where `P` is the projection of `Protocol`. The `GVar<Label>` type is projected to a `Var<N>` type, where `N` is the recursion depth (0 for the immediately enclosing `Rec`).

```rust
// Define a recursive global protocol
// This protocol represents:
// 1. RoleA sends a String to RoleB
// 2. RoleA then chooses to either:
//    a. Loop back to the beginning (GVar<RecursionLabel>)
//    b. End the protocol (GEnd)
struct RecursionLabel;
type RecursiveProtocol = GRec<RecursionLabel, GSend<String, RoleA, RoleB, GChoice<RoleA, (
    GVar<RecursionLabel>,
    GEnd
)>>>;

// Project for RoleA
type RoleAProtocol = <RecursiveProtocol as Project<RoleA>>::LocalProtocol;
// Equivalent to: Rec<Send<String, Choose<Var<0>, End>>>

// Project for RoleB
type RoleBProtocol = <RecursiveProtocol as Project<RoleB>>::LocalProtocol;
// Equivalent to: Rec<Recv<String, Offer<Var<0>, End>>>
```

In this example:
1. We define a recursive protocol where RoleA sends a String to RoleB and then chooses to either loop back to the beginning or end the protocol.
2. When projected for RoleA, this becomes a recursive protocol where RoleA sends a String and then chooses between looping back or ending.
3. When projected for RoleB, this becomes a recursive protocol where RoleB receives a String and then offers a choice between looping back or ending.

#### Nested Recursion

Recursive protocols can also be nested, allowing for more complex communication patterns:

```rust
// Define nested recursive protocols
struct OuterLoop;
struct InnerLoop;

// Inner loop: Server sends an i32 to Client repeatedly until Server chooses to end
type InnerProtocol = GRec<InnerLoop,
    GSend<i32, Server, Client,
        GChoice<Server, (
            GVar<InnerLoop>,
            GEnd
        )>
    >
>;

// Outer loop: Client sends a String to Server, then enters the inner loop
type GlobalProtocol = GRec<OuterLoop,
    GSend<String, Client, Server,
        InnerProtocol
    >
>;

// Project for Client
type ClientLocal = <GlobalProtocol as Project<Client>>::LocalProtocol;
// Equivalent to: Rec<Send<String, Rec<Recv<i32, Offer<Var<0>, End>>>>>

// Project for Server
type ServerLocal = <GlobalProtocol as Project<Server>>::LocalProtocol;
// Equivalent to: Rec<Recv<String, Rec<Send<i32, Choose<Var<0>, End>>>>>
```

In this nested recursion example:
1. We define an inner recursive protocol where Server repeatedly sends an i32 to Client until Server chooses to end.
2. We define an outer recursive protocol where Client sends a String to Server and then enters the inner recursive protocol.
3. When projected, the nested structure is preserved, with each role having the appropriate local protocol.

## Channel Implementation

### Chan Type

The `Chan<P, R, IO>` type represents a communication channel that follows protocol `P` from the perspective of role `R` using the underlying IO implementation `IO`.

```rust
pub struct Chan<P: Protocol, R: Role, IO> {
    io: IO,
    role: R,
    _marker: PhantomData<P>,
}
```

The `Chan` type is parameterized by:
- `P`: The protocol type that this channel follows. Must implement the `Protocol` trait.
- `R`: The role that this channel represents in the protocol. Must implement the `Role` trait.
- `IO`: The underlying IO implementation that handles the actual communication.

#### MPST Channel Support

The `Chan` type has been extended to support multiparty session types (MPST). It can now be used with local protocols that are projected from global protocols:

```rust
// Define a global protocol: RoleA sends a String to RoleB, then ends
type GlobalProtocol = GSend<String, RoleA, RoleB, GEnd>;

// Project it for RoleA
type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;

// Create a channel for RoleA
let chan = Chan::<RoleALocal, RoleA, _>::new(io);
```

The `Chan` type provides several methods specifically for working with MPST:

```rust
// Create a channel with a specific role instance
let chan = Chan::<MyProtocol, RoleA, _>::with_role(io, role);

// Convert a channel to use a different protocol type
let mpst_chan = chan.convert::<MPSTWrapper<MyProtocol, RoleA>>();

// Create a channel for a different role with the same protocol and IO
let chan_b = chan_a.for_role::<RoleB>(role_b);
```

#### Backward Compatibility

To ensure backward compatibility between binary and multiparty session types, sessrums provides a compatibility layer in the `compat` module:

```rust
use sessrums::proto::compat::{ProtocolCompat, BinaryWrapper, MPSTWrapper};
```

The `ProtocolCompat` trait allows for seamless conversion between binary and multiparty session types:

```rust
pub trait ProtocolCompat<R: Role> {
    /// The binary protocol type that is compatible with this multiparty protocol.
    type BinaryProtocol: Protocol;
    
    /// Converts a multiparty protocol to a binary protocol.
    fn to_binary(self) -> Self::BinaryProtocol;
    
    /// Converts a binary protocol to a multiparty protocol.
    fn from_binary(binary: Self::BinaryProtocol) -> Self;
}
```

The `BinaryWrapper` and `MPSTWrapper` types provide wrappers for binary and multiparty session types, respectively:

```rust
// Wrap a binary protocol for use with MPST
let binary_protocol = Send::<i32, End>::new();
let mpst_wrapper = MPSTWrapper::<Send<i32, End>, RoleA>::new(binary_protocol);

// Wrap an MPST local protocol for use with binary session types
let mpst_protocol = project::<GlobalProtocol, RoleA>();
let binary_wrapper = BinaryWrapper::<_, RoleA>::new(mpst_protocol);
```

The `ChanCompat` trait provides methods for converting channels between binary and multiparty session types:

```rust
// Convert a channel to use a binary protocol
let binary_chan = chan.to_binary();

// Convert a binary channel to use a multiparty protocol
let mpst_chan = Chan::<MPSTProtocol, RoleA, _>::from_binary(binary_chan);
```

This compatibility layer ensures that existing code using binary session types continues to work alongside new code using multiparty session types.

### IO Abstraction

The library provides a set of traits that abstract over different IO implementations:

- `Sender<T>`: A trait for sending values of type `T`.
- `Receiver<T>`: A trait for receiving values of type `T`.

These traits allow the session type system to work with various IO implementations, from simple in-memory channels to network sockets.

```rust
pub trait Sender<T> {
    type Error;
    fn send(&mut self, value: T) -> Result<(), Self::Error>;
}

pub trait Receiver<T> {
    type Error;
    fn recv(&mut self) -> Result<T, Self::Error>;
}
```

## Error Handling

The library provides a comprehensive error handling system through the `Error` enum. For detailed information about error handling, including error variants, handling patterns, and best practices, see the [Error Handling Guide](error-handling.md).

### Error Type

The library defines an `Error` enum that represents the various error conditions that might arise when using session-typed channels for communication.

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

### Error Variants

- **Io**: An error occurred in the underlying IO implementation.
- **Protocol**: A protocol violation occurred, such as unexpected messages or type mismatches.
- **Connection**: A connection error occurred during establishment or termination.
- **Serialization**: An error occurred when serializing data to be sent over the channel.
- **Deserialization**: An error occurred when deserializing received data.
- **ChannelClosed**: The channel was closed when attempting to communicate.

For more detailed information about error handling, including examples and best practices, see the [Error Handling Guide](error-handling.md).

## API Reference

### send Method

The `send` method is implemented for `Chan<Send<T, P>, IO>` and allows sending a value of type `T` over the channel.

```rust
impl<T, P: Protocol, IO> Chan<crate::proto::Send<T, P>, IO>
where
    IO: crate::io::Sender<T>,
    <IO as crate::io::Sender<T>>::Error: std::fmt::Debug,
{
    pub async fn send(mut self, value: T) -> Result<Chan<P, IO>, crate::error::Error> {
        // Send the value using the underlying IO implementation
        self.io_mut().send(value).map_err(|e| {
            // Convert the IO-specific error to our Error type
            crate::error::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Send error: {:?}", e),
            ))
        })?;

        // Return a new channel with the advanced protocol
        Ok(Chan {
            io: self.io,
            _marker: std::marker::PhantomData,
        })
    }
}
```

This method consumes the channel and returns a new channel with the advanced protocol. The protocol advances from `Send<T, P>` to `P` after sending the value.

#### Example

```rust
// Create a channel with a Send<i32, End> protocol
let chan = Chan::<Send<i32, End>, _>::new(io);

// Send a value and advance the protocol to End
let chan = chan.send(42).await?;
```

### recv Method

The `recv` method is implemented for `Chan<Recv<T, P>, IO>` and allows receiving a value of type `T` from the channel.

```rust
impl<T, P: Protocol, IO> Chan<crate::proto::Recv<T, P>, IO>
where
    IO: crate::io::Receiver<T>,
    <IO as crate::io::Receiver<T>>::Error: std::fmt::Debug,
{
    pub async fn recv(mut self) -> Result<(T, Chan<P, IO>), crate::error::Error> {
        // Receive the value using the underlying IO implementation
        let value = self.io_mut().recv().map_err(|e| {
            // Convert the IO-specific error to our Error type
            crate::error::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Receive error: {:?}", e),
            ))
        })?;

        // Return the received value and a new channel with the advanced protocol
        Ok((
            value,
            Chan {
                io: self.io,
                _marker: std::marker::PhantomData,
            },
        ))
    }
}
```

This method consumes the channel and returns the received value along with a new channel with the advanced protocol. The protocol advances from `Recv<T, P>` to `P` after receiving the value.

#### Example

```rust
// Create a channel with a Recv<i32, End> protocol
let chan = Chan::<Recv<i32, End>, _>::new(io);

// Receive a value and advance the protocol to End
let (value, chan) = chan.recv().await?;
assert_eq!(value, 42);
```

### close Method

The `close` method is implemented for `Chan<End, IO>` and closes the channel, indicating that the communication session has ended.

```rust
impl<IO> Chan<crate::proto::End, IO> {
    pub fn close(self) -> Result<(), crate::error::Error> {
        // The End protocol doesn't require any specific action to close
        // We just consume the channel and return Ok(())
        Ok(())
    }
}
```

This method consumes the channel and returns nothing on success, indicating that the protocol has been completed successfully.

#### Example

```rust
// Create a channel with an End protocol
let chan = Chan::<End, _>::new(io);

// Close the channel
chan.close()?;
```

## Usage Examples

### Simple Client-Server Protocol

This example demonstrates a simple client-server interaction using session types:

```rust
// Define the client's protocol: Send a query, receive a response, then end
type ClientProtocol = Send<String, Recv<String, End>>;

// Define the server's protocol: Receive a query, send a response, then end
// Note: This is the dual of the client's protocol
type ServerProtocol = <ClientProtocol as Protocol>::Dual;

// Implements the client side of the protocol
async fn run_client(chan: Chan<ClientProtocol, BiChannel<String>>) -> Result<(), Error> {
    println!("Client: Starting protocol");
    
    // Step 1: Send a query to the server
    println!("Client: Sending query");
    let query = "What is the meaning of life?".to_string();
    let chan = chan.send(query).await?;
    println!("Client: Query sent successfully");
    
    // Step 2: Receive the response from the server
    println!("Client: Waiting for response");
    let (response, chan) = chan.recv().await?;
    println!("Client: Received response: {}", response);
    
    // Step 3: Close the channel (end the protocol)
    println!("Client: Closing channel");
    chan.close()?;
    println!("Client: Protocol completed successfully");
    
    Ok(())
}

// Implements the server side of the protocol
async fn run_server(chan: Chan<ServerProtocol, BiChannel<String>>) -> Result<(), Error> {
    println!("Server: Starting protocol");
    
    // Step 1: Receive a query from the client
    println!("Server: Waiting for query");
    let (query, chan) = chan.recv().await?;
    println!("Server: Received query: {}", query);
    
    // Step 2: Process the query and send a response
    println!("Server: Processing query");
    let response = process_query(&query);
    println!("Server: Sending response");
    let chan = chan.send(response).await?;
    println!("Server: Response sent successfully");
    
    // Step 3: Close the channel (end the protocol)
    println!("Server: Closing channel");
    chan.close()?;
    println!("Server: Protocol completed successfully");
    
    Ok(())
}
```

### Error Handling Example

This example demonstrates how to handle errors when using session-typed channels:

```rust
// Create a custom IO implementation that fails on recv
struct FailingIO;

impl Sender<String> for FailingIO {
    type Error = ();
    
    fn send(&mut self, _value: String) -> Result<(), Self::Error> {
        println!("Send operation succeeded");
        Ok(())
    }
}

impl Receiver<String> for FailingIO {
    type Error = ();
    
    fn recv(&mut self) -> Result<String, Self::Error> {
        println!("Receive operation failed");
        Err(())
    }
}

// Create a client channel with our failing IO
let chan = Chan::<ClientProtocol, _>::new(FailingIO);

// Send should work
println!("Attempting to send a message...");
let chan = chan.send("Hello".to_string()).await.map_err(|_| {
    Error::Protocol("Unexpected send error")
})?;

// Receive should fail
println!("Attempting to receive a response (should fail)...");
match chan.recv().await {
    Ok(_) => println!("Unexpected success!"),
    Err(e) => {
        println!("Received expected error: {}", e);
        return Ok(());
    }
}
```

### Type Safety Examples

This example demonstrates how the type system ensures protocol adherence:

```rust
// Define a protocol: Send an i32, then receive a String, then end
type MyProtocol = Send<i32, Recv<String, End>>;

// The following function compiles because it follows the protocol correctly
async fn correct_protocol_usage(chan: Chan<MyProtocol, DummyIO>) {
    // First send an i32 as required by the protocol
    let chan = chan.send(42).await.unwrap();
    
    // Then receive a String as required by the protocol
    let (response, chan) = chan.recv().await.unwrap();
    let _: String = response; // Type check
    
    // Finally close the channel as required by the protocol
    chan.close().unwrap();
}

// The following function would not compile because it violates the protocol
// by trying to receive before sending
/*
async fn incorrect_protocol_usage(chan: Chan<MyProtocol, DummyIO>) {
    // Error: The protocol requires sending an i32 first, but we're trying to receive
    let (response, chan) = chan.recv().await.unwrap();
    
    // Send an i32
    let chan = chan.send(42).await.unwrap();
    
    // Close the channel
    chan.close().unwrap();
}
*/

// The following function would not compile because it violates the protocol
// by trying to send a String instead of an i32
/*
async fn incorrect_type_usage(chan: Chan<MyProtocol, DummyIO>) {
    // Error: The protocol requires sending an i32, but we're trying to send a String
    let chan = chan.send("hello".to_string()).await.unwrap();
    
    // Receive a String
    let (response, chan) = chan.recv().await.unwrap();
    
    // Close the channel
    chan.close().unwrap();
}
*/
```

## Visual Protocol Representation

### Simple Ping-Pong Protocol

```
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

Type-Level Representation:
```
Client: Send<i32, Recv<String, End>>
Server: Recv<i32, Send<String, End>>
```

### Query-Response Protocol

```
Client                                Server
  |                                     |
  |------- Send(String) - Query ------->|
  |                                     |
  |<------ Recv(String) - Response -----|
  |                                     |
  |-------------- End ---------------->|
```

Type-Level Representation:
```
Client: Send<String, Recv<String, End>>
Server: Recv<String, Send<String, End>>
```

## Advanced Topics

### Custom IO Implementations

The library allows you to create custom IO implementations by implementing the `Sender<T>` and `Receiver<T>` traits:

```rust
// Define a custom IO implementation
struct MyIO<T> {
    value: Option<T>,
}

// Define a custom error type
#[derive(Debug)]
struct MyError;

// Implement Sender for our custom IO
impl<T> Sender<T> for MyIO<T> {
    type Error = MyError;
    
    fn send(&mut self, value: T) -> Result<(), Self::Error> {
        self.value = Some(value);
        Ok(())
    }
}

// Implement Receiver for our custom IO
impl<T> Receiver<T> for MyIO<T> {
    type Error = MyError;
    
    fn recv(&mut self) -> Result<T, Self::Error> {
        self.value.take().ok_or(MyError)
    }
}

// Create a channel with our custom IO implementation
let io = MyIO { value: None };
let chan = Chan::<Send<i32, End>, _>::new(io);
```

### Protocol Testing

The library provides helper functions for testing protocol types:

```rust
// Assert that a type implements the Protocol trait
assert_protocol::<PingPongClient>();

// Assert that two types have the correct duality relationship
assert_dual::<PingPongClient, PingPongServer>();

// Create a mock channel for testing
let _client_chan: Chan<PingPongClient, ()> = mock_channel::<PingPongClient, ()>();
```

These functions help verify that protocol types are correctly defined and have the expected duality relationships.

For detailed information about testing session type protocols, including examples and best practices, see the [Testing Protocols Guide](testing-protocols.md).