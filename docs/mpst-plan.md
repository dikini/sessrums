# Plan to Add Multiparty Session Types (MPST) to the sessrums Library

This document outlines a detailed plan to extend the `sessrums` library with support for Multiparty Session Types (MPST). The implementation will include global protocol definitions, roles, projection from global to local types, and a macro for defining global protocols using a syntax inspired by sequence diagrams.

Document your work for each task in work/action-log.md
After each task write your learnings and insights to work/insights.md
Verify each task completion check that the code builds and tests are successful.

---

## Phase 1: Research and Design

### Task 1.1: Understand MPST Concepts
- **Goal**: Gain a deep understanding of MPST, including roles, global protocols, projection, branching, recursion, and composition.
- **Code**: No code changes.
- **Documentation**: Summarize key MPST concepts in `docs/mpst-concepts.md`.
- **Tests**: None.
- **Acceptance Criteria**: Team members are familiar with MPST concepts and their application.

### Task 1.2: Define Requirements and Architecture
- **Goal**: Define the scope of MPST support and design the architecture for integration with the existing library.
- **Code**: Create a high-level design document in `docs/mpst-design.md`.
- **Documentation**: Include diagrams and examples of global protocols and their projections.
- **Tests**: None.
- **Acceptance Criteria**: Approved design document with clear requirements and architecture.

---

## Phase 2: Core MPST Components

### Task 2.1: Implement Roles
- **Goal**: Define a `Role` trait and concrete role types (e.g., `RoleA`, `RoleB`).
- **Code**: Add `src/proto/roles.rs` with the `Role` trait and role definitions.
- **Documentation**: Document the `Role` trait and its usage in `docs/session-types-documentation.md`.
- **Tests**: Write unit tests to verify role creation and usage.
- **Acceptance Criteria**: Code builds, tests pass, and roles are documented.

### Task 2.2: Define Global Protocol
- **Goal**: Create a `GlobalProtocol` type to represent the overall choreography of interactions.
- **Code**: Add `src/proto/global.rs` with the `GlobalProtocol` type and helper methods.
- **Documentation**: Document the `GlobalProtocol` type in `docs/session-types-documentation.md`.
- **Tests**: Write unit tests to verify the representation of global protocols.
- **Acceptance Criteria**: Code builds, tests pass, and global protocols are documented.

### Task 2.3: Implement Projection
- **Goal**: Implement a `project` function to extract local types from a global protocol.
- **Code**: Add `src/proto/projection.rs` with the `project` function.
- **Documentation**: Document the projection process in `docs/session-types-documentation.md`.
- **Tests**: Write tests to verify that projections produce correct local types.
- **Acceptance Criteria**: Code builds, tests pass, and projection is documented.

---

## Phase 3: Advanced Features

### Task 3.1: Add Branching and Choice
- **Goal**: Extend the global protocol to support branching (`⊕` for choice, `&` for offer).
- **Code**: Update `src/proto/global.rs` and `src/proto/projection.rs` to handle branching.
- **Documentation**: Document branching and choice in `docs/session-types-documentation.md`.
- **Tests**: Write tests for protocols with branching and ensure correctness.
- **Acceptance Criteria**: Code builds, tests pass, and branching is documented.

### Task 3.2: Add Recursion
- **Goal**: Add support for recursion (`μ X.`) in global protocols and projections.
- **Code**: Update `src/proto/global.rs` and `src/proto/projection.rs` to handle recursion.
- **Documentation**: Document recursion in `docs/session-types-documentation.md`.
- **Tests**: Write tests for recursive protocols.
- **Acceptance Criteria**: Code builds, tests pass, and recursion is documented.

### Task 3.3: Process Composition
- **Goal**: Implement sequential, parallel, and branching composition for global protocols.
- **Code**: Update `src/proto/global.rs` to support composition.
- **Documentation**: Document composition in `docs/session-types-documentation.md`.
- **Tests**: Write tests for composed protocols.
- **Acceptance Criteria**: Code builds, tests pass, and composition is documented.

