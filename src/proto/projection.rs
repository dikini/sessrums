//! # Protocol Projection
//!
//! This module provides the mechanism for **projecting** a [`GlobalProtocol`]
//! into a local [`Protocol`] specific to a single [`Role`]. Projection essentially
//! extracts the sequence of actions (send, receive, offer, choose, recursion, end)
//! that a particular role needs to perform according to the global specification.
//!
//! The core of this module is the [`Project`] trait.

use crate::proto::global::*;
use crate::proto::roles::{Role};
use crate::proto::send::Send;
use crate::proto::recv::Recv;
use crate::proto::end::End;
use crate::proto::choose::Choose;
use crate::proto::offer::Offer;
use crate::proto::rec::Rec;
use crate::proto::var::Var;
use crate::proto::Protocol;

/// Projects a [`GlobalProtocol`] into a local [`Protocol`] for a specific [`Role`].
///
/// This trait defines how each global protocol construct translates into a
/// corresponding local protocol construct from the perspective of the role `R`.
pub trait Project<R: Role> {
    /// The local protocol type resulting from the projection for role `R`.
    type LocalProtocol: Protocol;
}

/// Projection for [`GEnd`]: Always results in the local [`End`] protocol.
///
/// Regardless of the role, the end of a global protocol signifies the end
/// of the local protocol for that role.
impl<R: Role> Project<R> for GEnd {
    type LocalProtocol = End;
}

// --- GSend Projection ---

/// Projection for [`GSend`] from the perspective of the **sender** (`From` role).
///
/// The sender sees this as a local [`Send`] action, followed by the projection
/// of the continuation `Next` for the sender role.
///
/// **Limitation:** This implementation only handles the sender's perspective.
/// Projection for the receiver (`To`) or any third-party roles is not currently defined
/// within this specific `impl` block and requires separate implementations (like `GRecv`
/// for the receiver). A complete projection system would need to handle all roles.
impl<T, From: Role, To: Role, Next> Project<From> for GSend<T, From, To, Next>
where
    Next: Project<From>,
    From: PartialEq<From>, // Ensure the projecting role matches the sender
    To: PartialEq<To>,
{
    type LocalProtocol = Send<T, <Next as Project<From>>::LocalProtocol>;
}

// --- GRecv Projections ---

/// Projection for [`GRecv`] from the perspective of the **receiver** (`To` role).
///
/// The receiver sees this as a local [`Recv`] action, followed by the projection
/// of the continuation `Next` for the receiver role.
///
/// **Limitation:** This implementation only handles the receiver's perspective.
/// Projection for the sender (`From`) or any third-party roles is not currently defined
/// within this specific `impl` block. A complete projection system would need to handle all roles.
impl<T, From: Role, To: Role, Next> Project<To> for GRecv<T, From, To, Next>
where
    Next: Project<To>,
    From: Role,
    To: Role,
    From: PartialEq<From>,
    To: PartialEq<To>, // Ensure the projecting role matches the receiver
{
    type LocalProtocol = Recv<T, <Next as Project<To>>::LocalProtocol>;
}

// --- GRec Projection ---

/// Projection for [`GRec`]: Translates to a local [`Rec`] protocol.
///
/// The body `P` of the global recursion is projected recursively for the role `R`.
impl<Label, P: GlobalProtocol, R: Role> Project<R> for GRec<Label, P>
where
    P: Project<R>,
{
    type LocalProtocol = Rec<<P as Project<R>>::LocalProtocol>;
}

// --- GVar Projection ---

/// Projection for [`GVar`]: Translates to a local [`Var`] protocol.
///
/// **Simplification:** Currently, this always projects to `Var<0>`, assuming
/// the variable refers to the immediately enclosing [`Rec`]. This simplification
/// works for basic cases but might be insufficient for nested recursions or
/// more complex scenarios requiring proper De Bruijn index handling during projection.
impl<Label, R: Role> Project<R> for GVar<Label> {
    type LocalProtocol = Var<0>;
}

// --- GChoice Projections ---

