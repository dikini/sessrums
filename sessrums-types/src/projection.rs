//! Projection mechanism for multiparty session types.
//!
//! This module provides the functionality to project global protocols to
//! role-specific local protocols. Projection is a fundamental operation in
//! multiparty session types that transforms a global protocol description
//! (involving all participants) into role-specific local protocols (describing
//! the behavior of individual participants).
//!
//! # Projection Rules
//!
//! The projection operation follows these core rules:
//!
//! 1. **Message Projection**:
//!    - When role A sends to role B, A's projection becomes `Send` and B's becomes `Receive`
//!    - When another role sends to someone else, it's omitted from the local protocol
//!
//! 2. **Choice Projection**:
//!    - When role A makes a choice, A's projection becomes `Select`
//!    - Other roles' projections become `Offer`
//!    - Branch labels are preserved during projection
//!
//! 3. **Recursion Projection**:
//!    - Recursion points (`Rec`) and variables (`Var`) are preserved in projection
//!    - The body of a recursion is projected for each role
//!
//! 4. **End Projection**:
//!    - The `End` global interaction projects to `End` for all roles
//!
//! # Examples
//!
//! ```
//! use sessrums_types::roles::{Client, Server};
//! use sessrums_types::session_types::global::GlobalInteraction;
//! use sessrums_types::projection::project_for_role;
//!
//! // Define a global protocol
//! let global = GlobalInteraction::message(
//!     "client",
//!     "server",
//!     GlobalInteraction::end(),
//! );
//!
//! // Project for Client role
//! let client_protocol = project_for_role::<Client, ()>(global.clone());
//!
//! // Project for Server role
//! let server_protocol = project_for_role::<Server, ()>(global.clone());
//! ```

use crate::roles::{Role, Client, Server};
use crate::session_types::common::{RoleIdentifier, RecursionLabel};
use crate::session_types::global::GlobalInteraction;
use crate::session_types::local::LocalProtocol;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::any::Any;

/// Checks if a local protocol contains any meaningful interactions for a role
/// Returns true if the protocol contains Send, Receive, Select, or Offer actions
/// Returns false if the protocol only contains End or Var actions
fn contains_meaningful_interactions<R: Role + RoleExt, M: Clone>(
    protocol: &LocalProtocol<R, M>
) -> bool {
    match protocol {
        LocalProtocol::Send { .. } | LocalProtocol::Receive { .. } 
        | LocalProtocol::Select { .. } | LocalProtocol::Offer { .. } => true,
        
        LocalProtocol::Rec { body, .. } => contains_meaningful_interactions(body),
        
        LocalProtocol::Var { .. } | LocalProtocol::End { .. } => false,
    }
}

/// Trait for projecting a global protocol to a role-specific local protocol.
///
/// The `Project<R, M>` trait defines how a global protocol is transformed into a
/// local protocol for a specific role `R`. This is a key operation in multiparty
/// session types, as it allows deriving role-specific behaviors from a single
/// global definition.
///
/// # Type Parameters
///
/// * `R` - A type parameter implementing the `Role` trait, representing the role
///         for which the protocol is being projected
///
/// # Associated Types
///
/// * `Output` - The resulting local protocol type after projection
///
/// # Examples
///
/// ```
/// use sessrums_types::roles::Client;
/// use sessrums_types::projection::project_for_role;
/// use sessrums_types::session_types::global::GlobalInteraction;
/// use std::marker::PhantomData;
///
/// // Define a global protocol
/// let global = GlobalInteraction::message(
///     "client",
///     "server",
///     GlobalInteraction::message(
///         "server",
///         "client",
///         GlobalInteraction::end(),
///     ),
/// );
///
/// // Project it for the Client role
/// let client_local = project_for_role::<Client, ()>(global);
/// ```
pub trait Project<R: Role, M: Clone = ()> {
    /// The resulting local protocol type after projection
    type Output;
    
    /// Project this global protocol to a local protocol for role R
    fn project(self) -> Self::Output;
}

/// Project a global protocol for a specific role
///
/// This helper function simplifies the projection process by handling the
/// type inference and trait resolution.
///
/// # Type Parameters
///
/// * `R` - A type parameter implementing the `Role` trait, representing the role
///         for which the protocol is being projected
///
/// # Parameters
///
/// * `global` - The global protocol to project
///
/// # Returns
///
/// The projected local protocol for role `R`
///
/// # Examples
///
/// ```
/// use sessrums_types::roles::Client;
/// use sessrums_types::projection::project_for_role;
/// use sessrums_types::session_types::global::GlobalInteraction;
/// use std::marker::PhantomData;
///
/// // Define a global protocol
/// let global = GlobalInteraction::message(
///     "client",
///     "server",
///     GlobalInteraction::end(),
/// );
///
/// // Project it for the Client role
/// let client_local = project_for_role::<Client, ()>(global);
/// ```
pub fn project_for_role<R: Role + RoleExt, M: Clone>(global: GlobalInteraction<M>) -> LocalProtocol<R, M> {
    <GlobalInteraction<M> as Project<R, M>>::project(global)
}

