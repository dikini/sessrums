# Stage 5 Review and Task Breakdown

## Context

Stage 5 extends the multiparty session type system to support **recursion**, a critical feature for expressing protocols with loops or repeated interactions. This stage builds on the previous work of implementing basic multiparty primitives and automated projection for message passing and choice. By adding recursion support, we complete the core type system needed for expressing realistic protocols with loops, enabling the modeling of complex interactions like file transfers with multiple chunks or persistent connections with repeated operations.

---

## Review of Stage 5

**Strengths:**
- Enables expression of infinite or bounded repetition in protocols
- Completes the core type system for multiparty session types
- Maintains static type safety for recursive protocols
- Follows established patterns in session type literature

**Risks/Considerations:**
- Correct handling of recursion variables during projection is subtle
- Recursion must compose correctly with other protocol constructs (choice, message passing)
- Projection of recursive protocols must preserve well-formedness
- Testing recursive protocols requires careful verification of both finite and potentially infinite behaviors

---

## Stage 5: Actionable Task Breakdown

### 1. Project Preparation
- **Short:** Ensure Stage 4 is complete and all tests pass.
    - **Implementation Prompt:**  
      Run all Stage 4 tests and resolve any failures. Ensure the codebase is clean and ready for extension.
    - **Documentation Prompt:**  
      Document in the changelog that Stage 4 is a prerequisite for Stage 5.
    - **Pre-conditions:** Stage 4 implementation complete
    - **Post-conditions:** All Stage 4 tests pass

---

### 2. Define Recursion Label Type
- **Short:** Define a type to represent recursion labels/variables.
    - **Implementation Prompt:**  
      Create a `RecursionLabel` type (e.g., a newtype wrapper around `String` or a struct with an identifier) to uniquely identify recursion points in protocols.
    - **Documentation Prompt:**  
      Document the purpose of recursion labels and how they're used to identify recursion points in protocols.
    - **Pre-conditions:** None
    - **Post-conditions:** `RecursionLabel` type is defined and documented
    - **Test:** Unit tests for `RecursionLabel` creation and comparison

```rust
// Example implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecursionLabel(String);

impl RecursionLabel {
    pub fn new(label: impl Into<String>) -> Self {
        RecursionLabel(label.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

---

### 3. Extend Global Protocol with Recursion
- **Short:** Add `Rec` and `Var` variants to `GlobalInteraction`.
    - **Implementation Prompt:**  
      Extend the `GlobalInteraction` enum with `Rec` and `Var` variants. `Rec` should contain a label and a boxed body, while `Var` should reference a label.
    - **Documentation Prompt:**  
      Document the new variants, explaining how they represent recursion points and variable references in global protocols.
    - **Pre-conditions:** `RecursionLabel` type defined
    - **Post-conditions:** `GlobalInteraction` includes recursion variants
    - **Test:** Unit tests for creating recursive global protocols

```rust
// Example extension to GlobalInteraction
pub enum GlobalInteraction {
    // Existing variants...
    
    /// Recursion point in the protocol
    Rec {
        /// Label identifying this recursion point
        label: RecursionLabel,
        /// Body of the recursive protocol
        body: Box<GlobalInteraction>,
    },
    
    /// Reference to a recursion point
    Var {
        /// Label of the referenced recursion point
        label: RecursionLabel,
    },
}
```

---

### 4. Extend Local Protocol with Recursion
- **Short:** Add `Rec` and `Var` variants to `LocalProtocol`.
    - **Implementation Prompt:**  
      Extend the `LocalProtocol` enum with `Rec` and `Var` variants, mirroring the structure in `GlobalInteraction`.
    - **Documentation Prompt:**  
      Document the new variants, explaining how they represent recursion in local protocols and their relationship to the global variants.
    - **Pre-conditions:** `RecursionLabel` type defined
    - **Post-conditions:** `LocalProtocol` includes recursion variants
    - **Test:** Unit tests for creating recursive local protocols

```rust
// Example extension to LocalProtocol
pub enum LocalProtocol<R: Role> {
    // Existing variants...
    
