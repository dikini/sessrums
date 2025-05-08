//! Local protocol representation for multiparty session types.
//!
//! This module provides types for representing the local view of a multiparty
//! session type protocol, where interactions are described from the perspective
//! of a single participant.

use std::marker::PhantomData;
use crate::roles::Role;
use super::common::{RoleIdentifier, Label, RecursionLabel};

/// Represents a local protocol in a multiparty session type system.
///
/// `LocalProtocol<R>` provides a way to describe the local view of a protocol
/// from the perspective of a single participant (role `R`). This is in contrast
/// to the global view, where all interactions between participants are specified
/// from a centralized perspective.
///
/// The local view is essential for:
/// 1. Implementing endpoint behavior for a specific participant
/// 2. Type-checking communication actions against the protocol specification
/// 3. Runtime monitoring and enforcement of protocol compliance
/// 4. Ensuring that each participant follows their prescribed role in the protocol
///
/// Local protocols are typically derived from global protocols through a process
/// called projection, which extracts the relevant communication actions for a
/// specific role from the global specification.
///
/// # Type Parameters
///
/// * `R` - A type parameter implementing the `Role` trait, representing the role
///         that this local protocol belongs to
/// * `M` - A generic type parameter representing the message type (used in `Send` and `Receive`)
///
/// # Variants
///
/// * `Send` - Represents sending a message to another role
/// * `Receive` - Represents receiving a message from another role
/// * `End` - Represents the termination of the protocol
///
/// # Examples
///
/// ```
/// use std::marker::PhantomData;
/// use sessrums_types::roles::{Client, Server};
/// use sessrums_types::session_types::local::LocalProtocol;
/// use sessrums_types::session_types::common::RoleIdentifier;
///
/// // Define a simple protocol for the Client role: client sends a request to server, then receives a response
/// let client_protocol: LocalProtocol<Client, String> = LocalProtocol::<Client, String>::send(
///     "server",
///     LocalProtocol::<Client, String>::receive(
///         "server",
///         LocalProtocol::<Client, String>::end()
///     )
/// );
///
/// // Define the corresponding protocol for the Server role: server receives a request from client, then sends a response
/// let server_protocol: LocalProtocol<Server, String> = LocalProtocol::<Server, String>::receive(
///     "client",
///     LocalProtocol::<Server, String>::send(
///         "client",
///         LocalProtocol::<Server, String>::end()
///     )
/// );
/// ```
#[derive(Debug, Clone)]
pub enum LocalProtocol<R: Role, M: Clone> {
    /// Represents sending a message to another role.
    ///
    /// # Fields
    ///
    /// * `to` - The role identifier of the receiver
    /// * `msg` - A phantom type representing the message type
    /// * `cont` - The continuation of the protocol after this message is sent
    Send {
        /// The role identifier of the receiver
        to: RoleIdentifier,
        
        /// A phantom type representing the message type
        msg: PhantomData<M>,
        
        /// The continuation of the protocol after this message is sent
        cont: Box<LocalProtocol<R, M>>,
        
        /// Phantom data to associate with the compile-time role type
        _role: PhantomData<R>,
    },
    
    /// Represents receiving a message from another role.
    ///
    /// # Fields
    ///
    /// * `from` - The role identifier of the sender
    /// * `msg` - A phantom type representing the message type
    /// * `cont` - The continuation of the protocol after this message is received
    Receive {
        /// The role identifier of the sender
        from: RoleIdentifier,
        
        /// A phantom type representing the message type
        msg: PhantomData<M>,
        
        /// The continuation of the protocol after this message is received
        cont: Box<LocalProtocol<R, M>>,
        
        /// Phantom data to associate with the compile-time role type
        _role: PhantomData<R>,
    },
    
    /// Represents selecting one of multiple branches (used by the deciding role).
    ///
    /// # Fields
    ///
    /// * `branches` - The available branches, each with a label and continuation
    ///
    /// # Examples
    ///
    /// ```
    /// use std::marker::PhantomData;
    /// use sessrums_types::roles::Client;
    /// use sessrums_types::session_types::local::LocalProtocol;
    /// use sessrums_types::session_types::common::Label;
    ///
    /// // Client selects between "login" and "register"
    /// let protocol = LocalProtocol::<Client, String>::select(
    ///     vec![
    ///         ("login".into(), LocalProtocol::<Client, String>::send(
    ///             "server",
    ///             LocalProtocol::<Client, String>::end(),
    ///         )),
    ///         ("register".into(), LocalProtocol::<Client, String>::send(
    ///             "server",
    ///             LocalProtocol::<Client, String>::end(),
    ///         )),
    ///     ],
    /// );
    /// ```
    Select {
        /// The available branches, each with a label and continuation
        branches: Vec<(Label, Box<LocalProtocol<R, M>>)>,
        
        /// Phantom data to associate with the compile-time role type
        _role: PhantomData<R>,
    },
    
