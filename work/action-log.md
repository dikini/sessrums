# Action Log

## Task 2.3: Implement Projection

- Implemented the `Project<R: Role>` trait in `src/proto/projection.rs` to extract local types from a global protocol.
- Implemented projection for all global protocol types:
  - `GEnd` projects to `End` for any role.
  - `GSend<T, From, To, Next>` projects to:
    - `Send<T, <Next as Project<From>>::LocalProtocol>` for the `From` role.
    - `Recv<T, <Next as Project<To>>::LocalProtocol>` for the `To` role.
  - `GRecv<T, From, To, Next>` projects to:
    - `Send<T, <Next as Project<From>>::LocalProtocol>` for the `From` role.
    - `Recv<T, <Next as Project<To>>::LocalProtocol>` for the `To` role.
  - `GRec<Label, Protocol>` projects to `Rec<<Protocol as Project<R>>::LocalProtocol>` for any role.
  - `GVar<Label>` projects to `Var<0>` for any role.
- Implemented the `project` function to extract local types from a global protocol.
- Added unit tests to verify that projections produce correct local types.
- Added integration tests to verify the projection functionality with more complex examples.
- Updated the documentation in `docs/session-types-documentation.md` to explain the projection process.
- Exposed the necessary types and functions in `src/proto/mod.rs` for public use.

The implementation allows for projecting global protocols to local protocols for specific roles, which is a key component of multiparty session types. This enables each participant to know exactly what actions they need to perform in the protocol.

## Task 3.1: Add Branching and Choice

- Completed the implementation of `GChoice<Chooser, Branches>` and `GOffer<Offeree, Branches>` in `src/proto/global.rs`:
  - Implemented the `validate` method to check that branches are valid
  - Implemented the `involved_roles` method to collect all roles from all branches
- Implemented projection for `GChoice` and `GOffer` in `src/proto/projection.rs`:
  - For `GChoice<Chooser, Branches>`:
    - Projects to `Choose<L, R>` for the Chooser role
    - Projects to `Offer<L, R>` for other roles
  - For `GOffer<Offeree, Branches>`:
    - Projects to `Offer<L, R>` for the Offeree role
    - Projects to `Choose<L, R>` for other roles
- Added unit tests in `src/proto/global.rs` and `src/proto/projection.rs` to verify the branching functionality
- Created integration tests in `tests/branching_tests.rs` to verify the branching functionality with more complex examples
- Updated the documentation in `docs/session-types-documentation.md` to explain the branching and choice functionality

The implementation allows for expressing protocols with branching, where one role makes a choice and other roles offer options. This is a key component of multiparty session types, enabling more complex communication patterns beyond simple message exchanges.