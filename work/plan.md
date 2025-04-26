**Project Goal:** Develop a Rust library for asynchronous session types with minimal dependencies, focusing on expressing the process calculus in the types using Rust's type system features, including `const generics`.

**1. Core Concepts Review**

- **(Unchanged)** Session Types, Asynchronous, Process Calculus (Send `!`, Receive `?`, Offer `&`, Choose `+`, Recursion `μ`/`X`, End), Duality, Type State Pattern.
- **Minimal Dependencies:** Primarily `std`, `futures-core`. Avoid large async runtimes in the core library.

**2. Design Considerations**

- **Representing Protocol Types:** ZSTs (`struct Send<T, P>(...)`, `struct Recv<T, P>(...)`, `struct End;`, etc.) using `PhantomData`. Enums for choices.
- **The Channel Type:** `struct Chan<P: Protocol, IO>` with `PhantomData<P>`. `IO` represents the underlying async communication primitive.
- **Operations as Methods:** `async fn` methods on `Chan` that consume `self` and return `Result<Chan<NewState, IO>, Error>`.
- **Underlying Communication (`IO`):** Prefer a generic `IO` constrained by custom async traits (`AsyncSender<T>`, `AsyncReceiver<T>`). User provides the concrete `IO` type. Avoid bundling a default channel unless absolutely necessary (and feature-flagged).
- **Recursion (Using Const Generics):**
    - Represent recursion points using `struct Rec<P: Protocol>(PhantomData<P>);`.
    - Represent recursion variables (references back to a `Rec`) using `struct Var<const N: usize>(PhantomData<[(); N]>);`. `N` will be a compile-time integer index (e.g., 0, 1, ...).
    - This approach leverages a built-in Rust feature, avoiding external dependencies like `typenum`.
    - The main challenge lies in relating `Var<N>` back to its corresponding `Rec<P>` and performing type-level checks, which will likely involve helper traits.
    - Be aware that performing arithmetic on `N` _within type bounds_ (e.g., `N + 1`) generally requires the unstable `generic_const_exprs` feature. Design should aim to avoid needing this, handling the logic within trait implementations instead.
- **Error Handling:** Define a clear library `Error` type.
- **Duality:** Use a `Protocol` trait with `type Dual: Protocol;` implemented for all protocol types.
    
    Rust
    
    ```rust
    trait Protocol: Sized { type Dual: Protocol; }
    // ... impls for Send, Recv, End, Offer, Choose ...
    impl<const N: usize> Protocol for Var<N> { type Dual = Var<N>; } // Vars are often self-dual
    impl<P: Protocol> Protocol for Rec<P> { type Dual = Rec<P::Dual>; }
    ```
    
- **Ergonomics:** Consider type aliases, helper functions, or macros later to manage complex type signatures.

**3. Comparison: `const generics` vs. `typenum` for `Var<N>`**

This section provides context for the decision to use `const generics`:

- **`const generics` (`struct Var<const N: usize>`)**
    
    - **Pros:**
        
        - **No External Dependency:** Uses a built-in Rust language feature, perfectly aligning with the minimal dependencies goal.
        - **Standard Feature:** Leverages a core language component.
        - **Clean Syntax:** `Var`, `Var` can be more direct than type-based representations.
        
    - **Cons:**
        
        - **Limited Stable Type-Level Arithmetic:** Performing calculations like `N + 1` _within type definitions or bounds_ generally requires the unstable `generic_const_exprs` feature (as of April 2025). Workarounds involve encoding logic in trait implementations.
        - **Error Messages:** Can sometimes be less clear than specialized libraries in complex scenarios (though improving).
        
    
- **`typenum` crate (`struct Var<N: typenum::Unsigned>`)**
    
    - **Pros:**
        
        - **Rich Type-Level Arithmetic:** Provides traits (`Add`, `Sub`, `Compare`, etc.) for complex computations on types representing numbers, all on stable Rust.
        - **Mature & Specific:** Dedicated library for type-level numbers.
        
    - **Cons:**
        
        - **External Dependency:** Adds a dependency to the project, contrary to the primary goal.
        - **More Verbose Syntax:** Requires using `typenum`'s types (e.g., `typenum::U0`, `typenum::U1`, `typenum::Succ<N>`).
        
    
