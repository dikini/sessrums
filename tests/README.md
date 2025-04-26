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

1. **Helper Functions**: Located in `tests/helpers.rs` and `tests/integration/mod.rs`, these functions help verify protocol properties:
   - `assert_protocol<P>()`: Verifies that a type implements the Protocol trait
   - `assert_dual<P, Q>()`: Verifies that two types have the correct duality relationship
   - `assert_self_dual<P>()`: Verifies that a type is its own dual
   - `mock_channel<P, IO>()`: Creates a channel with a specific protocol and IO type for testing
   - `verify_dual_protocols<P, Q, IO1, IO2>()`: Verifies that two channels have dual protocols

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

The compile-fail tests use the `trybuild` crate to verify that certain code patterns fail to compile with the expected error messages. These tests are crucial for ensuring that the session type system correctly rejects invalid protocols at compile time.

#### Running Compile-Fail Tests

To run the compile-fail tests:

```bash
cargo test --test compile_fail
```

#### Adding New Compile-Fail Tests

To add a new compile-fail test:

1. Create a new Rust file in the `tests/compile_fail/` directory (e.g., `tests/compile_fail/my_error.rs`)
2. Write code that should fail to compile, with detailed comments explaining why it should fail
3. Run the test with `cargo test --test compile_fail`
4. The first time you run the test, it will fail and generate a `.stderr` file in the `wip/` directory
5. Review the error messages in the `.stderr` file to ensure they match your expectations
6. Move the `.stderr` file from `wip/` to `tests/compile_fail/` to accept it as the expected output:
   ```bash
   cp wip/my_error.stderr tests/compile_fail/
   ```
7. Run the test again to verify that it now passes

#### Best Practices for Compile-Fail Tests

- Include detailed comments explaining why the code should fail to compile
- Focus on testing one specific error case per file
- Use descriptive file names that indicate what error is being tested
- Ensure the error messages are clear and helpful for users
- Keep the test cases minimal while still demonstrating the error

## Test Purpose

These tests serve multiple purposes:

1. **Verification**: Ensure that the session type system correctly enforces protocol adherence at compile time
2. **Documentation**: Provide concrete examples of how to use the library
3. **Regression Testing**: Prevent regressions in the type system

The integration tests demonstrate correct usage of the session type system, while the compile-fail tests verify that the type system correctly rejects invalid protocols.