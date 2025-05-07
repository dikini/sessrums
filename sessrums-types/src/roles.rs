//! Role definitions for session type protocols.
//! 
//! Roles are implemented as zero-sized types (ZSTs) to provide type-level
//! identification of protocol participants without runtime overhead.

/// Trait marking types that can act as roles in a protocol.
/// 
/// This trait is sealed and cannot be implemented outside this crate
/// to ensure role type safety.
pub trait Role: private::Sealed {}

/// Client role in a client-server protocol.
#[derive(Debug, Clone, Copy)]
pub struct Client;

/// Server role in a client-server protocol.
#[derive(Debug, Clone, Copy)]
pub struct Server;

// Implement Role for our basic roles
impl Role for Client {}
impl Role for Server {}

// Private sealed trait to prevent external implementations
mod private {
    pub trait Sealed {}
    impl Sealed for super::Client {}
    impl Sealed for super::Server {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_are_copy() {
        let client = Client;
        let _client_copy = client;
        // Original client still usable
        let _also_client = client;
        
        let server = Server;
        let _server_copy = server;
        // Original server still usable
        let _also_server = server;
    }
}