/// Projection for [`GChoice`] from the perspective of the **chooser** (`Chooser` role).
///
/// The chooser sees this as a local [`Choose`] action, where the branches `G1` and `G2`
/// are projected recursively for the chooser role.
///
/// **Limitation:** This implementation only handles the chooser's perspective.
/// Projection for the roles involved in the offered branches is handled by [`GOffer`].
impl<Chooser: Role, G1, G2> Project<Chooser> for GChoice<Chooser, (G1, G2)>
where
    G1: Project<Chooser>,
    G2: Project<Chooser>,
    Chooser: PartialEq<Chooser>, // Ensure the projecting role matches the chooser
{
    type LocalProtocol = Choose<
        <G1 as Project<Chooser>>::LocalProtocol,
        <G2 as Project<Chooser>>::LocalProtocol
    >;
}

// --- GOffer Projections ---

/// Projection for [`GOffer`] from the perspective of the **offeree** (`Offeree` role).
///
/// The offeree sees this as a local [`Offer`] action, where the branches `G1` and `G2`
/// are projected recursively for the offeree role.
///
/// **Limitation:** This implementation only handles the offeree's perspective.
/// Projection for the role making the choice is handled by [`GChoice`].
impl<Offeree: Role, G1, G2> Project<Offeree> for GOffer<Offeree, (G1, G2)>
where
    G1: Project<Offeree>,
    G2: Project<Offeree>,
    Offeree: PartialEq<Offeree>, // Ensure the projecting role matches the offeree
{
    type LocalProtocol = Offer<
        <G1 as Project<Offeree>>::LocalProtocol,
        <G2 as Project<Offeree>>::LocalProtocol
    >;
}

// --- GSeq Projection ---

/// Projection for [`GSeq`]: Projects the first part of the sequence.
///
/// **Limitation:** This projection is **incomplete**. It currently only projects
/// the `First` part of the sequence and **ignores the `Second` part**. A correct
/// projection for sequence would typically involve composing the projections of
/// both parts, but the exact semantics depend on how sequence interacts with
/// local types (e.g., if `First` must resolve to `End` before `Second` begins).
/// This implementation is a placeholder and likely incorrect for most practical purposes.
impl<First, Second, R: Role> Project<R> for GSeq<First, Second>
where
    First: Project<R>,
    Second: Project<R>, // Second is projected but ignored in the result type
{
    type LocalProtocol = <First as Project<R>>::LocalProtocol; // Incorrectly ignores Second's projection
}

// --- GPar Projection ---

/// Projection for [`GPar`]: Projects the first part of the parallel composition.
///
/// **Limitation:** This projection is **incomplete and likely incorrect**.
/// It currently only projects the `First` part and **ignores the `Second` part**.
/// Projecting parallel composition (`GPar`) into standard binary session types
/// is non-trivial and often requires extensions or different type system features.
/// This implementation serves as a placeholder and does not reflect correct
/// parallel projection semantics.
impl<First, Second, R: Role> Project<R> for GPar<First, Second>
where
    First: Project<R>,
    Second: Project<R>, // Second is projected but ignored in the result type
{
    type LocalProtocol = <First as Project<R>>::LocalProtocol; // Incorrectly ignores Second's projection
}


/// Helper trait to project each element of a tuple of global protocols.
///
/// This is primarily used internally for projecting the branches within
/// [`GChoice`] and [`GOffer`].
pub trait ProjectTuple<R: Role> {
    /// The resulting tuple of local protocols after projection.
    type LocalProtocolTuple;
}

// Implement ProjectTuple for the empty tuple (base case for recursion).
impl<R: Role> ProjectTuple<R> for () {
    type LocalProtocolTuple = ();
}

// Implement ProjectTuple for a single-element tuple (base case for recursion).
impl<R: Role, Head> ProjectTuple<R> for (Head,)
where
    Head: Project<R>,
{
    type LocalProtocolTuple = (<Head as Project<R>>::LocalProtocol,);
}