    /// Recursion point in the local protocol
    Rec {
        /// Label identifying this recursion point
        label: RecursionLabel,
        /// Body of the recursive protocol
        body: Box<LocalProtocol<R>>,
    },
    
    /// Reference to a recursion point
    Var {
        /// Label of the referenced recursion point
        label: RecursionLabel,
    },
}
```

---

### 5. Implement Projection for Recursion
- **Short:** Extend the `Project` trait implementation to handle `Rec` and `Var`.
    - **Implementation Prompt:**  
      Implement projection for `Rec` and `Var` variants. For `Rec`, project the body and wrap it in a local `Rec`. For `Var`, create a corresponding local `Var`.
    - **Documentation Prompt:**  
      Document the projection rules for recursion, explaining how recursion points and variables are preserved during projection.
    - **Pre-conditions:** `GlobalInteraction` and `LocalProtocol` extended with recursion variants
    - **Post-conditions:** Projection correctly handles recursive protocols
    - **Test:** Tests for projecting recursive protocols to different roles

```rust
// Example projection implementation for Rec and Var
impl<R: Role, M: Clone> Project<R, M> for GlobalInteraction<M> {
    type Output = LocalProtocol<R, M>;
    
    fn project(self) -> Self::Output {
        match self {
            // Existing cases...
            
            GlobalInteraction::Rec { label, body } => {
                LocalProtocol::Rec {
                    label,
                    body: Box::new(body.project()),
                    _role: PhantomData,
                }
            },
            
            GlobalInteraction::Var { label } => {
                LocalProtocol::Var {
                    label,
                    _role: PhantomData,
                }
            },
        }
    }
}
```

---

### 6. Implement Well-Formedness Checking for Recursive Protocols
- **Short:** Implement validation to ensure recursive protocols are well-formed.
    - **Implementation Prompt:**  
      Create a function to validate that all `Var` references point to defined `Rec` labels and that recursion is properly nested.
    - **Documentation Prompt:**  
      Document the well-formedness conditions for recursive protocols and how the validation ensures these conditions.
    - **Pre-conditions:** `GlobalInteraction` extended with recursion variants
    - **Post-conditions:** Well-formedness checking for recursive protocols implemented
    - **Test:** Tests for valid and invalid recursive protocols

```rust
// Example well-formedness checking
impl GlobalInteraction {
    /// Check if this protocol is well-formed with respect to recursion
    pub fn check_recursion_well_formedness(&self) -> Result<(), String> {
        let mut defined_labels = HashSet::new();
        self.check_recursion_well_formedness_inner(&mut defined_labels)
    }
    
    fn check_recursion_well_formedness_inner(
        &self, 
        defined_labels: &mut HashSet<RecursionLabel>
    ) -> Result<(), String> {
        match self {
            // Check each variant...
            GlobalInteraction::Rec { label, body } => {
                if defined_labels.contains(label) {
                    return Err(format!("Duplicate recursion label: {:?}", label));
                }
                defined_labels.insert(label.clone());
                let result = body.check_recursion_well_formedness_inner(defined_labels);
                defined_labels.remove(label);
                result
            },
            
            GlobalInteraction::Var { label } => {
                if !defined_labels.contains(label) {
                    return Err(format!("Reference to undefined recursion label: {:?}", label));
                }
                Ok(())
            },
            
            // Handle other variants...
            _ => Ok(()),
        }
    }
}
```

---

### 7. Create Builder Methods for Recursive Protocols
- **Short:** Add builder methods for creating recursive protocols.
    - **Implementation Prompt:**  
      Extend any existing builder APIs to support creating `Rec` and `Var` nodes in protocols.
    - **Documentation Prompt:**  
      Document the new builder methods with examples of creating recursive protocols.
    - **Pre-conditions:** `GlobalInteraction` extended with recursion variants
    - **Post-conditions:** Builder methods for recursive protocols implemented
    - **Test:** Tests using the builder methods to create recursive protocols

```rust
// Example builder methods
impl GlobalProtocolBuilder {
    /// Create a recursive protocol
    pub fn rec<F>(&mut self, label: impl Into<String>, body_fn: F) -> &mut Self 
    where 
        F: FnOnce(&mut Self) -> &mut Self 
    {
        let label = RecursionLabel::new(label);
        let var_builder = GlobalProtocolBuilder::new();
        let body_builder = body_fn(var_builder);
        let body = body_builder.build();
        
        self.add_interaction(GlobalInteraction::Rec {
            label,
            body: Box::new(body),
        })
    }
    
