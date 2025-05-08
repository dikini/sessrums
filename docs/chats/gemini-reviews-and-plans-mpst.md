# Multiparty Session Types DSL: Review and Implementation Plan

## 1. Introduction

This document provides a review of the provided materials concerning the design and implementation of a Domain-Specific Language (DSL) for Multiparty Session Types (MPST) in Rust. Following the review, a detailed iterative implementation plan is proposed to guide the development of such a system. The goal is to create a DSL that allows intuitive expression of complex protocols while generating type-level representations for compile-time verification.

## 2. Code and Design Review

Two main inputs were reviewed:
1.  `recursive_session_types.txt`: Containing Rust code exploring two approaches for implementing recursive binary session types.
2.  `session-types-dsl.txt`: A design document outlining the components of a DSL for MPST.

### 2.1. Review of `recursive_session_types.txt`

This file presented two concrete Rust implementations for handling session types, focusing on recursion.

#### 2.1.1. Approach 3: Higher-Ranked Trait Bounds (HRTBs) and Continuation-Passing Style

*   **Key Features:**
    *   `Context` trait for transport abstraction (send/receive).
    *   `Protocol<C: Context>` trait with an `execute` method and `Continue` associated type.
    *   `Mu<F>` struct for recursion using `F: FnOnce(Mu<F>) -> P`.
    *   `Send<T, P>`, `Receive<T, P>`, `End` combinators.
*   **Strengths:**
    *   Sound recursion mechanism.
    *   Good transport agnosticism via `Context`.
*   **Weaknesses & Issues:**
    *   **Critical Bug (Message Framing):** `TcpStream::receive` uses `read_to_end`, which is incorrect for message-based protocols as it reads until EOF, not per message. This requires proper message framing (e.g., length-prefixing).
    *   **Error Handling:** Uses `expect()`, leading to panics instead of propagating errors.
    *   **Binary Focus:** Inherently designed for two-party sessions.
    *   **Manual Projection:** Roles (client/server) are manually defined; no automatic projection from a global type.

#### 2.1.2. Approach 4: Session Type Combinators with Fixed Points

*   **Key Features:**
    *   `Protocol` trait with a `run` method and `Next` associated type for continuations.
    *   `Channel` trait for transport abstraction (`Read + Write`).
    *   `Fix<F>` struct for recursion, similar to `Mu`, used with an `Unfold` trait.
    *   `Send<T, P>`, `Receive<T, P>`, `End` combinators.
    *   `Choose<L, R>` and `Offer<L, R>` for external choice.
*   **Strengths:**
    *   Closer to typical session type combinator style, good for DSL generation.
    *   Sound recursion.
    *   Transport agnosticism via `Channel`.
    *   Includes choice combinators.
    *   Uses `TryFrom` for message deserialization (an improvement).
*   **Weaknesses & Issues:**
    *   **Critical Bug (Message Framing):** Similar `read_to_end` issue if `TcpStream` is used as a `Channel` without modification.
    *   **Error Handling:** Also uses `expect()`.
    *   **Bug in `ping_client_with_stop`:** The client example always chooses to continue; it doesn't demonstrate selecting the "stop" branch.
    *   **Choice Signaling:** Uses raw bytes (0/1) for choice, which is brittle.
    *   **Binary Focus:** Also inherently two-party.
    *   **Manual Projection.**

#### 2.1.3. Overall Assessment of `recursive_session_types.txt`

Both approaches provide valuable groundwork for binary session types and recursion. Approach 4 is particularly well-suited as a target for code generation from a DSL due to its explicit combinator style. Key improvements needed are message framing, robust error handling, and then extension to multiparty concepts.

### 2.2. Review of `session-types-dsl.txt`

This file presented a higher-level design sketch for an MPST DSL system.

*   **Section 1: DSL Syntax Definition**
    *   **Strengths:** Intuitive Mermaid-like syntax covering participants, messages, choice, and recursion.
    *   **Areas for Detail:** Payload type system (how DSL types like `String` map to Rust types), scope of `rec` labels, syntax for parallel composition (`par`).

*   **Section 2: Macro System for DSL Parsing**
    *   **Strengths:** Standard proc macro approach (`#[protocol]`).
    *   **Areas for Detail:** Definition of the `GlobalProtocol` trait that the macro generates code for; how roles and message types from the DSL are defined in Rust.

