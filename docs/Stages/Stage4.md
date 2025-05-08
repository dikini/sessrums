# Stage 4 Review and Task Breakdown

## Context

Stage 4 builds on the multiparty primitives established in Stage 3 by implementing **automated projection** from global protocols to local protocols. Projection is a fundamental operation in multiparty session types that transforms a global protocol description (involving all participants) into role-specific local protocols (describing the behavior of individual participants). This stage focuses on implementing projection for the core protocol constructs: `Message`, `End`, and `Choice`.

---

## Review of Stage 4

**Strengths:**
- Enables automatic derivation of role-specific protocols from a single global definition
- Ensures consistency between global and local protocol views
- Handles the complexity of projecting choice constructs with different behaviors for the deciding role vs. other roles
- Provides a foundation for the full MPST system

**Risks/Considerations:**
- Correct handling of uninvolved roles is subtle and error-prone
- The projection algorithm must be sound (preserve protocol semantics)
- Choice projection requires special attention to ensure the deciding role gets a `Select` while others get an `Offer`
- Tests must verify that projected protocols are correct and can be executed

---

## Stage 4: Actionable Task Breakdown

### 1. Project Preparation
- **Short:** Ensure Stage 3 is complete and all tests pass.
    - **Implementation Prompt:**  
      Run all Stage 3 tests and resolve any failures. Ensure the codebase is clean and ready for extension.
    - **Documentation Prompt:**  
      Document in the changelog or project notes that Stage 3 is a prerequisite for Stage 4.

---

### 2. Define the Project Trait
- **Short:** In `src/projection.rs`, define the `Project` trait for transforming global protocols to local protocols.
    - **Implementation Prompt:**  
      Create a new file `src/projection.rs` and define the `Project<R: Role>` trait with an associated type `Output` and a `project` method that transforms a global protocol into a local protocol for role `R`.
    - **Documentation Prompt:**  
      Add doc comments explaining the purpose of the `Project` trait, its type parameters, and how it's used in the MPST system.

```rust
/// Trait for projecting a global protocol to a role-specific local protocol.
pub trait Project<R: Role> {
    /// The resulting local protocol type after projection
    type Output;
    
    /// Project this global protocol to a local protocol for role R
    fn project(self) -> Self::Output;
}
```

---

### 3. Extend GlobalInteraction with Choice
- **Short:** In `src/session_types/global.rs`, extend the `GlobalInteraction` enum with a `Choice` variant.
    - **Implementation Prompt:**  
      Add a `Choice` variant to the `GlobalInteraction` enum that represents a choice made by a specific role between multiple protocol continuations. The variant should include the deciding role, a vector of labeled branches, and any other necessary metadata.
    - **Documentation Prompt:**  
      Document the `Choice` variant, explaining its purpose, parameters, and how it represents branching in global protocols.

```rust
pub enum GlobalInteraction {
    // Existing variants...
    
    /// A choice point in the protocol, where the deciding role selects one of the branches
    Choice {
        /// The role that makes the decision
        decider: RoleIdentifier,
        /// The available branches, each with a label and continuation
        branches: Vec<(Label, Box<GlobalInteraction>)>,
    },
    
    // ...
}
```

---

### 4. Extend LocalProtocol with Select and Offer
- **Short:** In `src/session_types/local.rs`, extend the `LocalProtocol` enum with `Select` and `Offer` variants.
    - **Implementation Prompt:**  
      Add `Select` and `Offer` variants to the `LocalProtocol` enum. `Select` represents making a choice (used by the deciding role), while `Offer` represents offering choices (used by other roles). Both should include the necessary metadata (decider role, branches, etc.).
    - **Documentation Prompt:**  
      Document both variants, explaining their purpose, parameters, and how they represent choice in local protocols.

```rust
pub enum LocalProtocol {
    // Existing variants...
    
    /// Select one of multiple branches (used by the deciding role)
    Select {
        /// The available branches, each with a label and continuation
        branches: Vec<(Label, Box<LocalProtocol>)>,
    },
    
    /// Offer multiple branches for selection by another role
    Offer {
        /// The role that makes the decision
        decider: RoleIdentifier,
        /// The available branches, each with a label and continuation
        branches: Vec<(Label, Box<LocalProtocol>)>,
    },
    
    // ...
}
```

---

### 5. Implement Basic Projection (Message, End)
- **Short:** Implement the `Project` trait for `GlobalInteraction` for the basic cases: `Message` and `End`.
    - **Implementation Prompt:**  
      Implement the `Project<R>` trait for `GlobalInteraction` handling the `Message` and `End` cases. For `Message`, the projection should:
      - If `R` is the sender, return a `LocalProtocol::Send`
      - If `R` is the receiver, return a `LocalProtocol::Receive`
      - If `R` is not involved, return the projection of the continuation
      For `End`, simply return `LocalProtocol::End`.
    - **Documentation Prompt:**  
      Document the implementation, explaining the projection rules for each case and how uninvolved roles are handled.

