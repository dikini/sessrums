//! Protocol type definitions for session types.
//!
//! This module contains the core protocol type definitions used to express
//! communication protocols at the type level.

mod proto;
pub use proto::Protocol;

mod send;
pub use send::Send;

mod recv;
pub use recv::Recv;

mod end;
pub use end::End;

// These will be uncommented in Phase 2
// mod offer;
// pub use offer::Offer;

// mod choose;
// pub use choose::Choose;

// These will be uncommented in Phase 5
// mod rec;
// pub use rec::Rec;

// mod var;
// pub use var::Var;