//! Test runner for Protocol 2: Request/Response
//!
//! This file imports the protocol_2 module from the integration tests
//! and allows it to be run with `cargo test --test protocol_2_test`.

// Import the integration test module
pub mod integration;

// Re-export the protocol_2 module for testing
pub use integration::protocol_2;

#[test]
fn run_protocol_2_tests() {
    // This is a placeholder test that always passes.
    // The actual tests are in the protocol_2 module.
    // This function ensures that `cargo test --test protocol_2_test` will run all tests.
    assert!(true);
}