```rust
impl<R: Role> Project<R> for GlobalInteraction {
    type Output = LocalProtocol;
    
    fn project(self) -> Self::Output {
        match self {
            GlobalInteraction::Message { from, to, msg_type, continuation } => {
                if R::is_role(&from) {
                    // Sender role gets a Send
                    LocalProtocol::Send {
                        to: to.clone(),
                        msg_type: msg_type.clone(),
                        continuation: Box::new(continuation.project()),
                    }
                } else if R::is_role(&to) {
                    // Receiver role gets a Receive
                    LocalProtocol::Receive {
                        from: from.clone(),
                        msg_type: msg_type.clone(),
                        continuation: Box::new(continuation.project()),
                    }
                } else {
                    // Uninvolved role skips this interaction
                    continuation.project()
                }
            },
            GlobalInteraction::End => LocalProtocol::End,
            // Other cases will be handled in subsequent tasks
            _ => panic!("Projection not yet implemented for this case"),
        }
    }
}
```

---

### 6. Implement Choice Projection
- **Short:** Extend the `Project` implementation to handle the `Choice` case.
    - **Implementation Prompt:**  
      Implement projection for the `Choice` variant. The projection should:
      - If `R` is the deciding role, return a `LocalProtocol::Select` with projected branches
      - If `R` is not the deciding role, return a `LocalProtocol::Offer` with projected branches
      - Ensure that branch labels are preserved during projection
    - **Documentation Prompt:**  
      Document the choice projection implementation, explaining how the deciding role gets a `Select` while other roles get an `Offer`, and how branch consistency is maintained.

```rust
// Inside the match statement in the Project implementation
GlobalInteraction::Choice { decider, branches } => {
    if R::is_role(&decider) {
        // Deciding role gets a Select
        LocalProtocol::Select {
            branches: branches
                .into_iter()
                .map(|(label, branch)| (label, Box::new(branch.project())))
                .collect(),
        }
    } else {
        // Other roles get an Offer
        LocalProtocol::Offer {
            decider: decider.clone(),
            branches: branches
                .into_iter()
                .map(|(label, branch)| (label, Box::new(branch.project())))
                .collect(),
        }
    }
}
```

---

### 7. Implement Helper Functions for Projection
- **Short:** Create helper functions to simplify projection usage.
    - **Implementation Prompt:**  
      Implement helper functions like `project_for_role<R: Role>(global: GlobalInteraction) -> LocalProtocol` to make projection more ergonomic for users.
    - **Documentation Prompt:**  
      Document these helper functions, providing usage examples and explaining their benefits.

```rust
/// Project a global protocol for a specific role
pub fn project_for_role<R: Role>(global: GlobalInteraction) -> LocalProtocol {
    global.project::<R>()
}

/// Project a global protocol for all roles in a set
pub fn project_for_all_roles(global: GlobalInteraction, roles: &[RoleIdentifier]) 
    -> HashMap<RoleIdentifier, LocalProtocol> 
{
    roles
        .iter()
        .map(|role| (role.clone(), project_for_role::<R>(global.clone())))
        .collect()
}
```

---

### 8. Testing Basic Projection
- **Short:** Create `tests/projection_basic.rs` to test basic projection cases.
    - **Implementation Prompt:**  
      Write tests that verify the projection of simple global protocols (with `Message` and `End`) to local protocols for different roles. Verify that:
      - Sender roles get `Send` actions
      - Receiver roles get `Receive` actions
      - Uninvolved roles skip the interaction
      - `End` projects to `End` for all roles
    - **Documentation Prompt:**  
      Add doc comments to each test explaining what aspect of projection is being verified.

```rust
#[test]
fn test_message_projection() {
    // Create a simple global protocol: A -> B: Msg; End
    let global = GlobalInteraction::Message {
        from: "A".into(),
        to: "B".into(),
        msg_type: PhantomData::<TestMessage>,
        continuation: Box::new(GlobalInteraction::End),
    };
    
    // Project for role A (sender)
    let a_local = project_for_role::<RoleA>(global.clone());
    assert_matches!(a_local, 
        LocalProtocol::Send { to, msg_type, continuation } 
        if to == "B" && continuation.as_ref() == &LocalProtocol::End
    );
    
    // Project for role B (receiver)
    let b_local = project_for_role::<RoleB>(global.clone());
    assert_matches!(b_local, 
        LocalProtocol::Receive { from, msg_type, continuation } 
        if from == "A" && continuation.as_ref() == &LocalProtocol::End
    );
    
    // Project for role C (uninvolved)
    let c_local = project_for_role::<RoleC>(global.clone());
    assert_eq!(c_local, LocalProtocol::End);
}
```

---

### 9. Testing Choice Projection
- **Short:** Create `tests/projection_choice.rs` to test choice projection.
    - **Implementation Prompt:**  
      Write tests that verify the projection of global protocols with choices. Verify that:
      - The deciding role gets a `Select` with the correct branches
      - Other roles get an `Offer` with the correct branches
      - Branch labels are preserved during projection
      - Nested choices project correctly
    - **Documentation Prompt:**  
      Add doc comments to each test explaining what aspect of choice projection is being verified.

