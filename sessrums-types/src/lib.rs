//! Sessrums: Session Types for Rust
//! 
//! This library implements multiparty session types for static verification
//! of communication protocols in Rust.

pub mod roles;
pub mod messages;
pub mod error;
pub mod transport;
pub mod session_types;
pub mod projection;

// Re-export commonly used types
pub use error::SessionError;
pub use transport::Transport;
pub use projection::{Project, project_for_role, project_for_all_roles};