    /// Reference a recursion point
    pub fn var(&mut self, label: impl Into<String>) -> &mut Self {
        let label = RecursionLabel::new(label);
        self.add_interaction(GlobalInteraction::Var { label })
    }
}
```

---

### 8. Basic Testing for Recursive Protocols
- **Short:** Create tests for simple recursive protocols.
    - **Implementation Prompt:**  
      Write tests for creating, validating, and projecting simple recursive protocols (e.g., a recursive ping-pong).
    - **Documentation Prompt:**  
      Document the test cases, explaining what aspects of recursion they verify.
    - **Pre-conditions:** Recursion implementation complete
    - **Post-conditions:** Basic tests for recursive protocols pass
    - **Test:** Tests for simple recursive protocols

```rust
#[test]
fn test_simple_recursive_protocol() {
    // Create a recursive ping-pong protocol
    let protocol = GlobalProtocolBuilder::new()
        .rec("loop", |b| b
            .message(Client, Server, "Ping")
            .message(Server, Client, "Pong")
            .var("loop")
        )
        .build();
    
    // Validate well-formedness
    assert!(protocol.check_recursion_well_formedness().is_ok());
    
    // Project to client and server
    let client_protocol = protocol.clone().project::<Client, ()>();
    let server_protocol = protocol.project::<Server, ()>();
    
    // Verify the structure of the projected protocols
    // (implementation-specific assertions)
}
```

---

### 9. Testing Recursion with Choice
- **Short:** Create tests for recursive protocols with choice.
    - **Implementation Prompt:**  
      Write tests for creating, validating, and projecting recursive protocols that include choice (e.g., a ping-pong with an option to stop).
    - **Documentation Prompt:**  
      Document the test cases, explaining how recursion interacts with choice.
    - **Pre-conditions:** Recursion and choice implementations complete
    - **Post-conditions:** Tests for recursive protocols with choice pass
    - **Test:** Tests for recursive protocols with choice

```rust
#[test]
fn test_recursive_protocol_with_choice() {
    // Create a recursive ping-pong protocol with a choice to stop
    let protocol = GlobalProtocolBuilder::new()
        .rec("loop", |b| b
            .message(Client, Server, "Ping")
            .message(Server, Client, "Pong")
            .choice(Client, |c| c
                .branch("continue", |b| b
                    .message(Client, Server, "Continue")
                    .var("loop")
                )
                .branch("stop", |b| b
                    .message(Client, Server, "Stop")
                    .end()
                )
            )
        )
        .build();
    
    // Validate well-formedness
    assert!(protocol.check_recursion_well_formedness().is_ok());
    
    // Project to client and server
    let client_protocol = protocol.clone().project::<Client, ()>();
    let server_protocol = protocol.project::<Server, ()>();
    
    // Verify the structure of the projected protocols
    // (implementation-specific assertions)
}
```

---

### 10. Integration Testing for Recursive Multiparty Protocols
- **Short:** Create integration tests for recursive multiparty protocols.
    - **Implementation Prompt:**  
      Write tests that create, validate, project, and execute recursive multiparty protocols with 3+ participants.
    - **Documentation Prompt:**  
      Document the test cases, explaining how recursion works in a multiparty context.
    - **Pre-conditions:** All previous tasks complete
    - **Post-conditions:** Integration tests for recursive multiparty protocols pass
    - **Test:** Integration tests for recursive multiparty protocols

```rust
#[test]
fn test_recursive_multiparty_protocol() {
    // Create a recursive protocol with 3 participants
    let protocol = GlobalProtocolBuilder::new()
        .rec("loop", |b| b
            .message(Client, Server, "Request")
            .message(Server, Storage, "FetchData")
            .message(Storage, Server, "Data")
            .message(Server, Client, "Response")
            .choice(Client, |c| c
                .branch("continue", |b| b
                    .message(Client, Server, "Continue")
                    .var("loop")
                )
                .branch("stop", |b| b
                    .message(Client, Server, "Stop")
                    .end()
                )
            )
        )
        .build();
    
    // Validate well-formedness
    assert!(protocol.check_recursion_well_formedness().is_ok());
    
    // Project to all roles
    let client_protocol = protocol.clone().project::<Client, ()>();
    let server_protocol = protocol.clone().project::<Server, ()>();
    let storage_protocol = protocol.project::<Storage, ()>();
    
    // Verify the structure of the projected protocols
    // (implementation-specific assertions)
    
    // Execute the protocol (if runtime is available)
    // ...
}
```

---

### 11. Documentation and Examples
- **Short:** Document the recursion features and provide examples.
    - **Implementation Prompt:**  
      Add comprehensive documentation for the recursion features, including examples of common recursive patterns.
    - **Documentation Prompt:**  
      Create documentation that explains recursion in session types, how it's implemented in the library, and how to use it effectively.
    - **Pre-conditions:** All implementation tasks complete
    - **Post-conditions:** Documentation for recursion features complete
    - **Test:** Documentation examples compile and run correctly

```rust
/// # Recursion in Multiparty Session Types
///
/// This module provides support for recursive protocols using the `Rec` and `Var` constructs.
///
/// ## Example: Simple Recursive Protocol
///
/// ```rust
/// let ping_pong = GlobalProtocolBuilder::new()
///     .rec("loop", |b| b
///         .message(Client, Server, "Ping")
///         .message(Server, Client, "Pong")
///         .var("loop")
///     )
///     .build();
/// ```
///
/// ## Example: Recursive Protocol with Choice
///
/// ```rust
/// let ping_pong_with_stop = GlobalProtocolBuilder::new()
///     .rec("loop", |b| b
///         .message(Client, Server, "Ping")
///         .message(Server, Client, "Pong")
///         .choice(Client, |c| c
///             .branch("continue", |b| b
///                 .message(Client, Server, "Continue")
///                 .var("loop")
///             )
///             .branch("stop", |b| b
///                 .message(Client, Server, "Stop")
///                 .end()
///             )
///         )
///     )
///     .build();
/// ```
```

---

## Summary Table

| Task Group | Actions |
|------------|---------|
| Project Preparation | Ensure Stage 4 complete, dependencies |
| Recursion Label Type | Define `RecursionLabel` type |
| Global Protocol Extension | Add `Rec` and `Var` variants to `GlobalInteraction` |
| Local Protocol Extension | Add `Rec` and `Var` variants to `LocalProtocol` |
| Projection Implementation | Extend `Project` trait for recursion |
| Well-Formedness Checking | Implement validation for recursive protocols |
| Builder Methods | Add builder methods for recursive protocols |
| Basic Testing | Tests for simple recursive protocols |
| Choice Testing | Tests for recursive protocols with choice |
| Integration Testing | Tests for recursive multiparty protocols |
| Documentation | Document recursion features with examples |

---

## Implementation Considerations

1. **Label Management**: Ensure recursion labels are unique within their scope to avoid ambiguity.

2. **Projection Correctness**: The projection of recursive protocols must preserve the semantics of the global protocol for each role.

3. **Type Safety**: Maintain type safety throughout the recursion implementation, especially when unfolding recursive protocols.

4. **Composition**: Ensure recursion composes correctly with other protocol constructs (message passing, choice).

5. **Performance**: Consider the performance implications of recursive protocols, especially for deeply nested recursion.

6. **Error Handling**: Provide clear error messages for malformed recursive protocols.

7. **API Ergonomics**: Design the API for creating and working with recursive protocols to be intuitive and user-friendly.

---

**This implementation plan provides a structured approach to adding recursion support to the multiparty session type system, completing the core type system needed for expressing realistic protocols.**

---

## Implementation Status and Improvements

### 1. Optimization for Uninvolved Roles in Recursive Protocols

One significant optimization implemented in the projection mechanism is the ability to detect and optimize away recursion for roles that are not involved in the recursive part of a protocol. This optimization works as follows:

- When projecting a recursive protocol (`Rec` node), the system first projects the body of the recursion
- It then checks if the projected body contains any meaningful interactions (Send, Receive, Select, or Offer) for the target role
- If the role is not involved in any meaningful interactions within the recursive part, the recursion is "pruned" and replaced with an `End` node
- This optimization prevents uninvolved roles from having to process recursive structures they don't participate in

This optimization is particularly valuable in multiparty protocols where some roles may only be involved in specific parts of the protocol. For example, in a protocol where a Logger role only participates in the initial setup but not in the main recursive interaction between Client and Server, the Logger's projected protocol will be simplified by removing the unnecessary recursion.

```rust
// Example of how the optimization works
fn contains_meaningful_interactions<R: Role, M: Clone>(protocol: &LocalProtocol<R, M>) -> bool {
    match protocol {
        LocalProtocol::Send { .. } | LocalProtocol::Receive { .. } 
        | LocalProtocol::Select { .. } | LocalProtocol::Offer { .. } => true,
        
        LocalProtocol::Rec { body, .. } => contains_meaningful_interactions(body),
        
        LocalProtocol::Var { .. } | LocalProtocol::End { .. } => false,
    }
}

