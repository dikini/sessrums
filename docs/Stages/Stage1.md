# Stage 1 Review and Task Breakdown

## Context

Stage 1 builds on the binary session type foundation by introducing **external choice** (branching) via `Select<L, R>` and `Offer<L, R>`. This enables protocols where one party offers a choice between two continuations, and the other party selects which branch to take. This is essential for modeling real-world protocols with alternatives (e.g., accept/reject, continue/abort).

---

## Review of Stage 1

**Strengths:**
- Introduces protocol branching, enabling more expressive protocols.
- Maintains typestate guarantees and transport abstraction.
- Uses enums (`ChoiceSignal`, `Either`) to encode choices at the type and value level.
- Extends the session API with methods for making and offering choices.

**Risks/Considerations:**
- Correct duality between `Offer` and `Select` must be enforced.
- Choice signals must be robustly encoded and decoded.
- Tests must cover both branches and invalid usage.
- API ergonomics for handling choices must be clear and type-safe.

---

## Stage 1: Actionable Task Breakdown

### 1. Project Preparation
- **Short:** Ensure Stage 0 is complete and all tests pass.
    - **Implementation Prompt:**  
      Run all Stage 0 tests and resolve any failures. Ensure the codebase is clean and ready for extension.
    - **Documentation Prompt:**  
      Document in the changelog or project notes that Stage 0 is a prerequisite for Stage 1.

---

### 2. Define ChoiceSignal and Either Enum
- **Short:** In `src/session_types/binary.rs`, define:
    - `pub enum ChoiceSignal { Left, Right }` (derive `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`)
    - `pub enum Either<L, R> { Left(L), Right(R) }` (derive `Debug`, `Clone`, `PartialEq`)
    - **Implementation Prompt:**  
      Implement the `ChoiceSignal` and `Either` enums with the required derives. Ensure `ChoiceSignal` is serializable for transport.
    - **Documentation Prompt:**  
      Add doc comments to both enums explaining their role in protocol branching and how they are used in the session API.

---

### 3. Protocol State Structs for Choice
- **Short:** In `src/session_types/binary.rs`, define:
    - `pub struct Offer<L, R>(PhantomData<(L, R)>)`
    - `pub struct Select<L, R>(PhantomData<(L, R)>)`
    - **Implementation Prompt:**  
      Implement the `Offer` and `Select` zero-sized types using `PhantomData` for generic parameters.
    - **Documentation Prompt:**  
      Add doc comments explaining the meaning of each protocol state and how they are used to model branching.

---

### 4. Session API for Choice
- **Short:** Implement on `Session<Select<L, R>, T>`:
    - `pub fn select_left(self) -> Result<Session<L, T>, SessionError>`
    - `pub fn select_right(self) -> Result<Session<R, T>, SessionError>`
    - Each sends a `ChoiceSignal` over the transport.
- **Short:** Implement on `Session<Offer<L, R>, T>`:
    - `pub fn offer(self) -> Result<Either<Session<L, T>, Session<R, T>>, SessionError>`
    - Receives a `ChoiceSignal` and returns the appropriate session branch.
    - **Implementation Prompt:**  
      Implement the above methods, ensuring correct serialization/deserialization of `ChoiceSignal` and correct typestate transitions.
    - **Documentation Prompt:**  
      Document each method, including usage examples and error cases.

---

### 5. Duality
- **Short:** Implement or update the `Dual` trait for `Offer<L, R>` and `Select<L, R>`, ensuring:
    - `Dual<Offer<L, R>> = Select<Dual<L>, Dual<R>>`
    - `Dual<Select<L, R>> = Offer<Dual<L>, Dual<R>>`
    - **Implementation Prompt:**  
      Implement the `Dual` trait for the new protocol states, ensuring type-level correctness.
    - **Documentation Prompt:**  
      Document the duality relationships and provide examples.

---

### 6. Transport Support for ChoiceSignal
- **Short:** Ensure `ChoiceSignal` can be serialized/deserialized via the `Transport` trait.
    - **Implementation Prompt:**  
      Update the `Transport` trait and its implementations (e.g., `MockChannelEnd`) to support sending and receiving `ChoiceSignal`.
    - **Documentation Prompt:**  
      Document how choice signals are transmitted and any caveats for implementers of custom transports.

---

### 7. Testing
- **Short:** Create `tests/binary_choice.rs`:
    - Set up a mock channel pair.
    - Implement a protocol using `Offer` and `Select` (e.g., a simple menu or accept/reject).
    - Test both left and right branches.
    - Assert correct message flow and typestate transitions.
    - Ensure the compiler enforces correct usage (e.g., cannot select after branch taken).
    - **Implementation Prompt:**  
      Write comprehensive tests covering both branches and invalid transitions. Use `MockChannelEnd` for transport.
    - **Documentation Prompt:**  
      Add doc comments to each test explaining the protocol and what is being verified.

---

### 8. Documentation and Examples
- **Short:** Document each new struct, enum, and method with doc comments.
    - **Implementation Prompt:**  
      For every new public item, add a Rust doc comment (`/// ...`) describing its purpose, usage, and any important details.
    - **Documentation Prompt:**  
      Ensure all doc comments are clear, concise, and include usage examples where appropriate.

- **Short:** Add a minimal example using `Offer`/`Select` to the crate root or `README.md`.
    - **Implementation Prompt:**  
      Write a minimal, self-contained example in `README.md` that demonstrates a protocol with branching using `Offer` and `Select`.
    - **Documentation Prompt:**  
      Ensure the example is copy-pastable and highlights the core API and typestate transitions for choice.

---

## Summary Table

| Task Group                | Actions                                                                                 |
|---------------------------|----------------------------------------------------------------------------------------|
| Project Preparation       | Ensure Stage 0 complete, dependencies                                                  |
| Choice Enums              | `ChoiceSignal`, `Either`                                                               |
| Protocol State Structs    | `Offer`, `Select` structs                                                              |
| Session API               | `select_left`, `select_right`, `offer` methods                                         |
| Duality                   | Dual trait for choice types                                                            |
| Transport                 | Serialization for `ChoiceSignal`                                                       |
| Testing                   | Choice protocol tests, typestate enforcement                                           |
| Documentation/Examples    | Doc comments, minimal usage example                                                    |

---

**Each of these tasks is self-contained and can be implemented and tested independently, ensuring a solid, verifiable foundation for the more advanced stages of your MPST project.**