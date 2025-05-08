//! Global protocol representation for multiparty session types.
//!
//! This module provides types for representing the global view of a multiparty
//! session type protocol, where interactions between all participants are
//! described from a centralized perspective.

use std::marker::PhantomData;
use super::common::{RoleIdentifier, Label};

/// Represents a global protocol interaction in a multiparty session type system.
///
/// `GlobalInteraction` provides a way to describe the global view of a protocol,
/// where all interactions between participants are specified from a centralized
/// perspective. This is in contrast to the local (or endpoint) view, where each
/// participant has their own perspective of the protocol.
///
/// The global view is essential for:
/// 1. Protocol design and specification
/// 2. Verification of protocol properties (e.g., deadlock freedom)
/// 3. Projection to local types for each participant
/// 4. Runtime monitoring and enforcement of protocol compliance
///
/// # Type Parameters
///
/// * `M` - A generic type parameter representing the message type
///
/// # Variants
///
/// * `Message` - Represents a message exchange from one role to another
/// * `End` - Represents the termination of the protocol
///
/// # Examples
///
/// ```
/// use std::marker::PhantomData;
/// use sessrums_types::session_types::global::GlobalInteraction;
/// use sessrums_types::session_types::common::RoleIdentifier;
///
/// // Define a simple protocol: client sends a request to server, server responds
/// let protocol: GlobalInteraction<String> = GlobalInteraction::Message {
///     from: RoleIdentifier::new("client"),
///     to: RoleIdentifier::new("server"),
///     msg: PhantomData::<String>,
///     cont: Box::new(GlobalInteraction::Message {
///         from: RoleIdentifier::new("server"),
///         to: RoleIdentifier::new("client"),
///         msg: PhantomData::<String>,
///         cont: Box::new(GlobalInteraction::End),
///     }),
/// };
/// ```
#[derive(Debug, Clone)]
pub enum GlobalInteraction<M: Clone> {
    /// Represents a message exchange from one role to another.
    ///
    /// # Fields
    ///
    /// * `from` - The role identifier of the sender
    /// * `to` - The role identifier of the receiver
    /// * `msg` - A phantom type representing the message type
    /// * `cont` - The continuation of the protocol after this message exchange
    Message {
        /// The role identifier of the sender
        from: RoleIdentifier,
        
        /// The role identifier of the receiver
        to: RoleIdentifier,
        
        /// A phantom type representing the message type
        msg: PhantomData<M>,
        
        /// The continuation of the protocol after this message exchange
        cont: Box<GlobalInteraction<M>>,
    },
    
    /// A choice point in the protocol, where the deciding role selects one of the branches.
    ///
    /// # Fields
    ///
    /// * `decider` - The role that makes the decision
    /// * `branches` - The available branches, each with a label and continuation
    ///
    /// # Examples
    ///
    /// ```
    /// use std::marker::PhantomData;
    /// use sessrums_types::session_types::global::GlobalInteraction;
    /// use sessrums_types::session_types::common::{RoleIdentifier, Label};
    ///
    /// // Define a protocol with a choice: client chooses between "login" and "register"
    /// let protocol = GlobalInteraction::choice(
    ///     "client",
    ///     vec![
    ///         ("login".into(), GlobalInteraction::message(
    ///             "client",
    ///             "server",
    ///             GlobalInteraction::end(),
    ///         )),
    ///         ("register".into(), GlobalInteraction::message(
    ///             "client",
    ///             "server",
    ///             GlobalInteraction::end(),
    ///         )),
    ///     ],
    /// );
    /// ```
    Choice {
        /// The role that makes the decision
        decider: RoleIdentifier,
        
        /// The available branches, each with a label and continuation
        branches: Vec<(Label, Box<GlobalInteraction<M>>)>,
    },
    
    /// Represents the termination of the protocol.
    ///
    /// When a protocol reaches the `End` state, no further interactions are expected.
    End,
}