/// Project a global protocol for all roles in a set
///
/// This helper function projects a global protocol for multiple roles at once,
/// returning a map from role identifiers to their corresponding local protocols.
///
/// # Parameters
///
/// * `global` - The global protocol to project
/// * `roles` - A slice of role identifiers to project for
///
/// # Returns
///
/// A HashMap mapping role identifiers to their projected local protocols
///
/// # Note
///
/// This function requires that the roles provided implement the necessary traits
/// for projection. In practice, this would typically be used with a macro or
/// code generation to ensure type safety.
pub fn project_for_all_roles<M: Clone>(
    _global: GlobalInteraction<M>,
    roles: &[RoleIdentifier],
) -> HashMap<RoleIdentifier, Box<dyn Any>> {
    // This is a placeholder implementation. In a real implementation,
    // we would need to handle the type erasure and dynamic dispatch
    // more carefully, possibly using a trait object or enum approach.
    roles
        .iter()
        .map(|role| (role.clone(), Box::new(()) as Box<dyn Any>))
        .collect()
}

/// Extension trait for Role to check if a role matches a role identifier
pub trait RoleExt: Role {
    /// Check if this role type matches the given role identifier
    fn is_role(role_id: &RoleIdentifier) -> bool;
}

// Implement RoleExt for the existing roles
impl RoleExt for Client {
    fn is_role(role_id: &RoleIdentifier) -> bool {
        role_id.name().to_lowercase() == "client"
    }
}

impl RoleExt for Server {
    fn is_role(role_id: &RoleIdentifier) -> bool {
        role_id.name().to_lowercase() == "server"
    }
}

/// Implementation of Project for GlobalInteraction
impl<R: Role + RoleExt, M: Clone> Project<R, M> for GlobalInteraction<M> {
    type Output = LocalProtocol<R, M>;
    
    fn project(self) -> Self::Output {
        match self {
            GlobalInteraction::Message { from, to, msg: _, cont } => {
                if R::is_role(&from) {
                    // Sender role gets a Send
                    LocalProtocol::Send {
                        to: to.clone(),
                        msg: PhantomData,
                        cont: Box::new(cont.project()),
                        _role: PhantomData,
                    }
                } else if R::is_role(&to) {
                    // Receiver role gets a Receive
                    LocalProtocol::Receive {
                        from: from.clone(),
                        msg: PhantomData,
                        cont: Box::new(cont.project()),
                        _role: PhantomData,
                    }
                } else {
                    // Uninvolved role skips this interaction
                    cont.project()
                }
            },
            GlobalInteraction::End => LocalProtocol::End {
                _role: PhantomData,
            },
            GlobalInteraction::Choice { decider, branches } => {
                if R::is_role(&decider) {
                    // Deciding role gets a Select
                    LocalProtocol::Select {
                        branches: branches
                            .into_iter()
                            .map(|(label, branch)| (label, Box::new(branch.project())))
                            .collect(),
                        _role: PhantomData,
                    }
                } else {
                    // Other roles get an Offer
                    LocalProtocol::Offer {
                        decider: decider.clone(),
                        branches: branches
                            .into_iter()
                            .map(|(label, branch)| (label, Box::new(branch.project())))
                            .collect(),
                        _role: PhantomData,
                    }
                }
            },
            GlobalInteraction::Rec { label, body } => {
                // Clone body before projecting to avoid moving out of self
                let body_clone = body.clone();
                
                // Project the body
                let projected_body = body_clone.project();
                
                // Check if the projected body contains meaningful interactions for this role
                if contains_meaningful_interactions(&projected_body) {
                    // If it does, preserve the recursion
                    LocalProtocol::Rec {
                        label: label.clone(),
                        body: Box::new(projected_body),
                        _role: PhantomData,
                    }
                } else {
                    // If it doesn't, skip the recursion and just return End
                    // This effectively "prunes" the recursion from the local protocol
                    LocalProtocol::End {
                        _role: PhantomData,
                    }
                }
            },
            GlobalInteraction::Var { label } => {
                // Recursion variable is preserved in projection
                LocalProtocol::Var {
                    label: label.clone(),
                    _role: PhantomData,
                }
            },
        }
    }
}