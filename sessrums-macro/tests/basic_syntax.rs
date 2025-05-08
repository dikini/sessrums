//! Tests for basic DSL syntax in the sessrums-macro crate.
//!
//! These tests verify that the macro correctly parses and processes
//! basic protocol definitions using the DSL syntax.

#[test]
fn test_simple_protocol() {
    // Test a simple protocol with two participants and basic message passing
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/simple_protocol.rs");
}

#[test]
fn test_participant_with_alias() {
    // Test a protocol with participants that have aliases
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/participant_with_alias.rs");
}

#[test]
fn test_message_passing() {
    // Test a protocol with various message types
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/message_passing.rs");
}