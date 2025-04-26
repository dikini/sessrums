# Session Types Library (sez) Tests

This directory contains tests for the sez library, organized into different categories.

## Test Structure

- **integration/**: Integration tests that demonstrate working protocol examples
  - `protocol_1.rs`: Simple Send/Recv Ping-Pong protocol
  - `protocol_2.rs`: Request/Response protocol
  - `protocol_3.rs`: Simple Choice protocol (Send u64 or Recv f32)
  - `protocol_4.rs`: Simple Authentication protocol
  - `protocol_5.rs`: Data Query with Options protocol

- **compile_fail/**: Tests that are expected to fail compilation
  - `error_1.rs`: Deadlock (Recv/Recv) example
  - `error_2.rs`: Deadlock (Send/Send) example
  - `error_3.rs`: Type Mismatch example
  - `error_4.rs`: Unexpected End example

## Running Tests

### Integration Tests

To run the integration tests:

```bash
cargo test --test 'protocol_*'
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