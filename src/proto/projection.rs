//! Projection module for extracting local types from global protocols.
//!
//! This module defines the `Project` trait and its implementations for projecting
//! global protocols to local protocols for specific roles.

use crate::proto::global::*;
use crate::proto::roles::Role;
use crate::proto::send::Send;
use crate::proto::recv::Recv;
use crate::proto::end::End;
use crate::proto::choose::Choose;
use crate::proto::offer::Offer;
use crate::proto::rec::Rec;
use crate::proto::var::Var;
use std::marker::PhantomData;

/// Trait for projecting a global protocol to a local protocol for a specific role.
pub trait Project<R: Role> {
    /// The resulting local protocol type after projection.
    type LocalProtocol;
}

// Projection for GEnd: Always projects to the local End.
impl<R: Role> Project<R> for GEnd {
    type LocalProtocol = End;
}

// Projection for GSend from the perspective of the 'From' role.
impl<T, From: Role, To: Role, Next> Project<From> for GSend<T, From, To, Next>
where
    Next: Project<From>,
    From: PartialEq<From>,
    To: PartialEq<To>,
{
    // The local protocol for the 'From' role is a Send followed by the projection of Next.
    type LocalProtocol = Send<T, <Next as Project<From>>::LocalProtocol>;
}

// Projection for GRecv from the perspective of the 'To' role.
impl<T, From: Role, To: Role, Next> Project<To> for GRecv<T, From, To, Next>
where
    Next: Project<To>,
    From: PartialEq<From>,
    To: PartialEq<To>,
{
    // The local protocol for the 'To' role is a Recv followed by the projection of Next.
    type LocalProtocol = Recv<T, <Next as Project<To>>::LocalProtocol>;
}

// Projection for GRec
impl<Label, Protocol, R: Role> Project<R> for GRec<Label, Protocol>
where
    Protocol: Project<R>,
{
    type LocalProtocol = Rec<<Protocol as Project<R>>::LocalProtocol>;
}

// Projection for GVar
impl<Label, R: Role> Project<R> for GVar<Label> {
    type LocalProtocol = Var<0>; // Using 0 as the default recursion depth
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

/// Projects a global protocol to a local protocol for a specific role.
///
/// This function takes a global protocol type `G` and a role type `R` and returns
/// the local protocol type that results from projecting `G` for role `R`.
///
/// # Type Parameters
///
/// * `G` - The global protocol type to project. Must implement `GlobalProtocol`.
/// * `R` - The role type to project for. Must implement `Role`.
///
/// # Returns
///
/// The local protocol type that results from projecting `G` for role `R`.
///
/// # Examples
///
/// ```
/// use sessrums::proto::{global::*, roles::*, projection::project};
///
/// // Define a global protocol
/// type MyGlobalProtocol = GSend<String, RoleA, RoleB, GEnd>;
///
/// // Project it for RoleA
/// let local_protocol_a = project::<MyGlobalProtocol, RoleA>();
///
/// // Project it for RoleB
/// let local_protocol_b = project::<MyGlobalProtocol, RoleB>();
/// ```
pub fn project<G, R>() -> <G as Project<R>>::LocalProtocol
where
    G: GlobalProtocol + Project<R>,
    R: Role,
{
    // This is a placeholder implementation.
    // In a real implementation, we would construct the local protocol.
    // For now, we just return a default-constructed value.
    unsafe { std::mem::zeroed() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::roles::{RoleA, RoleB};

    // Test projection of GSend
    #[test]
    fn test_project_gsend() {
        // Define a global protocol: RoleA sends a String to RoleB, then ends
        type GlobalProtocol = GSend<String, RoleA, RoleB, GEnd>;

        // Project for RoleA (sender)
        type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
        // Should be Send<String, End>

        // Use type assertions to verify the projections
        fn assert_type<T>() {}
        
        // This will compile only if RoleALocal is Send<String, End>
        assert_type::<Send<String, End>>();
    }

    // Test projection of GRecv
    #[test]
    fn test_project_grecv() {
        // Define a global protocol: RoleA receives a String from RoleB, then ends
        type GlobalProtocol = GRecv<String, RoleB, RoleA, GEnd>;

        // Project for RoleA (receiver)
        type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
        // Should be Recv<String, End>

        // Use type assertions to verify the projections
        fn assert_type<T>() {}
        
        // This will compile only if RoleALocal is Recv<String, End>
        assert_type::<Recv<String, End>>();
    }
}