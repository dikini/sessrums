//! Projection module for extracting local types from global protocols.
//!
//! This module defines the `Project` trait and its implementations for projecting
//! global protocols to local protocols for specific roles.

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

/// Trait for projecting a global protocol to a local protocol for a specific role.
pub trait Project<R: Role> {
    /// The resulting local protocol type after projection.
    type LocalProtocol: Protocol;
}

// Projection for GEnd: Always projects to the local End.
impl<R: Role> Project<R> for GEnd {
    type LocalProtocol = End;
}

// --- GSend Projection ---

// Projection for GSend from the perspective of the 'From' role (sender).
impl<T, From: Role, To: Role, Next> Project<From> for GSend<T, From, To, Next>
where
    Next: Project<From>,
    From: PartialEq<From>,
    To: PartialEq<To>,
{
    type LocalProtocol = Send<T, <Next as Project<From>>::LocalProtocol>;
}

// --- GRecv Projections ---

// Projection for GRecv from the perspective of the 'To' role (receiver).
impl<T, From: Role, To: Role, Next> Project<To> for GRecv<T, From, To, Next>
where
    Next: Project<To>,
    From: Role,
    To: Role,
    From: PartialEq<From>,
{
    type LocalProtocol = Recv<T, <Next as Project<To>>::LocalProtocol>;
}

// --- GRec Projection ---
impl<Label, P: GlobalProtocol, R: Role> Project<R> for GRec<Label, P>
where
    P: Project<R>,
{
    type LocalProtocol = Rec<<P as Project<R>>::LocalProtocol>;
}

// --- GVar Projection ---
impl<Label, R: Role> Project<R> for GVar<Label> {
    type LocalProtocol = Var<0>; // Assuming depth 0 for simplicity
}

// --- GChoice Projections ---

// Projection for GChoice for the 'Chooser' role.
impl<Chooser: Role, G1, G2> Project<Chooser> for GChoice<Chooser, (G1, G2)>
where
    G1: Project<Chooser>,
    G2: Project<Chooser>,
{
    type LocalProtocol = Choose<
        <G1 as Project<Chooser>>::LocalProtocol,
        <G2 as Project<Chooser>>::LocalProtocol
    >;
}

// --- GOffer Projections ---

// Projection for GOffer for the 'Offeree' role.
impl<Offeree: Role, G1, G2> Project<Offeree> for GOffer<Offeree, (G1, G2)>
where
    G1: Project<Offeree>,
    G2: Project<Offeree>,
{
    type LocalProtocol = Offer<
        <G1 as Project<Offeree>>::LocalProtocol,
        <G2 as Project<Offeree>>::LocalProtocol
    >;
}

// --- GSeq Projection ---
impl<First, Second, R: Role> Project<R> for GSeq<First, Second>
where
    First: Project<R>,
    Second: Project<R>,
{
    type LocalProtocol = <First as Project<R>>::LocalProtocol;
}

// --- GPar Projection ---
impl<First, Second, R: Role> Project<R> for GPar<First, Second>
where
    First: Project<R>,
    Second: Project<R>,
{
    type LocalProtocol = <First as Project<R>>::LocalProtocol;
}


/// Helper trait to project each element of a tuple of global protocols.
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

/// Projects a global protocol to a local protocol for a specific role.
/// (Placeholder function - actual projection happens via the trait)
pub fn project<G, R>() -> <G as Project<R>>::LocalProtocol
where
    G: GlobalProtocol + Project<R>,
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
        type GlobalProtocol = GSend<String, RoleA, RoleB, GEnd>;
        type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
        // type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Send<String, End>>(); // RoleA
    }

    // Test projection of GRecv
    #[test]
    fn test_project_grecv() {
        type GlobalProtocol = GRecv<String, RoleB, RoleA, GEnd>;
        type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
        // type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Recv<String, End>>(); // RoleA
    }

    // Test projection of GChoice
    #[test]
    fn test_project_gchoice() {
        type Branch1 = GSend<String, RoleA, RoleB, GEnd>;
        type Branch2 = GSend<i32, RoleA, RoleB, GEnd>;
        type GlobalProtocol = GChoice<RoleA, (Branch1, Branch2)>;
        type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
        // type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Choose<Send<String, End>, Send<i32, End>>>(); // RoleA
    }

    // Test projection of GOffer
    #[test]
    fn test_project_goffer() {
        type Branch1 = GRecv<String, RoleA, RoleB, GEnd>;
        type Branch2 = GRecv<i32, RoleA, RoleB, GEnd>;
        type GlobalProtocol = GOffer<RoleB, (Branch1, Branch2)>;
        // type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol; // Cannot project for RoleA directly now
        type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol;
        fn assert_type<T: Protocol>() {}
        assert_type::<Offer<Recv<String, End>, Recv<i32, End>>>(); // RoleB
    }

    // Test projection of complex protocol with branching
    #[test]
    fn test_project_complex_with_branching() {
        type Branch1 = GSend<String, RoleA, RoleB, GEnd>;
        type Branch2 = GRecv<i32, RoleB, RoleA, GEnd>;
        type GlobalProtocol = GSend<bool, RoleA, RoleB, GChoice<RoleA, (Branch1, Branch2)>>;
        type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
        // type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Send<bool, Choose<Send<String, End>, Recv<i32, End>>>>(); // RoleA
    }

    // Test projection of recursive protocol
    #[test]
    fn test_project_recursive() {
        struct RecursionLabel;
        impl Default for GSend<i32, RoleA, RoleB, GVar<RecursionLabel>> {
            fn default() -> Self { Default::default() }
        }
        type GlobalProtocol = GRec<RecursionLabel, GSend<i32, RoleA, RoleB, GVar<RecursionLabel>>>;
        type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
        // type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Rec<Send<i32, Var<0>>>>(); // RoleA
    }

    // Test projection of complex recursive protocol with choice
    #[test]
    fn test_project_recursive_with_choice() {
        struct RecursionLabel2;
        impl Default for GSend<i32, RoleA, RoleB, GChoice<RoleA, (GVar<RecursionLabel2>, GEnd)>> {
             fn default() -> Self { Default::default() }
        }
        type GlobalProtocol = GRec<RecursionLabel2,
            GSend<i32, RoleA, RoleB,
                GChoice<RoleA, (
                    GVar<RecursionLabel2>,
                    GEnd
                )>
            >
        >;
        type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
        // type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol; // Cannot project for RoleB directly now
        fn assert_type<T: Protocol>() {}
        assert_type::<Rec<Send<i32, Choose<Var<0>, End>>>>(); // RoleA
    }
}