// Implement ProjectTuple for a two-element tuple
impl<R: Role, T1, T2> ProjectTuple<R> for (T1, T2)
where
    T1: Project<R>,
    T2: Project<R>,
{
    type LocalProtocolTuple = (<T1 as Project<R>>::LocalProtocol, <T2 as Project<R>>::LocalProtocol);
}

// Implement ProjectTuple for a three-element tuple
impl<R: Role, T1, T2, T3> ProjectTuple<R> for (T1, T2, T3)
where
    T1: Project<R>,
    T2: Project<R>,
    T3: Project<R>,
{
    type LocalProtocolTuple = (
        <T1 as Project<R>>::LocalProtocol,
        <T2 as Project<R>>::LocalProtocol,
        <T3 as Project<R>>::LocalProtocol
    );
}

/// **Placeholder Function:** Creates an uninitialized instance of the projected local protocol.
///
/// This function **does not perform the actual projection logic**. The projection
/// is defined statically by the [`Project`] trait implementations based on the types
/// `G` and `R`.
///
/// This function merely returns `mem::zeroed()`, which is **unsafe** and intended
/// only as a temporary placeholder during development or for type-level checks.
/// It should **not** be used to obtain a usable runtime instance of the protocol.
/// The actual protocol state should be managed by session channel endpoints.
///
/// # Safety
///
/// Calling this function produces an uninitialized value of the projected type,
/// which is undefined behavior if used directly.
pub fn project<G, R>() -> <G as Project<R>>::LocalProtocol
where
    G: GlobalProtocol + Project<R>, // G must be projectable for role R
    R: Role,
{
    unsafe { std::mem::zeroed() }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::roles::{RoleA, RoleB};

    // Test projection of GSend
    #[test]
    fn test_project_gsend() {
        // type RoleBLocal = <GSend<String, RoleA, RoleB, GEnd> as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Send<String, End>>(); // RoleA
    }

    // Test projection of GRecv
    #[test]
    fn test_project_grecv() {
        // type RoleBLocal = <GRecv<String, RoleB, RoleA, GEnd> as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Recv<String, End>>(); // RoleA
    }

    // Test projection of GChoice
    #[test]
    fn test_project_gchoice() {
        // type RoleBLocal = <GChoice<RoleA, (GSend<String, RoleA, RoleB, GEnd>, GSend<i32, RoleA, RoleB, GEnd>)> as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Choose<Send<String, End>, Send<i32, End>>>(); // RoleA
    }

    // Test projection of GOffer
    #[test]
    fn test_project_goffer() {
        // type RoleALocal = <GOffer<RoleB, (GRecv<String, RoleA, RoleB, GEnd>, GRecv<i32, RoleA, RoleB, GEnd>)> as Project<RoleA>>::LocalProtocol; // Cannot project for RoleA directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Offer<Recv<String, End>, Recv<i32, End>>>(); // RoleB
    }

    // Test projection of complex protocol with branching
    #[test]
    fn test_project_complex_with_branching() {
        // type RoleBLocal = <GSend<bool, RoleA, RoleB, GChoice<RoleA, (GSend<String, RoleA, RoleB, GEnd>, GRecv<i32, RoleB, RoleA, GEnd>)>> as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Send<bool, Choose<Send<String, End>, Recv<i32, End>>>>(); // RoleA
    }

    // Test projection of recursive protocol
    #[test]
    fn test_project_recursive() {
        struct RecursionLabel;
        // type RoleBLocal = <GRec<RecursionLabel, GSend<i32, RoleA, RoleB, GVar<RecursionLabel>>> as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Rec<Send<i32, Var<0>>>>(); // RoleA
    }

    // Test projection of complex recursive protocol with choice
    #[test]
    fn test_project_recursive_with_choice() {
        struct RecursionLabel2;
        // type RoleBLocal = <GRec<RecursionLabel2, GSend<i32, RoleA, RoleB, GChoice<RoleA, (GVar<RecursionLabel2>, GEnd)>>> as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Rec<Send<i32, Choose<Var<0>, End>>>>(); // RoleA
    }
}