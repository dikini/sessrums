# Action Log

## Task 3.2: Add Recursion

- Added support for recursion in global protocols by implementing `GRec<Label, Protocol>` and `GVar<Label>` in `src/proto/global.rs`
- Implemented projection for recursive protocols in `src/proto/projection.rs`
- Removed the `Default` requirement from `GRec` implementation to make it more flexible
- Added comprehensive unit tests for recursive protocols in `src/proto/projection.rs`
- Created integration tests in `tests/recursion_tests.rs` to verify the functionality with more complex examples
- Updated documentation in `docs/session-types-documentation.md` to explain recursion in global protocols and projection
- All tests are passing, confirming that recursion is working correctly