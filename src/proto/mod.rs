//! Protocol type definitions for session types.
//!
//! This module contains the core protocol type definitions used to express
//! communication protocols at the type level.

pub mod roles;
pub use roles::{Role, RoleA, RoleB};

mod proto;
pub use proto::Protocol;

mod send;
pub use send::Send;

mod recv;
pub use recv::Recv;

mod end;
pub use end::End;

// Phase 2 implementations
mod offer;
pub use offer::Offer;

mod choose;
// Choose is now fully implemented in Task 2.4
pub use choose::Choose;

// These will be uncommented in Phase 5
mod rec;
pub use rec::Rec;

mod var;
pub use var::Var;

pub mod global;
pub use global::{
    GlobalProtocol, GSend, GRecv, GChoice, GOffer, GRec, GVar, GEnd, GSeq, GPar,
    GlobalProtocolBuilder, validate_global_protocol
};

pub mod projection;
pub use projection::{Project, project};

// Phase 5 implementations
pub mod compat;
pub use compat::{ProtocolCompat, BinaryWrapper, MPSTWrapper};

// Re-export the global_protocol macro from the sessrums-macro crate
pub use sessrums_macro::global_protocol;