impl<M: Clone> GlobalInteraction<M> {
    /// Creates a new message interaction in the global protocol.
    ///
    /// # Parameters
    ///
    /// * `from` - The role identifier of the sender
    /// * `to` - The role identifier of the receiver
    /// * `cont` - The continuation of the protocol after this message exchange
    ///
    /// # Returns
    ///
    /// A new `GlobalInteraction::Message` variant
    pub fn message(
        from: impl Into<RoleIdentifier>,
        to: impl Into<RoleIdentifier>,
        cont: GlobalInteraction<M>,
    ) -> Self {
        GlobalInteraction::Message {
            from: from.into(),
            to: to.into(),
            msg: PhantomData,
            cont: Box::new(cont),
        }
    }

    /// Creates a new end interaction, representing protocol termination.
    ///
    /// # Returns
    ///
    /// A new `GlobalInteraction::End` variant
    pub fn end() -> Self {
        GlobalInteraction::End
    }
    
    /// Creates a new choice interaction in the global protocol.
    ///
    /// # Parameters
    ///
    /// * `decider` - The role identifier of the participant making the choice
    /// * `branches` - A vector of labeled branches, each with a continuation
    ///
    /// # Returns
    ///
    /// A new `GlobalInteraction::Choice` variant
    pub fn choice(
        decider: impl Into<RoleIdentifier>,
        branches: Vec<(Label, GlobalInteraction<M>)>,
    ) -> Self {
        GlobalInteraction::Choice {
            decider: decider.into(),
            branches: branches
                .into_iter()
                .map(|(label, cont)| (label, Box::new(cont)))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_simple_protocol() {
        // Define a simple protocol: client sends a request to server, server responds
        let protocol: GlobalInteraction<String> = GlobalInteraction::message(
            "client",
            "server",
            GlobalInteraction::message(
                "server",
                "client",
                GlobalInteraction::end(),
            ),
        );

        // Verify the structure of the protocol
        if let GlobalInteraction::Message { from, to, cont, .. } = protocol {
            assert_eq!(from.name(), "client");
            assert_eq!(to.name(), "server");
            
            if let GlobalInteraction::Message { from, to, cont, .. } = *cont {
                assert_eq!(from.name(), "server");
                assert_eq!(to.name(), "client");
                
                assert!(matches!(*cont, GlobalInteraction::End));
            } else {
                panic!("Expected Message, got End");
            }
        } else {
            panic!("Expected Message, got End");
        }
    }

    #[test]
    fn test_create_three_party_protocol() {
        // Define a three-party protocol: client sends to server, server forwards to database, database responds to server, server responds to client
        let protocol: GlobalInteraction<String> = GlobalInteraction::message(
            "client",
            "server",
            GlobalInteraction::message(
                "server",
                "database",
                GlobalInteraction::message(
                    "database",
                    "server",
                    GlobalInteraction::message(
                        "server",
                        "client",
                        GlobalInteraction::end(),
                    ),
                ),
            ),
        );

        // Verify the first interaction
        if let GlobalInteraction::Message { from, to, cont, .. } = protocol {
            assert_eq!(from.name(), "client");
            assert_eq!(to.name(), "server");
            
            // Verify the second interaction
            if let GlobalInteraction::Message { from, to, cont, .. } = *cont {
                assert_eq!(from.name(), "server");
                assert_eq!(to.name(), "database");
                
                // Verify the third interaction
                if let GlobalInteraction::Message { from, to, cont, .. } = *cont {
                    assert_eq!(from.name(), "database");
                    assert_eq!(to.name(), "server");
                    
                    // Verify the fourth interaction
                    if let GlobalInteraction::Message { from, to, cont, .. } = *cont {
                        assert_eq!(from.name(), "server");
                        assert_eq!(to.name(), "client");
                        
                        // Verify the end
                        assert!(matches!(*cont, GlobalInteraction::End));
                    } else {
                        panic!("Expected fourth Message, got End");
                    }
                } else {
                    panic!("Expected third Message, got End");
                }
            } else {
                panic!("Expected second Message, got End");
            }
        } else {
            panic!("Expected first Message, got End");
        }
    }
}