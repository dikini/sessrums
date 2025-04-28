//! Protocol type definitions for session types.
//!
//! This module contains the core protocol type definitions used to express
//! communication protocols at the type level.

pub mod roles;
pub use roles::{Role, RoleA, RoleB};

mod base;
pub use base::Protocol;

mod send;
pub use send::Send;

mod recv;
pub use recv::Recv;

mod end;
pub use end::End;

mod offer;
pub use offer::Offer;

mod choose;
pub use choose::Choose;

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

pub mod compat;
pub use compat::{ProtocolCompat, BinaryWrapper, MPSTWrapper};

// Re-export the global_protocol macro from the sessrums-macro crate
pub use sessrums_macro::global_protocol;