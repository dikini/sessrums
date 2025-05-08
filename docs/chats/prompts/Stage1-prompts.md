# Stage 1 Implementation Prompts

---

## Shared Context

You are working on Stage 1 of the sessrums-types project, which extends the binary session type system with external choice (Offer/Select). The codebase already implements Send, Receive, End, and a typestate Session API. The goal is to add protocol branching using `Offer<L, R>` and `Select<L, R>`, with robust type safety and test coverage. See the Stage 1 plan and MPST DSL Review for details.

---

### Task: Define ChoiceSignal and Either Enums

**Prompt:**  
In `src/session_types/binary.rs`, define:
- `pub enum ChoiceSignal { Left, Right }` (derive Serialize, Deserialize, Debug, Clone, PartialEq)
- `pub enum Either<L, R> { Left(L), Right(R) }` (derive Debug, Clone, PartialEq)

Add doc comments explaining their role in protocol branching and how they are used in the session API.

---

### Task: Define Offer and Select Protocol State Structs

**Prompt:**  
In `src/session_types/binary.rs`, define:
- `pub struct Offer<L, R>(PhantomData<(L, R)>)`
- `pub struct Select<L, R>(PhantomData<(L, R)>)`

Add doc comments explaining the meaning of each protocol state and how they are used to model branching.

---

### Task: Implement Session API for Choice

**Prompt:**  
Implement the following methods:
- On `Session<Select<L, R>, T>`:  
  - `pub fn select_left(self) -> Result<Session<L, T>, SessionError>`
  - `pub fn select_right(self) -> Result<Session<R, T>, SessionError>`
  - Each sends a `ChoiceSignal` over the transport.
- On `Session<Offer<L, R>, T>`:  
  - `pub fn offer(self) -> Result<Either<Session<L, T>, Session<R, T>>, SessionError>`
  - Receives a `ChoiceSignal` and returns the appropriate session branch.

Add doc comments and usage examples for each method.

---

### Task: Implement Duality for Offer and Select

**Prompt:**  
Implement or update the `Dual` trait for `Offer<L, R>` and `Select<L, R>`, ensuring:
- `Dual<Offer<L, R>> = Select<Dual<L>, Dual<R>>`
- `Dual<Select<L, R>> = Offer<Dual<L>, Dual<R>>`

Document the duality relationships and provide examples.

---

### Task: Ensure Transport Support for ChoiceSignal

**Prompt:**  
Ensure that `ChoiceSignal` can be serialized/deserialized via the `Transport` trait and its implementations (e.g., `MockChannelEnd`). Document how choice signals are transmitted and any caveats for implementers of custom transports.

---

### Task: Write Tests for Protocols with Choices

**Prompt:**  
In `tests/binary_choice.rs`, write comprehensive tests covering:
- A protocol using `Offer` and `Select` (e.g., a simple menu or accept/reject).
- Both left and right branches.
- Correct message flow and typestate transitions.
- Compiler enforcement of correct usage (e.g., cannot select after branch taken).

Add doc comments to each test explaining the protocol and what is being verified.

---

### Task: Document All New Items

**Prompt:**  
For every new public struct, enum, and method added in Stage 1, add a Rust doc comment (`/// ...`) describing its purpose, usage, and any important details. Ensure all doc comments are clear, concise, and include usage examples where appropriate.

---

### Task: Add a Minimal Example to README

**Prompt:**  
Add a minimal, self-contained example to `README.md` that demonstrates a protocol with branching using `Offer` and `Select`. Ensure the example is copy-pastable and highlights the core API and typestate transitions for choice.

---

---

**How to use:**  
For each task, provide the shared context and the specific prompt to the implementer or LLM.  
Each prompt is self-contained and references only what is needed for the task.  
If several tasks are closely related, you may batch them, but keep prompts focused.

**Optimization rationale:**  
- The shared context is minimal but sufficient; do not repeat it for every prompt.
- Each prompt is direct, actionable, and references only the necessary files and types.
- Doc comments and examples are required for all public API changes to ensure clarity and maintainability.