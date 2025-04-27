//! Main integration test file for the sessrums library.
//!
//! This file imports and re-exports all the integration test modules,
//! allowing them to be run with `cargo test --test integration_tests`.
//! Individual protocol tests can also be run with `cargo test --test 'protocol_*'`.

// Import the integration test module
pub mod integration;

#[test]
fn run_all_integration_tests() {
    // This is a placeholder test that always passes.
    // The actual tests are in the integration module.
    // This function ensures that `cargo test --test integration_tests` will run all tests.
    assert!(true);
}