*   **Section 3: Type-Level Representation**
    *   **Strengths:** Defines `GlobalInteraction` and `LocalProtocol` enums with common constructs (Message, Choice, Rec, Var, Par, End).
    *   **Areas for Detail:** The `Next = ()` generic parameter on these enums needs careful refinement. If `Next` is the continuation, then branches of `Choice` or `Par` having the *same* `Next` type parameter (inherited from the enum definition) is too restrictive. A direct recursive structure (e.g., `continuation: Box<GlobalInteraction>`) or more advanced GADT-like patterns are typically used.

*   **Section 4: Projection Mechanism**
    *   **Strengths:** Outlines a `Project<R: Role>` trait. Captures the basic idea of mapping global actions to local sends/receives/selects/offers.
    *   **Areas for Detail:**
        *   Handling of roles *not involved* in a `Message` interaction (type mismatch in current sketch).
        *   Projection of `Par` is highly simplified and needs a complete algorithm considering role participation in branches.
        *   Definition of `Role` trait and `R::is_role`.

*   **Section 5 & 6: DSL Usage and Client Code Implementation**
    *   **Strengths:** Illustrates the intended workflow from DSL definition to client-side session usage.
    *   **Areas for Detail:**
        *   The `session.receive()` on an `Offer` state is oversimplified. Type-safe handling of choices usually requires a more structured API (e.g., methods to handle each branch explicitly or receiving an `Either` type).
        *   How the `Session` type evolves its state (`P` in `Session<P, T>`).

*   **Section 7: Transport Abstraction**
    *   **Strengths:** Good `Transport` trait using `serde Serialize/DeserializeOwned`. The `Session<P, T>` struct holds protocol state and transport.
    *   **Areas for Detail:** The generic `P` in `Session<P, T>` needs to be a concrete typestate (like `struct Send<...>`), not the generic `LocalProtocol` enum. `AsSend`/`AsReceive` helper traits (hinted at) are crucial for this.

*   **Section 8, 9, 10: Combinators, Algebra, Full Example**
    *   **Strengths:** Show how the DSL could map to Rust combinator functions and touch on session type algebra concepts (`then`, `par`, `dual`).
    *   **Areas for Detail:** `GlobalProtocol` trait is still undefined. `dual()` on a local protocol is unusual and needs clarification.

#### 2.2.1. Overall Assessment of `session-types-dsl.txt`

The document provides a good high-level vision. It correctly identifies the major components of an MPST DSL system. However, many critical type-system mechanics, projection details, and session API specifics are underspecified and require significant elaboration to ensure correctness and feasibility.

### 2.3. Combined Recommendations from Review

1.  **Fix Critical Bugs:** Address message framing and error handling in any runtime implementation.
2.  **Refine Type Representations:** The `Next` parameter in `GlobalInteraction`/`LocalProtocol` needs careful design to correctly model protocol branches and continuations.
3.  **Detail Projection Logic:** Especially for uninvolved roles and parallel composition.
4.  **Flesh out Session API:** Particularly for choice handling (`Offer`/`Select`) and how the `Session` struct transitions its state type `P`.
5.  **Iterative Development:** Build the system incrementally, starting with core runtime mechanics.

## 3. Iterative Implementation Plan

This plan aims to build the MPST DSL system incrementally, focusing on a runtime-first approach, compiler verification, and thorough testing at each stage.

### 3.1. Overall Philosophy

*   **Runtime First, DSL Last:** Build core session type mechanics and type safety first. The DSL and proc macro will target this runtime.
*   **Incremental Complexity:** Start with binary, sequential protocols, then add choice, recursion, multiparty, and finally the DSL frontend.
*   **Compiler as Verifier:** Leverage Rust's type system at each stage.
*   **Test-Driven:** Accompany functional additions with unit tests.
*   **Concrete Examples:** Use Ping-Pong and a FileTransfer-like protocol as running examples.

### 3.2. Core Definitions (Established early and evolved)

*   **Roles:** Zero-Sized Types (ZSTs).
    ```rust
    // src/roles.rs
    pub struct Client;
    pub struct Server;
    pub struct Storage; // Added later
    // Trait to group roles, potentially for future use in identifying participants
    pub trait Role: Copy + Clone + Send + Sync + 'static {}
    impl Role for Client {}
    impl Role for Server {}
    impl Role for Storage {}
    ```