    /// Represents offering multiple branches for selection by another role.
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
    /// use sessrums_types::roles::Server;
    /// use sessrums_types::session_types::local::LocalProtocol;
    /// use sessrums_types::session_types::common::Label;
    ///
    /// // Server offers "login" and "register" branches for client to select
    /// let protocol = LocalProtocol::<Server, String>::offer(
    ///     "client",
    ///     vec![
    ///         ("login".into(), LocalProtocol::<Server, String>::receive(
    ///             "client",
    ///             LocalProtocol::<Server, String>::end(),
    ///         )),
    ///         ("register".into(), LocalProtocol::<Server, String>::receive(
    ///             "client",
    ///             LocalProtocol::<Server, String>::end(),
    ///         )),
    ///     ],
    /// );
    /// ```
    Offer {
        /// The role that makes the decision
        decider: RoleIdentifier,
        
        /// The available branches, each with a label and continuation
        branches: Vec<(Label, Box<LocalProtocol<R, M>>)>,
        
        /// Phantom data to associate with the compile-time role type
        _role: PhantomData<R>,
    },
    
    /// Recursion point in the local protocol.
    ///
    /// # Fields
    ///
    /// * `label` - The label identifying this recursion point
    /// * `body` - The body of the recursive protocol
    ///
    /// # Examples
    ///
    /// ```
    /// use std::marker::PhantomData;
    /// use sessrums_types::roles::Client;
    /// use sessrums_types::session_types::local::LocalProtocol;
    /// use sessrums_types::session_types::common::RecursionLabel;
    ///
    /// // Define a recursive ping-pong protocol for the client
    /// let protocol = LocalProtocol::<Client, String>::rec(
    ///     "loop",
    ///     LocalProtocol::<Client, String>::send(
    ///         "server",
    ///         LocalProtocol::<Client, String>::receive(
    ///             "server",
    ///             LocalProtocol::<Client, String>::var("loop"),
    ///         ),
    ///     ),
    /// );
    /// ```
    Rec {
        /// The label identifying this recursion point
        label: RecursionLabel,
        
        /// The body of the recursive protocol
        body: Box<LocalProtocol<R, M>>,
        
        /// Phantom data to associate with the compile-time role type
        _role: PhantomData<R>,
    },
    
    /// Reference to a recursion point.
    ///
    /// # Fields
    ///
    /// * `label` - The label of the referenced recursion point
    Var {
        /// The label of the referenced recursion point
        label: RecursionLabel,
        
        /// Phantom data to associate with the compile-time role type
        _role: PhantomData<R>,
    },
    
    /// Represents the termination of the protocol.
    ///
    /// When a protocol reaches the `End` state, no further interactions are expected.
    End {
        /// Phantom data to associate with the compile-time role type
        _role: PhantomData<R>,
    },
}

impl<R: Role, M: Clone> LocalProtocol<R, M> {
    /// Creates a new send interaction in the local protocol.
    ///
    /// # Parameters
    ///
    /// * `to` - The role identifier of the receiver
    /// * `cont` - The continuation of the protocol after this message is sent
    ///
    /// # Returns
    ///
    /// A new `LocalProtocol::Send` variant
    pub fn send(
        to: impl Into<RoleIdentifier>,
        cont: LocalProtocol<R, M>,
    ) -> Self {
        LocalProtocol::Send {
            to: to.into(),
            msg: PhantomData,
            cont: Box::new(cont),
            _role: PhantomData,
        }
    }

    /// Creates a new receive interaction in the local protocol.
    ///
    /// # Parameters
    ///
    /// * `from` - The role identifier of the sender
    /// * `cont` - The continuation of the protocol after this message is received
    ///
    /// # Returns
    ///
    /// A new `LocalProtocol::Receive` variant
    pub fn receive(
        from: impl Into<RoleIdentifier>,
        cont: LocalProtocol<R, M>,
    ) -> Self {
        LocalProtocol::Receive {
            from: from.into(),
            msg: PhantomData,
            cont: Box::new(cont),
            _role: PhantomData,
        }
    }

    /// Creates a new end interaction, representing protocol termination.
    ///
    /// # Returns
    ///
    /// A new `LocalProtocol::End` variant
    pub fn end() -> Self {
        LocalProtocol::End {
            _role: PhantomData,
        }
    }
    
