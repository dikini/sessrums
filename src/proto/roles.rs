//! Defines the `Role` trait and concrete role types for MPST protocols.

/// A trait representing a role in an MPST protocol.
///
/// Each participant in a protocol is assigned a unique role. Roles are used
/// to distinguish between different participants and to define the structure
/// of the session types from each participant's perspective.
pub trait Role: Send + 'static + Default {
    /// Returns a string representation of the role.
    fn name(&self) -> &'static str;
}

/// A concrete role type, RoleA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RoleA;

impl Default for RoleA {
    fn default() -> Self {
        RoleA
    }
}

impl Role for RoleA {
    fn name(&self) -> &'static str {
        "RoleA"
    }
}

/// A concrete role type, RoleB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RoleB;

impl Default for RoleB {
    fn default() -> Self {
        RoleB
    }
}

impl Role for RoleB {
    fn name(&self) -> &'static str {
        "RoleB"
    }
}

// Add more concrete role types as needed for specific protocols.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_a_name() {
        let role_a = RoleA;
        assert_eq!(role_a.name(), "RoleA");
    }

    #[test]
    fn test_role_b_name() {
        let role_b = RoleB;
        assert_eq!(role_b.name(), "RoleB");
    }

    #[test]
    fn test_role_a_debug() {
        let role_a = RoleA;
        assert_eq!(format!("{:?}", role_a), "RoleA");
    }

    #[test]
    fn test_role_b_debug() {
        let role_b = RoleB;
        assert_eq!(format!("{:?}", role_b), "RoleB");
    }

    #[test]
    fn test_role_a_clone() {
        let role_a = RoleA;
        let cloned_role_a = role_a.clone();
        assert_eq!(role_a, cloned_role_a);
    }

    #[test]
    fn test_role_b_clone() {
        let role_b = RoleB;
        let cloned_role_b = role_b.clone();
        assert_eq!(role_b, cloned_role_b);
    }

    #[test]
    fn test_role_a_partial_eq() {
        let role_a1 = RoleA;
        let role_a2 = RoleA;
        assert_eq!(role_a1, role_a2);
    }

    #[test]
    fn test_role_b_partial_eq() {
        let role_b1 = RoleB;
        let role_b2 = RoleB;
        assert_eq!(role_b1, role_b2);
    }

    #[test]
    fn test_role_a_b_not_equal() {
        let role_a = RoleA;
        let role_b = RoleB;
        // We can't directly compare different types with assert_ne!
        // Instead, let's compare their names
        assert_ne!(role_a.name(), role_b.name());
    }
}