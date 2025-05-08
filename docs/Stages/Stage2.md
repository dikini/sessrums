# Stage 2 Review and Task Breakdown

## Context

Stage 2 extends the binary session type system by introducing **recursion**. This enables the modeling of protocols with loops or repeated interactions, such as repeated ping-pong or file transfer chunks. Recursion is typically implemented using a fixed-point combinator (`Rec`) and a recursion variable (`Var`), following established patterns in session types literature.

---

### Symbol Index

### src/error.rs
Defines error types and handling for session operations using thiserror.

- enum SessionError : error.rs (6-22)
- impl From<bincode::Error> for SessionError : error.rs (24-28)
- impl<T> From<PoisonError<T>> for SessionError : error.rs (30-34)
- #[cfg(test)] mod tests : error.rs (36-56)
  - #[test] fn test_error_messages : error.rs (38-49)
  - #[test] fn test_error_conversion : error.rs (51-55)

### src/lib.rs
Crate root with module declarations and public API exports.

- pub mod roles : lib.rs (6-6)
- pub mod messages : lib.rs (7-7)
- pub mod error : lib.rs (8-8)
- pub mod transport : lib.rs (9-9)
- pub mod session_types : lib.rs (10-10)
- pub use error::SessionError : lib.rs (13-13)
- pub use transport::Transport : lib.rs (14-14)

### src/messages.rs
Defines serializable message types (PingMsg, PongMsg) using serde.

- struct PingMsg : messages.rs (8-11)
- struct PongMsg : messages.rs (13-17)
- #[cfg(test)] mod tests : messages.rs (19-43)
  - #[test] fn test_ping_serialization : messages.rs (24-30)
  - #[test] fn test_pong_serialization : messages.rs (32-41)

### src/roles.rs
Implements protocol roles (Client, Server) as zero-sized types with a sealed Role trait.

- trait Role : roles.rs (7-7)
- struct Client : roles.rs (14-15)
- struct Server : roles.rs (17-18)
- impl Role for Client : roles.rs (20-20)
- impl Role for Server : roles.rs (21-21)
- mod private : roles.rs (24-28)
- #[cfg(test)] mod tests : roles.rs (30-45)
  - #[test] fn roles_are_copy : roles.rs (33-44)

### src/session_types/mod.rs
Module organization and re-exports for session type components.

- pub mod binary : session_types/mod.rs (3-3)
- pub use binary::{End, Send, Receive, Session, Offer, Select, Dual, Either, ChoiceSignal} : session_types/mod.rs (5-5)

### src/session_types/binary.rs
Core implementation of binary session types using the typestate pattern.

- enum ChoiceSignal : binary.rs (28-33)
- enum Either<L, R> : binary.rs (79-87)
- struct Send<M, NextP> : binary.rs (188-189)
- struct Receive<M, NextP> : binary.rs (235-236)
- struct Offer<L, R> : binary.rs (304-305)
- struct Select<L, R> : binary.rs (365-366)
- struct Session<S, T: Transport> : binary.rs (413-416)
- trait Dual : binary.rs (907-911)
- impl Dual for End : binary.rs (913-917)
- impl<M, P> Dual for Send<M, P> : binary.rs (918-926)
- impl<M, P> Dual for Receive<M, P> : binary.rs (927-935)
- impl<L, R> Dual for Offer<L, R> : binary.rs (946-970)
- impl<L, R> Dual for Select<L, R> : binary.rs (992-1034)
- impl<S, T: Transport> Session<S, T> : binary.rs (419-429)
- impl<T: Transport> Session<End, T> : binary.rs (430-434)
- impl<M, NextP, T: Transport> Session<Send<M, NextP>, T> : binary.rs (444-456)
- impl<M, NextP, T: Transport> Session<Receive<M, NextP>, T> : binary.rs (457-469)
- impl<L, R, T: Transport> Session<Select<L, R>, T> : binary.rs (483-583)
- impl<L, R, T: Transport> Session<Offer<L, R>, T> : binary.rs (592-665)
- #[cfg(test)] mod tests : binary.rs (670-870)
  - #[test] fn test_ping_pong_protocol : binary.rs (678-699)
  - #[test] fn test_session_type_safety : binary.rs (701-710)
  - #[test] fn test_choice_protocol : binary.rs (731-806)
  - #[test] fn test_duality_relationships : binary.rs (817-859)

### src/transport.rs
Transport abstraction trait and mock implementation for testing.

- trait Transport : transport.rs (7-19)
- struct MockChannelEnd : transport.rs (21-27)
- impl MockChannelEnd : transport.rs (29-44)
- impl Transport for MockChannelEnd : transport.rs (46-61)
- #[cfg(test)] mod tests : transport.rs (62-131)
  - #[derive(Debug, Serialize, Deserialize, PartialEq)] struct TestMessage : transport.rs (65-68)
  - #[test] fn test_mock_channel : transport.rs (70-96)
  - #[test] fn test_queue_behavior : transport.rs (98-112)
  - #[test] fn test_choice_signal_transmission : transport.rs (114-128)
  - #[test] fn test_multiple_messages : transport.rs (130-131)
---

## Review of Stage 2

