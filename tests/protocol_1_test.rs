//! Test runner for Protocol 1: Simple Send/Recv Ping-Pong

// Import the integration test module
mod integration;

// Re-export the protocol_1 test
#[path = "integration/protocol_1.rs"]
mod protocol_1_impl;

// This ensures that the test is run when executing `cargo test --test protocol_1_test`