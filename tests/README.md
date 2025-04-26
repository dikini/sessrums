# Session Types Library (sez) Tests

This directory contains tests for the sez library, organized into different categories. The tests are designed to verify both the type-level properties and runtime behavior of session types, ensuring that the library correctly enforces protocol adherence at compile time.

## Test Structure

- **integration/**: Integration tests that demonstrate working protocol examples
  - `mod.rs`: Common test infrastructure, helper functions, and utilities
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
   - `protocol_2_test.rs`: Runs the tests for Protocol 2
   - `protocol_3_test.rs`: Runs the tests for Protocol 3
   - `protocol_4_test.rs`: Runs the tests for Protocol 4
   - Integration tests for Protocol 5 are included in the general integration tests

## Running Tests

### Integration Tests

To run all integration tests:

```bash
cargo test --test 'protocol_*' --test integration_tests
```

To run a specific protocol test:

```bash
cargo test --test protocol_1_test
```

To run a specific test function:

```bash
cargo test --test protocol_1_test test_ping_pong_protocol
```

### Compile-Fail Tests

The compile-fail tests use the `trybuild` crate to verify that certain code patterns fail to compile with the expected error messages. These tests are crucial for ensuring that the session type system correctly rejects invalid protocols at compile time.

#### Running Compile-Fail Tests

To run the compile-fail tests:

```bash
cargo test --test compile_fail
```

#### How to Interpret Compile-Fail Test Results

When running compile-fail tests:

- **Success**: The test passes if the code fails to compile with exactly the expected error messages in the corresponding `.stderr` file.
- **Failure**: The test fails if either:
  - The code compiles when it should fail
  - The error messages don't match the expected ones in the `.stderr` file

The error messages in the `.stderr` files are carefully crafted to demonstrate specific type safety properties of the session type system. They show how the Rust compiler detects protocol violations at compile time.

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
- Add visual diagrams to illustrate the protocol error
- Include a correct version of the protocol for reference

## How to Add New Tests

### Adding a New Protocol Example

To add a new protocol example:

1. Create a new file in the `tests/integration/` directory (e.g., `tests/integration/protocol_6.rs`)
2. Define the protocol types using the session type primitives (`Send`, `Recv`, `Choose`, `Offer`, `End`)
3. Add test functions that verify the type-level properties of the protocol
4. Update `tests/integration/mod.rs` to include the new protocol module
5. Create a test runner file in the `tests` directory (e.g., `tests/protocol_6_test.rs`)
6. Update the documentation in `tests/EXAMPLES.md` to include the new protocol

### Adding a New Error Example

To add a new error example:

1. Create a new file in the `tests/compile_fail/` directory (e.g., `tests/compile_fail/error_5.rs`)
2. Write code that should fail to compile, with detailed comments explaining why it should fail
3. Follow the steps in "Adding New Compile-Fail Tests" above
4. Update the documentation in `tests/ERRORS.md` to include the new error example

## How Session Types Demonstrate Type Safety

Session types provide compile-time guarantees about communication protocols by encoding the protocol in the type system. The tests in this directory demonstrate several key aspects of session type safety:

1. **Protocol Adherence**: The type system ensures that both parties follow the agreed-upon protocol.
2. **Deadlock Freedom**: The duality relationship between client and server protocols ensures that communication can proceed without deadlocks.
3. **Type Safety**: The type system ensures that the correct types are sent and received at each step.
4. **Protocol Completion**: Both sides must follow the protocol to completion, ensuring that no communication is left hanging.
5. **Branching Safety**: When using `Choose` and `Offer` types, the type system ensures that all branches are handled correctly.

The integration tests demonstrate correct usage of the session type system, while the compile-fail tests verify that the type system correctly rejects invalid protocols. Together, they provide comprehensive verification of the session type system's safety properties.

For more detailed information about the protocol examples, see [EXAMPLES.md](EXAMPLES.md).
For more detailed information about the error examples, see [ERRORS.md](ERRORS.md).