*   **Message Payloads:** Simple structs, eventually using `serde`.
    ```rust
    // src/messages.rs
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct PingMsg { pub data: String }
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct PongMsg { pub data: String }

    // Example for file transfer
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct RequestFilename { pub filename: String }
    // ... other message types
    ```
*   **Error Type:**
    ```rust
    // src/error.rs
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum SessionError {
        #[error("Transport error: {0}")]
        Transport(#[from] std::io::Error), // Example, refine as needed
        #[error("Serialization error: {0}")]
        Serialization(String), // Example for serde errors
        #[error("Deserialization error: {0}")]
        Deserialization(String),
        #[error("Protocol violation: {0}")]
        Protocol(String),
        #[error("Choice signal error: {0}")]
        ChoiceSignal(String),
    }
    ```
*   **Mock Transport:** For testing protocol logic.
    ```rust
    // src/transport.rs
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use serde::{Serialize, de::DeserializeOwned};
    use crate::error::SessionError;

    // Defines how messages are framed for transmission
    // For simplicity, we'll use bincode for serialization to Vec<u8>
    fn serialize_payload<M: Serialize>(payload: &M) -> Result<Vec<u8>, SessionError> {
        bincode::serialize(payload).map_err(|e| SessionError::Serialization(e.to_string()))
    }

    fn deserialize_payload<M: DeserializeOwned>(bytes: Vec<u8>) -> Result<M, SessionError> {
        bincode::deserialize(&bytes).map_err(|e| SessionError::Deserialization(e.to_string()))
    }
    
    pub trait Transport: Send + 'static {
        fn send_payload<M: Serialize>(&mut self, message: M) -> Result<(), SessionError>;
        fn receive_payload<M: DeserializeOwned>(&mut self) -> Result<M, SessionError>;
    }

    #[derive(Debug)]
    pub struct MockChannelEnd {
        name: String, // For debugging
        send_buf: Arc<Mutex<VecDeque<Vec<u8>>>>,
        recv_buf: Arc<Mutex<VecDeque<Vec<u8>>>>,
    }

    impl MockChannelEnd {
        fn send_bytes(&mut self, bytes: Vec<u8>) -> Result<(), SessionError> {
            // println!("{}: Sending {} bytes", self.name, bytes.len());
            self.send_buf.lock().unwrap().push_back(bytes);
            Ok(())
        }
        fn recv_bytes(&mut self) -> Result<Vec<u8>, SessionError> {
            match self.recv_buf.lock().unwrap().pop_front() {
                Some(bytes) => {
                    // println!("{}: Received {} bytes", self.name, bytes.len());
                    Ok(bytes)
                },
                None => Err(SessionError::Transport(
                    std::io::Error::new(std::io::ErrorKind::WouldBlock, "MockChannel: no message")
                )),
            }
        }
    }
    
    impl Transport for MockChannelEnd {
        fn send_payload<M: Serialize>(&mut self, message: M) -> Result<(), SessionError> {
            let bytes = serialize_payload(&message)?;
            self.send_bytes(bytes)
        }
        fn receive_payload<M: DeserializeOwned>(&mut self) -> Result<M, SessionError> {
            let bytes = self.recv_bytes()?;
            deserialize_payload(bytes)
        }
    }

    pub fn new_mock_channel_pair() -> (MockChannelEnd, MockChannelEnd) {
        let client_to_server = Arc::new(Mutex::new(VecDeque::new()));
        let server_to_client = Arc::new(Mutex::new(VecDeque::new()));
        (
            MockChannelEnd { name: "ClientEnd".to_string(), send_buf: client_to_server.clone(), recv_buf: server_to_client.clone() },
            MockChannelEnd { name: "ServerEnd".to_string(), send_buf: server_to_client, recv_buf: client_to_server },
        )
    }
    ```

### Stage 0: Foundational Binary Session Types (Send/Receive/End)

*   **Description:** Implement basic session types for two-party communication: `Send<M, Next>`, `Receive<M, Next>`, `End`. Focus on the type-state pattern where the session object's type changes per operation.
*   **Structures and Algorithms:**
    *   Protocol state structs: `End`, `Send<M, NextP>`, `Receive<M, NextP>`.
    *   `Session<CurrentState, T: Transport>` struct.