- **Decision for this Project:** Use **`const generics`**. The benefit of eliminating an external dependency is significant. The primary limitation (stable type-level arithmetic) is considered manageable for the typical recursion patterns in session types, where indices (`N`) primarily serve as identifiers rather than requiring complex calculations within type bounds.
    

**4. Development Plan (Phased Approach)**

Phase 1**Project Goal:** Develop a Rust library for asynchronous session types with minimal dependencies, focusing on expressing the process calculus in the types using Rust's type system features, including `const generics`.

**1. Core Concepts Review**

- **(Unchanged)** Session Types, Asynchronous, Process Calculus (Send `!`, Receive `?`, Offer `&`, Choose `+`, Recursion `μ`/`X`, End), Duality, Type State Pattern.
- **Minimal Dependencies:** Primarily `std`, `futures-core`. Avoid large async runtimes in the core library.

**2. Design Considerations**

- **Representing Protocol Types:** ZSTs (`struct Send<T, P>(...)`, `struct Recv<T, P>(...)`, `struct End;`, etc.) using `PhantomData`. Enums for choices.
- **The Channel Type:** `struct Chan<P: Protocol, IO>` with `PhantomData<P>`. `IO` represents the underlying async communication primitive.
- **Operations as Methods:** `async fn` methods on `Chan` that consume `self` and return `Result<Chan<NewState, IO>, Error>`.
- **Underlying Communication (`IO`):** Prefer a generic `IO` constrained by custom async traits (`AsyncSender<T>`, `AsyncReceiver<T>`). User provides the concrete `IO` type. Avoid bundling a default channel unless absolutely necessary (and feature-flagged).
- **Recursion (Using Const Generics):**
    - Represent recursion points using `struct Rec<P: Protocol>(PhantomData<P>);`.
    - Represent recursion variables (references back to a `Rec`) using `struct Var<const N: usize>(PhantomData<[(); N]>);`. `N` will be a compile-time integer index (e.g., 0, 1, ...).
    - This approach leverages a built-in Rust feature, avoiding external dependencies like `typenum`.
    - The main challenge lies in relating `Var<N>` back to its corresponding `Rec<P>` and performing type-level checks, which will likely involve helper traits.
    - Be aware that performing arithmetic on `N` _within type bounds_ (e.g., `N + 1`) generally requires the unstable `generic_const_exprs` feature. Design should aim to avoid needing this, handling the logic within trait implementations instead.
- **Error Handling:** Define a clear library `Error` type.
- **Duality:** Use a `Protocol` trait with `type Dual: Protocol;` implemented for all protocol types.
    
    Rust
    
    ```rust
    trait Protocol: Sized { type Dual: Protocol; }
    // ... impls for Send, Recv, End, Offer, Choose ...
    impl<const N: usize> Protocol for Var<N> { type Dual = Var<N>; } // Vars are often self-dual
    impl<P: Protocol> Protocol for Rec<P> { type Dual = Rec<P::Dual>; }
    ```
    
- **Ergonomics:** Consider type aliases, helper functions, or macros later to manage complex type signatures.

**3. Comparison: `const generics` vs. `typenum` for `Var<N>`**

This section provides context for the decision to use `const generics`:

- **`const generics` (`struct Var<const N: usize>`)**
    
    - **Pros:**
        
        - **No External Dependency:** Uses a built-in Rust language feature, perfectly aligning with the minimal dependencies goal.
        - **Standard Feature:** Leverages a core language component.
        - **Clean Syntax:** `Var`, `Var` can be more direct than type-based representations.
        
    - **Cons:**
        
        - **Limited Stable Type-Level Arithmetic:** Performing calculations like `N + 1` _within type definitions or bounds_ generally requires the unstable `generic_const_exprs` feature (as of April 2025). Workarounds involve encoding logic in trait implementations.
        - **Error Messages:** Can sometimes be less clear than specialized libraries in complex scenarios (though improving).
        
    
