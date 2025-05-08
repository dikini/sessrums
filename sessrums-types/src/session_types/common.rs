//! Common structures for multiparty session types.
//!
//! This module provides the foundational types needed to bridge compile-time
//! role types with runtime role identifiers in multiparty session type protocols.

use crate::roles::Role;
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;
use std::borrow::Cow;

/// A runtime identifier for a protocol role.
///
/// While the role types (implementing the `Role` trait) provide compile-time
/// type safety, `RoleIdentifier` provides a way to identify roles at runtime
/// with string-based names. This is particularly important in multiparty protocols
/// where participants need to be identified dynamically.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoleIdentifier(pub String);

impl RoleIdentifier {
    /// Creates a new role identifier with the given name.
    pub fn new<S: Into<String>>(name: S) -> Self {
        RoleIdentifier(name.into())
    }

    /// Returns the name of this role identifier as a string slice.
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl Display for RoleIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RoleIdentifier {
    fn from(name: String) -> Self {
        RoleIdentifier(name)
    }
}

impl From<&str> for RoleIdentifier {
    fn from(name: &str) -> Self {
        RoleIdentifier(name.to_string())
    }
}

/// A label for protocol branches in choice constructs.
///
/// Labels are used to identify different branches in choice constructs,
/// allowing participants to select between alternative protocol continuations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(pub String);

impl Label {
    /// Creates a new label with the given name.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Label(name.into())
    }

    /// Returns the name of this label as a string slice.
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Label {
    fn from(name: String) -> Self {
        Label(name)
    }
}

impl From<&str> for Label {
    fn from(name: &str) -> Self {
        Label(name.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for Label {
    fn from(name: Cow<'a, str>) -> Self {
        Label(name.into_owned())
    }
}

/// A label for recursion points in recursive protocols.
///
/// Recursion labels are used to identify recursion points in protocols,
/// allowing for the definition of loops and repeated interactions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecursionLabel(pub String);

impl RecursionLabel {
    /// Creates a new recursion label with the given name.
    pub fn new<S: Into<String>>(name: S) -> Self {
        RecursionLabel(name.into())
    }

    /// Returns the name of this recursion label as a string slice.
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl Display for RecursionLabel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RecursionLabel {
    fn from(name: String) -> Self {
        RecursionLabel(name)
    }
}

impl From<&str> for RecursionLabel {
    fn from(name: &str) -> Self {
        RecursionLabel(name.to_string())
    }
}

/// Associates a runtime role identifier with a compile-time role type.
///
/// `Participant` bridges the gap between the compile-time type system (where roles
/// are represented as zero-sized types implementing the `Role` trait) and runtime
/// identification (where roles need string identifiers for dynamic dispatch and
/// network communication).
///
/// The type parameter `R` represents the compile-time role type, while the
/// `identifier` field holds the runtime name of the role.
///
/// # Examples
///
/// ```
/// use sessrums_types::roles::Client;
/// use sessrums_types::session_types::common::{Participant, RoleIdentifier};
///
/// // Create a participant with the Client role type and "client" identifier
/// let client = Participant::<Client>::new("client");
/// ```
#[derive(Debug, Clone)]
pub struct Participant<R: Role> {
    /// The runtime identifier for this participant
    pub identifier: RoleIdentifier,
    
    /// Phantom data to associate with the compile-time role type
    _role: PhantomData<R>,
}

impl<R: Role> Participant<R> {
    /// Creates a new participant with the given role identifier.
    pub fn new<S: Into<RoleIdentifier>>(identifier: S) -> Self {
        Participant {
            identifier: identifier.into(),
            _role: PhantomData,
        }
    }
    
    /// Returns the role identifier for this participant.
    pub fn identifier(&self) -> &RoleIdentifier {
        &self.identifier
    }
}

impl<R: Role> Display for Participant<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Participant({})", self.identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roles::{Client, Server};

    #[test]
    fn test_role_identifier() {
        let id1 = RoleIdentifier::new("client");
        let id2 = RoleIdentifier::from("client");
        let id3: RoleIdentifier = "client".into();
        
        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
        assert_eq!(id1.name(), "client");
    }

    #[test]
    fn test_participant() {
        let client = Participant::<Client>::new("client");
        let server = Participant::<Server>::new("server");
        
        assert_eq!(client.identifier.name(), "client");
        assert_eq!(server.identifier.name(), "server");
    }
    
    #[test]
    fn test_recursion_label() {
        let label1 = RecursionLabel::new("loop");
        let label2 = RecursionLabel::from("loop");
        let label3: RecursionLabel = "loop".into();
        
        assert_eq!(label1, label2);
        assert_eq!(label2, label3);
        assert_eq!(label1.name(), "loop");
    }
}

#[cfg(test)]
mod label_tests {
    use super::*;

    #[test]
    fn test_label_creation() {
        let label1 = Label::new("option1");
        let label2 = Label::from("option1");
        let label3: Label = "option1".into();
        
        assert_eq!(label1, label2);
        assert_eq!(label2, label3);
        assert_eq!(label1.name(), "option1");
    }
}