*   **Code Artifacts:**
    *   `src/roles.rs`: (defined above)
    *   `src/messages.rs`: (defined above)
    *   `src/error.rs`: (defined above)
    *   `src/transport.rs`: (defined above)
    *   `src/session_types/binary.rs`:
        *   `pub struct End;`
        *   `pub struct Send<M, NextP>(std::marker::PhantomData<(M, NextP)>);`
        *   `pub struct Receive<M, NextP>(std::marker::PhantomData<(M, NextP)>);`
        *   `pub struct Session<S, T: Transport> { state: S, channel: T }`
        *   `impl<S, T: Transport> Session<S, T> { pub fn new(channel: T) -> Self { Session { state: std::marker::PhantomData, channel } }` (State is ZST)
        *   `impl<T: Transport> Session<End, T> { pub fn close(self) -> T { self.channel } }`
        *   `impl<M, NextP, T: Transport> Session<Send<M, NextP>, T> where M: Serialize + 'static, NextP: 'static { pub fn send(self, message: M) -> Result<Session<NextP, T>, SessionError> { /* ... */ } }`
        *   `impl<M, NextP, T: Transport> Session<Receive<M, NextP>, T> where M: DeserializeOwned + 'static, NextP: 'static { pub fn receive(self) -> Result<(M, Session<NextP, T>), SessionError> { /* ... */ } }`
    *   `src/lib.rs`: Module declarations.
    *   `tests/ping_pong_binary_sequential.rs`: Test cases.
*   **Pre-conditions:**
    *   Rust environment set up. `serde`, `bincode`, `thiserror` dependencies added.
*   **Post-conditions:**
    *   Code compiles.
    *   Compiler enforces correct message types and operation order for sequences.
    *   Tests pass for Ping -> Pong -> End.
*   **Implementation Plan:**
    1.  Setup project structure, `Cargo.toml`.
    2.  Implement `roles.rs`, `messages.rs`, `error.rs`.
    3.  Implement `transport.rs` with `MockChannelEnd` and `bincode` serialization.
    4.  Implement `End`, `Send`, `Receive` ZSTs in `session_types/binary.rs`.
    5.  Implement `Session` struct and its `new`, `send`, `receive`, `close` methods.
    6.  Write and pass tests.
*   **Concepts Involved:** Typestate Pattern, Generics, PhantomData, Traits.

---

### Stage 1: Binary Session Types with External Choice (Offer/Select)

*   **Description:** Add external choice: `Select<L, R>` (choosing role) and `Offer<L, R>` (offering role).
*   **Structures and Algorithms:**
    *   Protocol state structs: `Select<L, R>`, `Offer<L, R>`.
    *   `ChoiceSignal` enum for signaling choice over transport.
    *   `Session` methods: `select_left`/`select_right` for `Select`, `offer` for `Offer` (returning an `Either` enum).
