//! Test runner for Protocol 3: Simple Choice
//!
//! This file serves as an entry point for running the Protocol 3 tests.
//! It imports the integration test module and re-exports the protocol_3 module.

mod integration;

// Re-export protocol_3 module to make it available for testing
pub use integration::protocol_3;