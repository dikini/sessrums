# Stage 2 Implementation Prompts

## Common Context

You are working on Stage 2 of the `sessrums-types` project, which extends the binary session type system with **recursion** using a fixed-point style (see Approach 4 in `recursive_session_types.txt`).  
The codebase already supports Send, Receive, End, Offer, Select, and typestate Session API.  
Your goal is to add recursive protocol support using `Rec<F>` and `Var`, with robust type safety and test coverage.  
**Reference:**  
- [docs/MPST_DSL-Review AndImplementation.md](../MPST_DSL-Review%20AndImplementation.md) (Stage 2 plan, design rationale, and requirements)  
- [docs/code-map-result.md](../code-map-result.md) (current codebase map and symbol index)  
**Pre-condition:** Stage 1 is complete and all tests pass.

---

## Task 1: Define Rec and Var Types

**Prompt:**  
In `src/session_types/binary.rs`, define:
- `pub struct Rec<F>(F);` where `F: FnOnce(Var) -> PBody`
- `pub struct Var<RecMarker = Z>(std::marker::PhantomData<RecMarker>);` (Z is a base recursion marker)

Add concise doc comments explaining their role in recursive protocols and how they are used to encode loops in session types.

**Description:**  
These types enable recursive protocol definitions. `Rec` wraps a closure or type-level function that, when given a `Var`, produces the protocol body. `Var` is a marker for the recursion point.

---

## Task 2: Implement ProtocolState Trait for Recursion

**Prompt:**  
Define a `ProtocolState` trait (or similar) in `src/session_types/binary.rs` and implement it for all protocol state structs, including `Rec` and `Var`.

**Description:**  
This trait unifies all protocol state types, allowing generic handling of protocol states, including recursion. Ensure all protocol state structs (`End`, `Send`, `Receive`, `Select`, `Offer`, `Rec`, `Var`) implement it.

---

## Task 3: Implement Session API for Recursion

**Prompt:**  
Implement on `Session<Rec<F>, T>`:
- `pub fn enter_rec(self) -> Session<PBody, T>` (where `PBody` is the body produced by `F(Var)`).

Ensure `Session<Var, T>` can be used to jump back to the recursion point.  
Add doc comments and a minimal usage example for each method.

**Description:**  
This allows unrolling recursion by invoking the closure in `Rec` with a `Var`, returning a session in the body state. The API should make recursion ergonomic and type-safe.

---

## Task 4: Implement Duality for Rec and Var

**Prompt:**  
Implement or update the `Dual` trait for `Rec` and `Var`, ensuring:
- `Dual<Rec<F>> = Rec<Dual<F>>`
- `Dual<Var> = Var`

Document the duality relationships and provide a short example.

**Description:**  
Duality ensures that recursive protocols have correct duals for safe communication.

---

## Task 5: Write Tests for Recursive Protocols

**Prompt:**  
In `tests/binary_recursive.rs`, write comprehensive tests covering:
- A recursive protocol (e.g., repeated ping-pong with a counter).
- Both finite and infinite (or bounded) recursion.
- Correct message flow and typestate transitions.
- Compiler enforcement of correct usage (e.g., cannot send after End, recursion variable is only used in the correct context).

Add doc comments to each test explaining the protocol and what is being verified.

**Description:**  
Tests should demonstrate recursive protocol execution, type safety, and correct duality.

---

## Task 6: Document All New Items

**Prompt:**  
For every new public struct, enum, and method added in Stage 2, add a Rust doc comment (`/// ...`) describing its purpose, usage, and any important details.  
Ensure all doc comments are clear, concise, and include usage examples where appropriate.

**Description:**  
Documentation is required for maintainability and for future users of the recursion API.

---

## Task 7: Add a Minimal Example to README

**Prompt:**  
Add a minimal, self-contained example to `README.md` that demonstrates a protocol with recursion using `Rec` and `Var`.  
Ensure the example is copy-pastable and highlights the core API and typestate transitions for recursion.

**Description:**  
This example should help users understand how to define and use recursive protocols with the new API.

---

**How to use:**  
For each task, provide the common context and the specific prompt to the implementer or LLM.  
Each prompt is self-contained and references only what is needed for the task.  
Refer to [docs/code-map-result.md](../code-map-result.md) for symbol locations and to avoid duplication or conflicts.