use trybuild::TestCases;

#[test]
fn test_projection_integration() {
    let result = TestCases::new().pass("tests/pass/projection_integration.rs");
}