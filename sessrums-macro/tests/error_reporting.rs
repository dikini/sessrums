//! Tests for error reporting in the sessrums-macro crate.
//!
//! These tests verify that the macro correctly reports errors with clear,
//! actionable messages when users make mistakes in their protocol definitions.

/// Test syntax errors in protocol definitions
#[test]
fn test_syntax_errors() {
    let test_cases = trybuild::TestCases::new();
    test_cases.compile_fail("tests/fail/syntax/invalid_choice_syntax.rs");
    test_cases.compile_fail("tests/fail/syntax/missing_semicolon.rs");
    test_cases.compile_fail("tests/fail/syntax/invalid_message_arrow.rs");
}

/// Test participant-related errors in protocol definitions
#[test]
fn test_participant_errors() {
    let test_cases = trybuild::TestCases::new();
    test_cases.compile_fail("tests/fail/participant/undefined_participant.rs");
    test_cases.compile_fail("tests/fail/participant/duplicate_participant.rs");
    test_cases.compile_fail("tests/fail/participant/invalid_participant_name.rs");
}

/// Test recursion-related errors in protocol definitions
#[test]
fn test_recursion_errors() {
    let test_cases = trybuild::TestCases::new();
    test_cases.compile_fail("tests/fail/recursion/undefined_recursion_label.rs");
    test_cases.compile_fail("tests/fail/recursion/duplicate_recursion_label.rs");
    test_cases.compile_fail("tests/fail/recursion/continue_outside_recursion.rs");
}

/// Test type-related errors in protocol definitions
#[test]
fn test_type_errors() {
    let test_cases = trybuild::TestCases::new();
    test_cases.compile_fail("tests/fail/type/invalid_message_type.rs");
    test_cases.compile_fail("tests/fail/type/unsupported_generic_type.rs");
}

/// Test semantic errors in protocol definitions
#[test]
fn test_semantic_errors() {
    let test_cases = trybuild::TestCases::new();
    test_cases.compile_fail("tests/fail/semantic/invalid_choice_role.rs");
    test_cases.compile_fail("tests/fail/semantic/empty_choice.rs");
    test_cases.compile_fail("tests/fail/semantic/unreachable_code.rs");
}