// In the projection implementation for Rec:
if contains_meaningful_interactions(&projected_body) {
    // If the role is involved, preserve the recursion
    LocalProtocol::Rec { label, body: Box::new(projected_body), ... }
} else {
    // If the role is not involved, prune the recursion
    LocalProtocol::End { ... }
}
```

This optimization has been thoroughly tested with both completely uninvolved roles and partially involved roles (roles that participate in some interactions outside the recursion but not within it).

### 2. Integration Testing for Multiparty Recursive Protocols

A comprehensive integration test suite has been implemented to verify the correct behavior of multiparty recursive protocols. The tests cover:

- **Projection Correctness**: Verifying that global protocols are correctly projected to local protocols for each role, preserving the recursive structure where appropriate
- **Protocol Execution**: Testing the actual execution of recursive protocols with multiple roles communicating through channels
- **Multiple Iterations**: Ensuring that recursive protocols can execute for multiple iterations before terminating

The integration tests use a three-role protocol (Client, Server, and Logger) with the following structure:

```
rec loop {
  Client -> Server: Request;
  Server -> Logger: LogEntry;
  Logger -> Server: Confirmation;
  Server -> Client: Response;
  choice at Client {
    Client -> Server: Continue;
    continue loop;
  } or {
    Client -> Server: Stop;
    end;
  }
}
```

The tests verify both the static structure of the projected protocols and their dynamic behavior during execution. The execution test simulates two iterations of the protocol, with the Client choosing to continue after the first iteration and to stop after the second iteration.

This integration testing approach ensures that the recursion implementation works correctly in realistic multiparty scenarios, including the interaction between recursion and choice constructs.

### 3. Other Improvements and Fixes

In addition to the major features described above, several other improvements have been made to the recursion implementation:

- **Well-formedness Checking**: Enhanced validation to ensure that all `Var` references point to defined `Rec` labels and that recursion is properly nested
- **Type Safety**: Improved type safety throughout the recursion implementation, especially when unfolding recursive protocols
- **API Ergonomics**: Refined the API for creating and working with recursive protocols to be more intuitive and user-friendly
- **Documentation**: Added comprehensive documentation for the recursion features, including examples of common recursive patterns

These improvements collectively ensure that the recursion implementation is robust, efficient, and easy to use, completing the core type system needed for expressing realistic protocols with loops or repeated interactions.