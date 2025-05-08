//! Tests for complex protocol definitions in the sessrums-macro crate.
//!
//! These tests verify that the macro correctly parses and processes
//! complex protocol definitions with recursion, choice blocks, and nested structures.

#[test]
fn test_recursive_protocol() {
    let result = trybuild::TestCases::new().pass("tests/pass/recursive_protocol.rs");
}

#[test]
fn test_choice_protocol() {
    let result = trybuild::TestCases::new().pass("tests/pass/choice_protocol.rs");
}

#[test]
fn test_nested_recursion() {
    let result = trybuild::TestCases::new().pass("tests/pass/nested_recursion.rs");
}

#[test]
fn test_recursion_in_choice() {
    let result = trybuild::TestCases::new().pass("tests/pass/recursion_in_choice.rs");
}

#[test]
fn test_complex_protocol() {
    let result = trybuild::TestCases::new().pass("tests/pass/complex_protocol.rs");
}