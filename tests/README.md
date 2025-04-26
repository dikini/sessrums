# Session Types Library (sez) Tests

This directory contains tests for the sez library, organized into different categories.

## Test Structure

- **integration/**: Integration tests that demonstrate working protocol examples
  - `mod.rs`: Common test infrastructure, helper functions, and utilities
  - `protocol_1.rs`: Simple Send/Recv Ping-Pong protocol
  - `protocol_2.rs`: Request/Response protocol (to be implemented)
  - `protocol_3.rs`: Simple Choice protocol (Send u64 or Recv f32) (to be implemented)
  - `protocol_4.rs`: Simple Authentication protocol (to be implemented)
  - `protocol_5.rs`: Data Query with Options protocol (to be implemented)

- **compile_fail/**: Tests that are expected to fail compilation
  - `error_1.rs`: Deadlock (Recv/Recv) example
  - `error_2.rs`: Deadlock (Send/Send) example (to be implemented)
  - `error_3.rs`: Type Mismatch example (to be implemented)
  - `error_4.rs`: Unexpected End example (to be implemented)

## Test Infrastructure

The integration test infrastructure is designed to make it easy to write tests that verify both the type-level properties and runtime behavior of protocols. It provides:

1. **Helper Functions**: Located in `tests/integration/mod.rs`, these functions help verify protocol properties:
   - `assert_protocol<P>()`: Verifies that a type implements the Protocol trait
   - `assert_dual<P, Q>()`: Verifies that two types have the correct duality relationship
   - `assert_self_dual<P>()`: Verifies that a type is its own dual
   - `mock_channel<P, IO>()`: Creates a channel with a specific protocol and IO type for testing

2. **Test Runners**: Each protocol has its own test runner file in the `tests` directory:
   - `protocol_1_test.rs`: Runs the tests for Protocol 1
   - Additional test runners will be added for future protocols

## Running Tests

### Integration Tests

To run the integration tests:

```bash
cargo test --test 'protocol_*'
```

To run a specific protocol test:

```bash
cargo test --test protocol_1_test
```

### Compile-Fail Tests

The compile-fail tests use the `trybuild` crate to verify that certain code patterns fail to compile with the expected error messages. To run these tests:

```bash
cargo test --test compile_fail
```

## Test Purpose

These tests serve multiple purposes:

1. **Verification**: Ensure that the session type system correctly enforces protocol adherence at compile time
2. **Documentation**: Provide concrete examples of how to use the library
3. **Regression Testing**: Prevent regressions in the type system

The integration tests demonstrate correct usage of the session type system, while the compile-fail tests verify that the type system correctly rejects invalid protocols.