*   **Code Artifacts:**
    *   `src/session_types/binary.rs` (add):
        *   `pub struct Select<L, R>(std::marker::PhantomData<(L, R)>);`
        *   `pub struct Offer<L, R>(std::marker::PhantomData<(L, R)>);`
        *   `#[derive(Serialize, Deserialize)] enum ChoiceSignal { Left, Right }`
        *   `pub enum Either<S1, S2> { Left(S1), Right(S2) }`
        *   
        ```rust
        impl<L, R, T: Transport> Session<Select<L, R>, T> { 
            pub fn select_left(self) -> Result<Session<L, T>, SessionError> { /* send ChoiceSignal::Left */ } 
            pub fn select_right(self) -> Result<Session<R, T>, SessionError> { /* send ChoiceSignal::Right */ } 
        }
        ``` 
        (where L, R are 'static)
        *   
        ```rust
        impl<L, R, T: Transport> Session<Offer<L, R>, T> { 
            pub fn offer(self) -> Result<Either<Session<L, T>, Session<R, T>>, SessionError> { /* receive ChoiceSignal, return Either */ } }
        ``` 
        (where L, R are 'static)
    *   `tests/binary_choice.rs`.
*   **Pre-conditions:** Stage 0 completed.
*   **Post-conditions:** Code compiles. Compiler enforces choice logic. Tests for choice protocols pass.
*   **Implementation Plan:**
    1.  Define `ChoiceSignal` and `Either` enums.
    2.  Implement `Select` and `Offer` ZSTs.
    3.  Implement `select_left`/`select_right` and `offer` methods on `Session`.
    4.  Write tests for protocols with choices.
*   **Concepts Involved:** External Choice, Sum Types.

---

### Stage 2: Binary Session Types with Recursion

*   **Description:** Introduce recursion using a fixed-point style, similar to Approach 4 from `recursive_session_types.txt`.
*   **Structures and Algorithms:**
    *   `Rec<P>` struct, where `P` is a type that, when "unrolled," yields a protocol body. `P` will typically be a closure `F: FnOnce(Var) -> Body`.
    *   `Var` struct: marker for recursion point.
    *   The core idea is that `Session<Rec<P>, T>` methods will unroll `P` to get the actual next step.
*   **Code Artifacts:**
    *   `src/session_types/binary.rs` (add/refactor):
        *   `pub struct Var<RecMarker = Z>(std::marker::PhantomData<RecMarker>);` (Z is a base recursion marker)
        *   `pub struct Rec<F>(F);` // F: FnOnce(Var) -> P_Body, P_Body is some protocol state.
        *   This requires making the `Session` methods more sophisticated or introducing helper traits. A common way:
            ```rust
            pub trait ProtocolState: 'static {}
            impl ProtocolState for End {}
            impl<M: 'static, NextP: ProtocolState> ProtocolState for Send<M, NextP> {}
            // ... for Receive, Select, Offer

            // For methods on Session<S, T>:
            // If S = Rec<F>, then F is called with Var, producing P_Body.
            // The operation then proceeds as if the state was P_Body.
            // This means the method signatures may need to handle this "unrolling" step.
            // e.g. Session<Rec<impl FnOnce(Var) -> Send<Msg, Next>>, T>::send(...)
            // This can get complex with HRTBs or by boxing the closure.
            // Simpler: Session methods check S. If S is Rec, unroll and call again on Session<Unrolled, T>.
            // This needs careful handling of the `self` consumption and type transitions.
            ```
        *   A practical approach: Define `Session::enter_rec` for `Rec` states that returns a session typed with the loop body.
    *   `tests/binary_recursive.rs`: Recursive Ping-Pong.
*   **Pre-conditions:** Stage 1 completed.
*   **Post-conditions:** Code compiles. Recursive protocols are type-checked. Tests for recursive Ping-Pong (e.g., 3 iterations, possibly combined with choice to terminate) pass.
*   **Implementation Plan:**
    1.  Define `Rec<F>` and `Var`.
    2.  Refactor `Session` methods or introduce a mechanism (e.g., helper traits or direct unrolling logic within methods) to handle `Rec` states. This is the most complex part of this stage. The goal is for `Session<Rec<F>, T>` to behave like `Session<Body_of_F, T>`.
    3.  Implement recursive Ping-Pong using `Rec`, `Var`, and choice for termination.
    4.  Test.
*   **Concepts Involved:** Fixed-Point Combinators, Higher-Order Functions/Types.

---

### Stage 3: Basic Multiparty Primitives & Manual Global/Local Types

*   **Description:** Shift to multiparty. Define `GlobalInteraction` (Message, End) and `LocalProtocol` (Send, Receive, End) enums. Manually write a global spec and corresponding local specs for 3+ parties. No automated projection yet.
*   **Structures and Algorithms:**
    *   `src/session_types/common.rs`: `RoleIdentifier` (e.g., an enum or string type), `Participant<R: Role>`.
    *   `src/session_types/global.rs`: `enum GlobalInteraction { Message { from: RoleIdentifier, to: RoleIdentifier, msg: PhantomData<M>, cont: Box<GlobalInteraction> }, End }` (simplified, direct recursion).
    *   `src/session_types/local.rs`: `enum LocalProtocol { Send { to: RoleIdentifier, msg: PhantomData<M>, cont: Box<LocalProtocol> }, Receive { from: RoleIdentifier, msg: PhantomData<M>, cont: Box<LocalProtocol> }, End }`.
    *   `src/transport.rs`: `MultipartyTransport` trait, `MockMultipartyBroker` to manage multiple `MockChannelEnd`s. Each participant gets a `ParticipantChannel` that knows its own role and can send/receive with specific other roles via the broker.
*   **Code Artifacts:** As above.
*   **Pre-conditions:** Stage 2 completed.
*   **Post-conditions:** Manually defined global and local protocols for a 3-party interaction compile. Participants can execute their local protocols using `MockMultipartyBroker`.
*   **Implementation Plan:**
    1.  Define `RoleIdentifier` and related multiparty participant structures.
    2.  Implement basic `GlobalInteraction` and `LocalProtocol` enums (just Message/End for now).
    3.  Design and implement `MockMultipartyBroker` and `ParticipantChannel` (which implements a refined `Transport` trait aware of send/recv roles).
    4.  Manually write a simple 3-party protocol (e.g., A -> B: Msg1; B -> C: Msg2) as a `GlobalInteraction` instance and corresponding `LocalProtocol` instances for A, B, C.
    5.  Write a test where A, B, C execute these local protocols concurrently (using threads or async).
*   **Concepts Involved:** Multiparty Session Types (basic structures), Inter-process Communication (simulated).

---

### Stage 4: Automated Projection (Message, End, Choice)

*   **Description:** Implement automated projection from `GlobalInteraction` to `LocalProtocol` for `Message`, `End`, and `Choice`.
*   **Structures and Algorithms:**
    *   `Project<MyRole: Role>` trait with `project(global: GlobalInteraction) -> LocalProtocol<MyRole>`.
    *   Extend `GlobalInteraction` and `LocalProtocol` with `Choice`/`Select`/`Offer` variants compatible with multiparty roles.
        *   `GlobalInteraction::Choice { decider: RoleIdentifier, branches: Vec<(Label, GlobalInteraction)> }`
        *   `LocalProtocol::Select { decider: RoleIdentifier, choices: Vec<(Label, LocalProtocol)> }`
        *   `LocalProtocol::Offer { decider: RoleIdentifier, choices: Vec<(Label, LocalProtocol)> }`
    *   Implement `impl<MyRole: Role> Project<MyRole> for GlobalInteraction`.
*   **Code Artifacts:** `src/projection.rs`. Refactor `global.rs`, `local.rs`.
*   **Pre-conditions:** Stage 3 completed.
*   **Post-conditions:** Automated projection correctly generates `LocalProtocol` for simple non-recursive, choice-based global protocols. These projected protocols can run.
*   **Implementation Plan:**
    1.  Add multiparty-aware Choice constructs to `GlobalInteraction` and `Select`/`Offer` to `LocalProtocol`.
    2.  Implement the `Project` trait and its `project` method for `Message`, `End`, and `Choice`. Pay careful attention to uninvolved roles (their projection becomes the projection of the continuation).
    3.  Test by defining a `GlobalInteraction` with choices, projecting it for each role, and running the resulting local protocols.
*   **Concepts Involved:** Projection Algorithm (for choice).

---

### Stage 5: Projection for Recursion & Full Multiparty Types

*   **Description:** Extend `GlobalInteraction` and `LocalProtocol` with `Rec`/`Var`. Implement projection for these. This completes the core type system.
*   **Structures and Algorithms:**
    *   `GlobalInteraction::Rec { label: RecLabel, body: Box<GlobalInteraction> }`, `GlobalInteraction::Var { label: RecLabel }`.
    *   `LocalProtocol::Rec { label: RecLabel, body: Box<LocalProtocol> }`, `LocalProtocol::Var { label: RecLabel }`.
    *   Extend `Project` trait for `Rec`/`Var`.
*   **Code Artifacts:** Refactor `global.rs`, `local.rs`, `projection.rs`.
*   **Pre-conditions:** Stage 4 completed.
*   **Post-conditions:** Projection works for recursive multiparty protocols.
*   **Implementation Plan:**
    1.  Add `Rec`/`Var` to global and local types (using labels instead of De Bruijn for simplicity first).
    2.  Extend projection logic: `Rec` projects to `Rec` with projected body. `Var` projects to `Var`.
    3.  Test with a recursive multiparty protocol.
*   **Concepts Involved:** Projection for Recursion.

---

### Stage 6: Multiparty Session Runtime

*   **Description:** Develop a robust `MultipartySession<MyRole, CurrentLocalState, AllChannels>` struct that uses the projected `LocalProtocol` to guide type-safe communication for a specific role.
*   **Structures and Algorithms:**
    *   `MultipartySession` whose `CurrentLocalState` parameter changes with each operation, similar to the binary `Session`.
    *   Methods like `send_to<OtherRole, Msg>(...)`, `receive_from<OtherRole, Msg>(...)`, `select_choice(...)`, `offer_choices(...)`.
*   **Code Artifacts:** `src/session_types/multiparty_session.rs`.
*   **Pre-conditions:** Stage 5 completed. The `LocalProtocol` enum can be generated.
*   **Post-conditions:** Roles can execute projected local protocols in a type-safe manner using `MultipartySession`.
*   **Implementation Plan:**
    1.  Design the `MultipartySession` struct, leveraging typestates for `CurrentLocalState`.
    2.  Implement methods for Send, Receive, Select, Offer, Rec (unrolling), End. These methods will interact with the `ParticipantChannel` (from Stage 3) to communicate with specific roles.
    3.  Test extensively with various projected protocols.
*   **Concepts Involved:** Multiparty Typestates.

---

### Stage 7: DSL Parser and AST Definition

*   **Description:** Implement `parse_protocol_dsl` (e.g., using `pest`) to convert the Mermaid-like DSL string into a Rust AST (a separate set of structs representing the DSL's syntax tree, not `GlobalInteraction` directly yet).
*   **Code Artifacts:** `src/dsl/parser.rs`, `src/dsl/ast.rs`.
*   **Pre-conditions:** -
*   **Post-conditions:** DSL examples parse into correct, well-defined AST representations. Error reporting for syntax errors.
*   **Implementation Plan:**
    1.  Write a `pest` grammar for the DSL syntax.
    2.  Define Rust structs for the DSL AST.
    3.  Implement the parser logic to transform `pest`'s parse tree into this AST.
    4.  Test parsing with valid and invalid DSL strings.
*   **Concepts Involved:** Parsing (Pest), Abstract Syntax Trees.

---

### Stage 8: Proc Macro for Global Protocol Generation

*   **Description:** Implement the `#[protocol]` procedural macro. It uses the parser (Stage 7) to get the DSL AST, then transforms this AST into Rust code that instantiates the `GlobalInteraction` enum (from Stage 5).
*   **Code Artifacts:** A new crate for the proc macro (e.g., `mpst_macros`).
*   **Pre-conditions:** Stage 5 (GlobalInteraction definition) and Stage 7 (DSL parser) completed.
*   **Post-conditions:** `#[protocol] const P_DSL: &str = "...";` correctly generates a function like `fn P_DSL_global() -> GlobalInteraction { ... }`.
*   **Implementation Plan:**
    1.  Set up the proc macro crate.
    2.  In the proc macro:
        *   Take the string literal input.
        *   Call the DSL parser to get the DSL AST.
        *   Write a transformer function: DSL AST -> `GlobalInteraction` instance (as `TokenStream` using `quote`). This involves generating participant role identifiers and message type markers.
    3.  Test the proc macro.
*   **Concepts Involved:** Procedural Macros, Code Generation (`quote`).

---

### Stage 9: Full End-to-End Integration

*   **Description:** Connect all pieces: DSL (`#[protocol]`) -> `GlobalInteraction` instance -> `project()` -> `LocalProtocol` instance -> `MultipartySession` execution.
*   **Pre-conditions:** All preceding stages.
*   **Post-conditions:** A complex protocol defined in the DSL can be parsed, projected for each role, and then type-safe role implementations using `MultipartySession` can execute it correctly.
*   **Implementation Plan:**
    1.  Create example projects/tests that use `#[protocol]` to define a global protocol.
    2.  Implement role functions that:
        *   Get the generated global protocol.
        *   Project it for their specific role.
        *   Instantiate `MultipartySession` with the projected local protocol and appropriate channels.
        *   Execute the protocol.
    3.  Thoroughly test this full pipeline.
*   **Concepts Involved:** System Integration.

---

### Stage 10: Advanced Features & Polish

*   **Description:** Implement `GlobalInteraction::Par` and its projection. Consider session type algebra (e.g., a `then` combinator for `GlobalInteraction`). Improve Serde integration for diverse message payloads. Refine error handling, APIs, and documentation.
*   **Pre-conditions:** Stage 9.
*   **Post-conditions:** A more feature-rich and robust MPST library.
*   **Implementation Plan:** Iteratively add features like `Par`, ensure documentation is complete, optimize, and respond to usability feedback.
*   **Concepts Involved:** Parallel Composition in Session Types, API Design, Documentation.

This iterative plan provides a roadmap for developing the MPST DSL system in Rust, ensuring verifiability and manageability at each step.