---

## Phase 4: Macro for Global Protocols

### Task 4.1: Design Macro Syntax
- **Goal**: Define a syntax for the macro inspired by sequence diagrams.
- **Code**: Create a design document in `docs/mpst-macro.md`.
- **Documentation**: Include examples of the macro syntax.
- **Tests**: None.
- **Acceptance Criteria**: Approved macro design document.

### Task 4.2: Implement Macro
- **Goal**: Develop a macro to convert the defined syntax into a `GlobalProtocol`.
- **Code**: Add `src/proto/macro.rs` with the macro implementation.
- **Documentation**: Document the macro in `docs/session-types-documentation.md`.
- **Tests**: Write tests to verify the macro generates correct `GlobalProtocol` instances.
- **Acceptance Criteria**: Code builds, tests pass, and macro is documented.

---

## Phase 5: Integration and Testing

### Task 5.1: Update Channel Implementation
- **Goal**: Extend the `Chan` type to support MPST roles and local types.
- **Code**: Update `src/chan/mod.rs` to integrate MPST.
- **Documentation**: Update `docs/session-types-documentation.md` to reflect changes.
- **Tests**: Write integration tests to ensure MPST works with the existing channel implementation.
- **Acceptance Criteria**: Code builds, tests pass, and documentation is updated.

### Task 5.2: Backward Compatibility
- **Goal**: Ensure binary session types remain functional alongside MPST.
- **Code**: Add compatibility layers in `src/proto/compat.rs`.
- **Documentation**: Highlight compatibility in `docs/session-types-documentation.md`.
- **Tests**: Write tests to verify backward compatibility.
- **Acceptance Criteria**: Code builds, tests pass, and compatibility is documented.

---

## Phase 6: Examples and Documentation

### Task 6.1: Create Examples
- **Goal**: Demonstrate MPST features with practical examples.
- **Code**: Add examples in `examples/mpst/`.
- **Documentation**: Add detailed comments to examples.
- **Tests**: Ensure examples compile and run correctly.
- **Acceptance Criteria**: Examples are created, documented, and run successfully.

### Task 6.2: Comprehensive Documentation
- **Goal**: Update all documentation to include MPST features.
- **Code**: Update `README.md` and `docs/`.
- **Documentation**: Ensure all new features are documented.
- **Tests**: None.
- **Acceptance Criteria**: Documentation is complete and accurate.

---

## Phase 7: Final Testing and Release

### Task 7.1: Compile-Time Tests
- **Goal**: Verify that invalid MPST protocols fail to compile.
- **Code**: Add tests in `tests/compile_fail/`.
- **Documentation**: Document how to run compile-time tests.
- **Tests**: Write tests for common MPST errors.
- **Acceptance Criteria**: Compile-time tests pass.

### Task 7.2: Runtime Tests
- **Goal**: Verify the behavior of MPST protocols at runtime.
- **Code**: Add tests in `tests/runtime_tests.rs`.
- **Documentation**: Document how to interpret runtime test results.
- **Tests**: Write tests for runtime behavior.
- **Acceptance Criteria**: Runtime tests pass.

### Task 7.3: Final Integration Test
- **Goal**: Create a comprehensive test that uses all MPST features.
- **Code**: Add `tests/final_integration_test.rs`.
- **Documentation**: Document the integration test.
- **Tests**: Ensure the integration test passes.
- **Acceptance Criteria**: Integration test passes.

---

## Timeline
- **Phase 1**: 2 weeks
- **Phase 2**: 3 weeks
- **Phase 3**: 4 weeks
- **Phase 4**: 2 weeks
- **Phase 5**: 3 weeks
- **Phase 6**: 2 weeks
- **Phase 7**: 2 weeks

---

## Completion Criteria
- MPST roles, global protocols, and projections are implemented and tested.
- Documentation and examples are complete and accessible.
- The library supports both binary and multiparty session types seamlessly.