    /// Creates a new select interaction in the local protocol.
    ///
    /// # Parameters
    ///
    /// * `branches` - A vector of labeled branches, each with a continuation
    ///
    /// # Returns
    ///
    /// A new `LocalProtocol::Select` variant
    pub fn select(
        branches: Vec<(Label, LocalProtocol<R, M>)>,
    ) -> Self {
        LocalProtocol::Select {
            branches: branches
                .into_iter()
                .map(|(label, cont)| (label, Box::new(cont)))
                .collect(),
            _role: PhantomData,
        }
    }
    
    /// Creates a new offer interaction in the local protocol.
    ///
    /// # Parameters
    ///
    /// * `decider` - The role identifier of the participant making the choice
    /// * `branches` - A vector of labeled branches, each with a continuation
    ///
    /// # Returns
    ///
    /// A new `LocalProtocol::Offer` variant
    pub fn offer(
        decider: impl Into<RoleIdentifier>,
        branches: Vec<(Label, LocalProtocol<R, M>)>,
    ) -> Self {
        LocalProtocol::Offer {
            decider: decider.into(),
            branches: branches
                .into_iter()
                .map(|(label, cont)| (label, Box::new(cont)))
                .collect(),
            _role: PhantomData,
        }
    }
    
    /// Creates a new recursion point in the local protocol.
    ///
    /// # Parameters
    ///
    /// * `label` - The label identifying this recursion point
    /// * `body` - The body of the recursive protocol
    ///
    /// # Returns
    ///
    /// A new `LocalProtocol::Rec` variant
    pub fn rec(
        label: impl Into<RecursionLabel>,
        body: LocalProtocol<R, M>,
    ) -> Self {
        LocalProtocol::Rec {
            label: label.into(),
            body: Box::new(body),
            _role: PhantomData,
        }
    }
    
    /// Creates a new reference to a recursion point.
    ///
    /// # Parameters
    ///
    /// * `label` - The label of the referenced recursion point
    ///
    /// # Returns
    ///
    /// A new `LocalProtocol::Var` variant
    pub fn var(label: impl Into<RecursionLabel>) -> Self {
        LocalProtocol::Var {
            label: label.into(),
            _role: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roles::{Client, Server};

    #[test]
    fn test_create_client_protocol() {
        // Define a simple protocol for the Client role: client sends a request to server, then receives a response
        let protocol: LocalProtocol<Client, String> = LocalProtocol::<Client, String>::send(
            "server",
            LocalProtocol::<Client, String>::receive(
                "server",
                LocalProtocol::<Client, String>::end(),
            ),
        );

        // Verify the structure of the protocol
        if let LocalProtocol::Send { to, cont, .. } = protocol {
            assert_eq!(to.name(), "server");
            
            if let LocalProtocol::Receive { from, cont, .. } = *cont {
                assert_eq!(from.name(), "server");
                
                assert!(matches!(*cont, LocalProtocol::End { .. }));
            } else {
                panic!("Expected Receive, got something else");
            }
        } else {
            panic!("Expected Send, got something else");
        }
    }

    #[test]
    fn test_create_server_protocol() {
        // Define the corresponding protocol for the Server role: server receives a request from client, then sends a response
        let protocol: LocalProtocol<Server, String> = LocalProtocol::<Server, String>::receive(
            "client",
            LocalProtocol::<Server, String>::send(
                "client",
                LocalProtocol::<Server, String>::end(),
            ),
        );

        // Verify the structure of the protocol
        if let LocalProtocol::Receive { from, cont, .. } = protocol {
            assert_eq!(from.name(), "client");
            
            if let LocalProtocol::Send { to, cont, .. } = *cont {
                assert_eq!(to.name(), "client");
                
                assert!(matches!(*cont, LocalProtocol::End { .. }));
            } else {
                panic!("Expected Send, got something else");
            }
        } else {
            panic!("Expected Receive, got something else");
        }
    }

    #[test]
    fn test_create_three_party_client_protocol() {
        // Define a three-party protocol from the Client's perspective:
        // 1. Client sends to Server
        // 2. Client receives from Server
        let protocol: LocalProtocol<Client, String> = LocalProtocol::<Client, String>::send(
            "server",
            LocalProtocol::<Client, String>::receive(
                "server",
                LocalProtocol::<Client, String>::end(),
            ),
        );

        // Verify the first interaction
        if let LocalProtocol::Send { to, cont, .. } = protocol {
            assert_eq!(to.name(), "server");
            
            // Verify the second interaction
            if let LocalProtocol::Receive { from, cont, .. } = *cont {
                assert_eq!(from.name(), "server");
                
                // Verify the end
                assert!(matches!(*cont, LocalProtocol::End { .. }));
            } else {
                panic!("Expected Receive, got something else");
            }
        } else {
            panic!("Expected Send, got something else");
        }
    }
}