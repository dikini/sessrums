# Stage 3 Review and Task Breakdown

## Context

Stage 3 extends the session type system to support **multiparty session types (MPST)**, enabling the specification and enforcement of communication protocols involving more than two participants. This is a significant advancement over the binary session types from Stages 1 and 2, as it allows modeling complex distributed systems with multiple interacting components.

Multiparty session types provide a formal framework for specifying and verifying the interactions between multiple participants in a distributed system. They ensure that all participants follow their prescribed roles in the protocol, preventing communication errors, deadlocks, and other concurrency issues.

The key innovation in multiparty session types is the distinction between:
1. **Global protocols** - describing the overall communication flow from a centralized perspective
2. **Local protocols** - describing the communication from each participant's individual perspective

Global protocols are projected to local protocols for each participant, ensuring that the local views are consistent with the global specification.

---

## Review of Stage 3

**Strengths:**
- Enables modeling of complex distributed systems with multiple participants
- Maintains typestate guarantees and transport abstraction from previous stages
- Provides a clear separation between global and local protocol views
- Supports protocol projection from global to local views
- Includes verification mechanisms for projection correctness
- Implements a flexible multiparty transport abstraction for message routing

**Risks/Considerations:**
- Correct projection from global to local protocols is complex and error-prone
- The API must make multiparty protocols ergonomic and type-safe
- Ensuring deadlock freedom in multiparty protocols requires careful design
- Multiparty protocols must compose correctly with existing binary protocols
- Runtime message routing adds complexity compared to binary channels

---

## Stage 3: Actionable Task Breakdown

### 1. Project Preparation
- **Short:** Ensure Stage 2 is complete and all tests pass.
    - **Implementation Prompt:**  
      Run all Stage 2 tests and resolve any failures. Ensure the codebase is clean and ready for extension.
    - **Documentation Prompt:**  
      Document in the changelog or project notes that Stage 2 is a prerequisite for Stage 3.
    - **Pre-conditions:** Stage 2 implementation complete and tests passing
    - **Post-conditions:** Project ready for Stage 3 implementation
    - **Tests:** All Stage 2 tests pass

---

### 2. Common Multiparty Structures
- **Short:** Define common structures for multiparty session types:
    - `RoleIdentifier` for runtime role identification
    - `ProtocolState` trait for protocol state types
    - **Implementation Prompt:**  
      Implement the `RoleIdentifier` struct and `ProtocolState` trait to provide the foundation for multiparty session types.
    - **Documentation Prompt:**  
      Document the purpose of each structure and how they relate to the multiparty session type system.
    - **Pre-conditions:** None
    - **Post-conditions:** Common structures defined and documented
    - **Tests:** Unit tests for `RoleIdentifier` creation and comparison

---

### 3. Global Protocol Representation
- **Short:** In `src/session_types/global.rs`, define:
    - `GlobalInteraction<M>` enum for representing global protocols
    - Methods for constructing global protocols
    - **Implementation Prompt:**  
      Implement the `GlobalInteraction` enum with variants for message exchange and protocol termination. Include methods for constructing global protocols in a fluent style.
    - **Documentation Prompt:**  
      Document the global protocol representation, including examples of how to define protocols.
    - **Pre-conditions:** Common structures defined
    - **Post-conditions:** Global protocol representation implemented
    - **Tests:** Tests for creating and validating global protocols

---

### 4. Local Protocol Representation
- **Short:** In `src/session_types/local.rs`, define:
    - `LocalProtocol<R, M>` enum for representing local protocols
    - Methods for constructing local protocols
    - **Implementation Prompt:**  
      Implement the `LocalProtocol` enum with variants for sending, receiving, and termination. Include methods for constructing local protocols in a fluent style.
    - **Documentation Prompt:**  
      Document the local protocol representation, including examples of how to define protocols for specific roles.
    - **Pre-conditions:** Common structures and global protocol representation defined
    - **Post-conditions:** Local protocol representation implemented
    - **Tests:** Tests for creating and validating local protocols

---

### 5. Multiparty Transport Abstraction
- **Short:** In `src/transport.rs`, define:
    - `MultipartyTransport` trait for multiparty communication
    - `MockMultipartyBroker` for testing
    - `ParticipantChannel` for role-specific communication
    - **Implementation Prompt:**  
      Implement the multiparty transport abstraction, including a broker for message routing and channels for participant communication.
    - **Documentation Prompt:**  
      Document the multiparty transport abstraction, including examples of how to use it for protocol execution.
    - **Pre-conditions:** Common structures defined
    - **Post-conditions:** Multiparty transport abstraction implemented
    - **Tests:** Tests for message routing and error handling

---

### 6. Manual Protocol Definition and Projection
- **Short:** In `sessrums-examples/src/examples/manual_projection.rs`, implement:
    - Manual definition of a global protocol
    - Manual projection to local protocols
    - Verification of projection correctness
    - **Implementation Prompt:**  
      Implement an example demonstrating manual protocol definition and projection, including verification functions.
    - **Documentation Prompt:**  
      Document the manual projection process, including examples and verification techniques.
    - **Pre-conditions:** Global and local protocol representations defined
    - **Post-conditions:** Manual projection example implemented
    - **Tests:** Tests for projection correctness

---