- **`typenum` crate (`struct Var<N: typenum::Unsigned>`)**
    
    - **Pros:**
        
        - **Rich Type-Level Arithmetic:** Provides traits (`Add`, `Sub`, `Compare`, etc.) for complex computations on types representing numbers, all on stable Rust.
        - **Mature & Specific:** Dedicated library for type-level numbers.
        
    - **Cons:**
        
        - **External Dependency:** Adds a dependency to the project, contrary to the primary goal.
        - **More Verbose Syntax:** Requires using `typenum`'s types (e.g., `typenum::U0`, `typenum::U1`, `typenum::Succ<N>`).
        
    
- **Decision for this Project:** Use **`const generics`**. The benefit of eliminating an external dependency is significant. The primary limitation (stable type-level arithmetic) is considered manageable for the typical recursion patterns in session types, where indices (`N`) primarily serve as identifiers rather than requiring complex calculations within type bounds.
    

**4. Development Plan (Phased Approach)**

**Phase 1**: Core Type Definitions & Duality
* Setup project
* Define `Send<T, P>`, `Recv<T, P>`, End.
* Define Protocol trait with Dual. Implement for basic types.

**Phase 2**: Channel Abstraction & Basic IO Traits
* Define `Chan<P: Protocol, IO>`.
* Define minimal `Sender<T>` / `Receiver<T>` traits for IO.
* Define `Offer<L, R>`, `Choose<L, R>` and implement Protocol for them.

**Phase 3**: Implement send and recv
*  Implement `async fn send(...) for Chan<Send<...>>`.
* Implement `async fn recv(...) for Chan<Recv<...>>`.
* Implement `close() for Chan<End, IO>`.
* Define basic `Error enum`.

**Phase 4**: Add asynchronous traits for IO
- add futures-core.
* Define minimal `AsyncSender<T>` / `AsyncReceiver<T>` traits for IO.
* Implement `async fn offer(...) for Chan<Offer<...>>`.
* Implement `async fn choose_left(...)` / `choose_right(...)` (or `choose(...)) for Chan<Choose<...>>`.

**Phase 5:** Implement Bounded Recursion (Using Const Generics)
* Define `Rec<P>` and `Var<const N: usize>`.
* Implement Protocol for Rec and Var.
* Implement `enter(self) -> Chan<P, IO> for Chan<Rec<P>, IO>`.
* Implement mechanism (e.g., defer(self) or specific trait impls) to relate states back to `Chan<Var<N>, IO>`. This requires careful design of helper traits to manage the indices N without unstable features if possible.

* Start simple: Ensure basic non-recursive protocols work perfectly first.

**Phase 6**: Connection Establishment
* Provide functions like `fn connect<P: Protocol, IO>(io: IO) -> (Chan<P, IO>, Chan<P::Dual, IO>)` or wrappers for existing streams/channels.

**Phase 7**: Asynchronous Runtime Integration & Examples
* Write examples using tokio or async-std to demonstrate providing a concrete IO type.
* Ensure futures are Send where appropriate.

**Phase 8**: Testing & Refinement
* Compile-time tests (invalid sequences fail to compile).
* Runtime tests (using e.g. futures-channel in dev-dependencies).
* Refine API, error handling, documentation.

**5. Minimal Dependencies Strategy**

- **Core:** `std`, `futures-core`.
- **No `typenum`:** Rely on built-in `const generics`.
- **Optional Bundled IO:** Strongly discouraged. If ever added, use minimal crates (e.g., `futures-channel`) and feature flags.
- **Dev-dependencies:** `tokio`, `async-std`, `futures-channel` etc. are acceptable for tests and examples.

This updated plan incorporates `const generics` as the chosen approach for handling recursion variables, aligning strongly with your goal of minimal dependencies, while also documenting the trade-offs involved compared to `typenum`.