**Strengths:**
- Enables expressive protocols with repetition and looping.
- Maintains typestate guarantees and transport abstraction.
- Follows established patterns for recursion in session types (fixed-point combinators).

**Risks/Considerations:**
- Correct handling of recursion variables and unrolling is subtle and error-prone.
- The API must make recursion ergonomic and type-safe.
- Tests must ensure both finite and infinite (or bounded) recursion works as intended.
- Recursion must compose correctly with choice and sequencing.

---

## Stage 2: Actionable Task Breakdown

### 1. Project Preparation
- **Short:** Ensure Stage 1 is complete and all tests pass.
    - **Implementation Prompt:**  
      Run all Stage 1 tests and resolve any failures. Ensure the codebase is clean and ready for extension.
    - **Documentation Prompt:**  
      Document in the changelog or project notes that Stage 1 is a prerequisite for Stage 2.

---

### 2. Define Recursion Types
- **Short:** In `src/session_types/binary.rs`, define:
    - `pub struct Rec<F>(F);` where `F: FnOnce(Var) -> PBody`
    - `pub struct Var;` as a marker for the recursion point.
    - **Implementation Prompt:**  
      Implement the `Rec` and `Var` types. `Rec` should wrap a closure or type-level function that, when given a `Var`, produces the protocol body.
    - **Documentation Prompt:**  
      Add doc comments explaining the purpose of `Rec` and `Var`, and how they are used to encode loops in protocols.

---

### 3. Protocol State Trait for Recursion
- **Short:** Define a `ProtocolState` trait (or similar) to unify all protocol state types, including recursion.
    - **Implementation Prompt:**  
      Implement a trait (e.g., `ProtocolState`) and ensure all protocol state structs (`End`, `Send`, `Receive`, `Select`, `Offer`, `Rec`, `Var`) implement it.
    - **Documentation Prompt:**  
      Document the trait and its role in enabling generic handling of protocol states.

---

### 4. Session API for Recursion
- **Short:** Implement on `Session<Rec<F>, T>`:
    - `pub fn enter_rec(self) -> Session<PBody, T>` (where `PBody` is the body produced by `F(Var)`).
    - **Implementation Prompt:**  
      Implement a method to "unroll" the recursion by invoking the closure in `Rec` with a `Var`, returning a session in the body state.
    - **Documentation Prompt:**  
      Document the method, including usage examples and how it enables looping.

- **Short:** Ensure `Session<Var, T>` can be used to jump back to the recursion point.
    - **Implementation Prompt:**  
      Implement logic so that when the protocol state is `Var`, the session can jump back to the start of the recursion (i.e., re-enter the body).
    - **Documentation Prompt:**  
      Document how recursion variables are used and how the API supports looping.

---

### 5. Duality
- **Short:** Implement or update the `Dual` trait for `Rec` and `Var`, ensuring:
    - `Dual<Rec<F>> = Rec<Dual<F>>`
    - `Dual<Var> = Var`
    - **Implementation Prompt:**  
      Implement the `Dual` trait for the new recursion protocol states, ensuring type-level correctness.
    - **Documentation Prompt:**  
      Document the duality relationships and provide examples.

---

### 6. Testing
- **Short:** Create `tests/binary_recursive.rs`:
    - Set up a mock channel pair.
    - Implement a recursive protocol (e.g., repeated ping-pong with a counter).
    - Test both finite and infinite (or bounded) recursion.
    - Assert correct message flow and typestate transitions.
    - Ensure the compiler enforces correct usage (e.g., cannot send after End, recursion variable is only used in the correct context).
    - **Implementation Prompt:**  
      Write comprehensive tests covering recursion, including both branches and invalid transitions. Use `MockChannelEnd` for transport.
    - **Documentation Prompt:**  
      Add doc comments to each test explaining the protocol and what is being verified.

---

### 7. Documentation and Examples
- **Short:** Document each new struct, trait, and method with doc comments.
    - **Implementation Prompt:**  
      For every new public item, add a Rust doc comment (`/// ...`) describing its purpose, usage, and any important details.
    - **Documentation Prompt:**  
      Ensure all doc comments are clear, concise, and include usage examples where appropriate.

- **Short:** Add a minimal example using recursion to the crate root or `README.md`.
    - **Implementation Prompt:**  
      Write a minimal, self-contained example in `README.md` that demonstrates a protocol with recursion using `Rec` and `Var`.
    - **Documentation Prompt:**  
      Ensure the example is copy-pastable and highlights the core API and typestate transitions for recursion.

---

## Summary Table

| Task Group                | Actions                                                                                 |
|---------------------------|----------------------------------------------------------------------------------------|
| Project Preparation       | Ensure Stage 1 complete, dependencies                                                  |
| Recursion Types           | `Rec`, `Var` structs                                                                   |
| Protocol State Trait      | `ProtocolState` trait for all protocol states                                          |
| Session API               | `enter_rec` method, recursion variable handling                                        |
| Duality                   | Dual trait for recursion types                                                         |
| Testing                   | Recursive protocol tests, typestate enforcement                                        |
| Documentation/Examples    | Doc comments, minimal usage example                                                    |

---

**Each of these tasks is self-contained and can be implemented and tested independently, ensuring a solid, verifiable foundation for the more advanced stages of your MPST project.**