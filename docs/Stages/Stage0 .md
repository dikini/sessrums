# Stage 0 Review and Task Breakdown

## Context

Stage 0 is the foundational step in the MPST DSL project. Its goal is to implement the core binary session type machinery—`Send`, `Receive`, and `End`—using the typestate pattern in Rust. This stage establishes the runtime and type-level guarantees that all later features (choice, recursion, multiparty, DSL, projection) will build upon.

---

## Review of Stage 0

**Strengths:**
- Clearly defines the minimal set of protocol states (`Send`, `Receive`, `End`) and their typestate transitions.
- Uses Rust generics and zero-sized types to encode protocol state at the type level.
- Separates protocol logic from transport via a `Transport` trait.
- Specifies robust error handling and serialization via `serde`, `bincode`, and `thiserror`.
- Includes a mock transport for testing and a test plan for a simple Ping → Pong → End protocol.

**Risks/Considerations:**
- Correctness of typestate transitions must be enforced by the type system (no runtime state leaks).
- The API must be ergonomic for both protocol implementers and future code generation.
- The transport abstraction must be flexible enough for later multiparty and real IO backends.
- Tests must ensure not only correct message flow but also that invalid protocol usage is rejected at compile time.

---

## Stage 0: Actionable Task Breakdown

### 1. Project Setup
- [x] Initialize a new Rust workspace and library crate.
- [x] Add dependencies: `serde`, `bincode`, `thiserror` in `Cargo.toml`.
- [x] Create the following directory structure:
    - `src/roles.rs`
    - `src/messages.rs`
    - `src/error.rs`
    - `src/transport.rs`
    - `src/session_types/binary.rs`
    - `src/lib.rs`
    - `tests/`

### 2. Define Roles
- [x] Implement zero-sized types for roles (e.g., `Client`, `Server`) in `src/roles.rs`.
- [x] Optionally define a `Role` trait for extensibility.

### 3. Define Message Types
- [x] Create simple message structs (e.g., `PingMsg`, `PongMsg`) in `src/messages.rs`.
- [x] Derive `Serialize`, `Deserialize`, and `PartialEq` for these types.

### 4. Error Handling
- [x] Implement a `SessionError` enum in `src/error.rs` using `thiserror`.
- [x] Cover at least: transport errors, serialization/deserialization errors, protocol violations.

### 5. Transport Abstraction
- [x] Define a `Transport` trait in `src/transport.rs` with `send_payload` and `receive_payload` methods.
- [x] Implement a `MockChannelEnd` struct for in-memory testing.
- [x] Implement serialization/deserialization helpers using `bincode`.

### 6. Protocol State Structs
- [x] In `src/session_types/binary.rs`, define:
    - `pub struct End;`
    - `pub struct Send<M, NextP>(std::marker::PhantomData<(M, NextP)>);`
    - `pub struct Receive<M, NextP>(std::marker::PhantomData<(M, NextP)>);`

### 7. Session Struct and API
- [x] Define `pub struct Session<S, T: Transport> { state: S, channel: T }`.
- [x] Implement `Session::new(channel: T) -> Self` (state is ZST).
- [x] For `Session<End, T>`, implement `fn close(self) -> T`.
- [x] For `Session<Send<M, NextP>, T>`, implement `fn send(self, message: M) -> Result<Session<NextP, T>, SessionError>`.
- [x] For `Session<Receive<M, NextP>, T>`, implement `fn receive(self) -> Result<(M, Session<NextP, T>), SessionError>`.

### 8. Testing
- [x] Write a test in `tests/ping_pong_binary_sequential.rs`:
    - Set up a mock channel pair.
    - Implement a Ping → Pong → End protocol using the typestate API.
    - Assert correct message flow and typestate transitions.
    - Ensure the compiler enforces correct usage (e.g., cannot send after End).

### 9. Documentation and Examples
- [ ] Document each struct and method with doc comments.
- [ ] Provide a minimal example in the crate root in README.md

---

## Summary Table

| Task Group                | Actions                                                                                 |
|---------------------------|----------------------------------------------------------------------------------------|
| Project Setup             | Workspace, dependencies, directory structure                                            |
| Roles                     | ZSTs for roles, `Role` trait                                                           |
| Messages                  | Message structs, derive traits                                                         |
| Error Handling            | `SessionError` enum, error variants                                                    |
| Transport                 | `Transport` trait, `MockChannelEnd`, serialization helpers                             |
| Protocol State Structs    | `End`, `Send`, `Receive` structs                                                       |
| Session Struct & API      | `Session<S, T>`, constructor, `send`, `receive`, `close` methods                       |
| Testing                   | Ping-Pong test, typestate enforcement                                                  |
| Documentation/Examples    | Doc comments, minimal usage example                                                    |

---

**Each of these tasks is self-contained and can be implemented and tested independently, ensuring a solid, verifiable foundation for the more advanced stages of your MPST project.**