```rust
#[test]
fn test_choice_projection() {
    // Create a global protocol with choice: 
    // choice at A { A -> B: Msg1; End } or { A -> B: Msg2; End }
    let global = GlobalInteraction::Choice {
        decider: "A".into(),
        branches: vec![
            ("option1".into(), Box::new(GlobalInteraction::Message {
                from: "A".into(),
                to: "B".into(),
                msg_type: PhantomData::<Msg1>,
                continuation: Box::new(GlobalInteraction::End),
            })),
            ("option2".into(), Box::new(GlobalInteraction::Message {
                from: "A".into(),
                to: "B".into(),
                msg_type: PhantomData::<Msg2>,
                continuation: Box::new(GlobalInteraction::End),
            })),
        ],
    };
    
    // Project for role A (decider)
    let a_local = project_for_role::<RoleA>(global.clone());
    assert_matches!(a_local, LocalProtocol::Select { branches } => {
        assert_eq!(branches.len(), 2);
        // Verify branch contents...
    });
    
    // Project for role B (participant)
    let b_local = project_for_role::<RoleB>(global.clone());
    assert_matches!(b_local, LocalProtocol::Offer { decider, branches } => {
        assert_eq!(decider, "A");
        assert_eq!(branches.len(), 2);
        // Verify branch contents...
    });
    
    // Project for role C (uninvolved)
    let c_local = project_for_role::<RoleC>(global.clone());
    assert_matches!(c_local, LocalProtocol::Offer { decider, branches } => {
        assert_eq!(decider, "A");
        assert_eq!(branches.len(), 2);
        // Verify branch contents are just End for uninvolved role...
    });
}
```

---

### 10. Integration Testing
- **Short:** Create `tests/projection_integration.rs` to test projection in a complete workflow.
    - **Implementation Prompt:**  
      Write tests that verify the full projection workflow:
      1. Define a complex global protocol with messages, choices, and multiple roles
      2. Project it for each role
      3. Verify that the projected local protocols can be executed correctly
      4. Ensure that the execution respects the protocol semantics
    - **Documentation Prompt:**  
      Document the test scenario, explaining how it demonstrates the correctness of the projection mechanism in a realistic setting.

```rust
#[test]
fn test_projection_execution() {
    // Define a global protocol: 
    // A -> B: Request;
    // choice at B {
    //   B -> A: Accept; B -> C: Notify; End
    // } or {
    //   B -> A: Reject; End
    // }
    let global = /* ... */;
    
    // Project for all roles
    let a_local = project_for_role::<RoleA>(global.clone());
    let b_local = project_for_role::<RoleB>(global.clone());
    let c_local = project_for_role::<RoleC>(global.clone());
    
    // Set up mock channels for all role pairs
    let (a_to_b, b_from_a) = new_mock_channel_pair();
    let (b_to_a, a_from_b) = new_mock_channel_pair();
    let (b_to_c, c_from_b) = new_mock_channel_pair();
    
    // Execute the protocol concurrently
    let a_handle = std::thread::spawn(move || {
        // A sends request, then receives either Accept or Reject
        // ...
    });
    
    let b_handle = std::thread::spawn(move || {
        // B receives request, then chooses Accept or Reject
        // If Accept, also sends Notify to C
        // ...
    });
    
    let c_handle = std::thread::spawn(move || {
        // C may receive Notify from B
        // ...
    });
    
    // Join threads and verify results
    // ...
}
```

---

### 11. Documentation and Examples
- **Short:** Document the projection mechanism and provide examples.
    - **Implementation Prompt:**  
      Update the crate documentation to explain the projection mechanism, its importance, and how to use it. Include examples of defining global protocols and projecting them to local protocols.
    - **Documentation Prompt:**  
      Create comprehensive documentation that covers:
      - The concept of projection in MPST
      - The `Project` trait and its implementation
      - How to use the projection API
      - Examples of projection for different protocol constructs
      - Best practices and common pitfalls

---

## Summary Table

| Task Group                | Actions                                                                                 |
|---------------------------|----------------------------------------------------------------------------------------|
| Project Preparation       | Ensure Stage 3 complete, dependencies                                                  |
| Project Trait             | Define `Project<R: Role>` trait                                                        |
| GlobalInteraction         | Add `Choice` variant                                                                   |
| LocalProtocol             | Add `Select` and `Offer` variants                                                      |
| Basic Projection          | Implement projection for `Message` and `End`                                           |
| Choice Projection         | Implement projection for `Choice`                                                      |
| Helper Functions          | Create ergonomic projection helpers                                                    |
| Basic Testing             | Test projection of simple protocols                                                    |
| Choice Testing            | Test projection of protocols with choices                                              |
| Integration Testing       | Test end-to-end projection and execution                                               |
| Documentation/Examples    | Document projection mechanism with examples                                            |

---

**Each of these tasks is self-contained and can be implemented and tested independently, ensuring a solid, verifiable foundation for the more advanced stages of your MPST project.**