### 7. Multiparty Session Execution
- **Short:** In `src/session_types/multiparty_session.rs`, implement:
    - `MultipartySession<R, P, T>` struct for executing local protocols
    - Methods for sending and receiving messages
    - **Implementation Prompt:**  
      Implement the `MultipartySession` struct with methods for executing protocol actions, ensuring type safety through the typestate pattern.
    - **Documentation Prompt:**  
      Document the multiparty session execution, including examples of how to execute protocols.
    - **Pre-conditions:** Local protocol representation and multiparty transport defined
    - **Post-conditions:** Multiparty session execution implemented
    - **Tests:** Tests for protocol execution and error handling

---

### 8. Concurrent Protocol Execution
- **Short:** In `sessrums-types/tests/multiparty_basic.rs`, implement:
    - Concurrent execution of a multiparty protocol
    - Verification of correct message exchange
    - **Implementation Prompt:**  
      Implement tests demonstrating concurrent execution of multiparty protocols, including verification of correct message exchange.
    - **Documentation Prompt:**  
      Document the concurrent execution of multiparty protocols, including examples and verification techniques.
    - **Pre-conditions:** Multiparty session execution implemented
    - **Post-conditions:** Concurrent protocol execution demonstrated
    - **Tests:** Tests for concurrent protocol execution

---

### 9. Documentation and Examples
- **Short:** Document each new struct, enum, trait, and method with doc comments.
    - **Implementation Prompt:**  
      For every new public item, add a Rust doc comment (`/// ...`) describing its purpose, usage, and any important details.
    - **Documentation Prompt:**  
      Ensure all doc comments are clear, concise, and include usage examples where appropriate.

- **Short:** Add comprehensive examples demonstrating multiparty protocols.
    - **Implementation Prompt:**  
      Write examples demonstrating multiparty protocols, including global definition, projection, and execution.
    - **Documentation Prompt:**  
      Ensure the examples are copy-pastable and highlight the core API and typestate transitions for multiparty protocols.
    - **Pre-conditions:** All implementation tasks complete
    - **Post-conditions:** Comprehensive documentation and examples provided
    - **Tests:** Examples compile and run correctly

---

## Summary Table

| Task Group                        | Actions                                                                                 |
|-----------------------------------|----------------------------------------------------------------------------------------|
| Project Preparation               | Ensure Stage 2 complete, dependencies                                                  |
| Common Multiparty Structures      | `RoleIdentifier`, `ProtocolState` trait                                                |
| Global Protocol Representation    | `GlobalInteraction` enum, construction methods                                         |
| Local Protocol Representation     | `LocalProtocol` enum, construction methods                                             |
| Multiparty Transport Abstraction  | `MultipartyTransport` trait, `MockMultipartyBroker`, `ParticipantChannel`              |
| Manual Protocol Definition        | Global protocol definition, manual projection, verification                            |
| Multiparty Session Execution      | `MultipartySession` struct, protocol execution methods                                 |
| Concurrent Protocol Execution     | Concurrent execution tests, message exchange verification                              |
| Documentation/Examples            | Doc comments, comprehensive examples                                                   |

---

## Implementation Considerations

### Global vs. Local Protocols

The distinction between global and local protocols is fundamental to multiparty session types:

1. **Global Protocols**:
   - Provide a centralized view of the entire protocol
   - Describe all interactions between all participants
   - Serve as the source of truth for protocol specification
   - Enable verification of protocol properties (e.g., deadlock freedom)

2. **Local Protocols**:
   - Provide a participant-specific view of the protocol
   - Describe only the interactions involving a specific participant
   - Derived from global protocols through projection
   - Used for implementing participant behavior

### Projection

Projection is the process of deriving local protocols from a global protocol. It ensures that each participant's local view is consistent with the global specification. Key considerations for projection include:

1. **Causality Preservation**: Ensuring that the causal dependencies between messages in the global protocol are preserved in the local protocols.
2. **Coherence**: Ensuring that the local protocols are coherent with each other, i.e., they can be composed to form a valid global protocol.
3. **Completeness**: Ensuring that all relevant interactions from the global protocol are included in each local protocol.

### Multiparty Transport

The multiparty transport abstraction provides a flexible mechanism for routing messages between participants. Key components include:

1. **Broker**: A centralized component for message routing
2. **Participant Channels**: Role-specific channels for sending and receiving messages
3. **Message Envelopes**: Containers for messages with routing information

### Typestate Pattern

The typestate pattern is used to ensure protocol adherence at compile time. Key aspects include:

1. **Protocol State Types**: Types representing the current state of the protocol
2. **Session Type Evolution**: The type of the session evolves as protocol actions are executed
3. **Type-Level Guarantees**: The compiler enforces that protocol actions are executed in the correct order

### Best Practices

1. **Define Global Protocols First**: Start by defining the global protocol, then project to local protocols.
2. **Verify Projection Correctness**: Ensure that the projection preserves causality and coherence.
3. **Use Type Parameters for Message Types**: Parameterize protocols with message types for flexibility.
4. **Handle Errors Gracefully**: Implement robust error handling for protocol violations.
5. **Test Concurrent Execution**: Verify that protocols execute correctly in concurrent environments.

---

**Each of these tasks is self-contained and can be implemented and tested independently, ensuring a solid, verifiable foundation for the more advanced stages of your MPST project.**