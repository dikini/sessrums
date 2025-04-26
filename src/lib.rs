//! # SEZ: Session Types EZ
//!
//! A Rust library for asynchronous session types with minimal dependencies,
//! focusing on expressing the process calculus in the types using Rust's type system features,
//! including `const generics`.
//!
//! ## Overview
//!
//! This library implements session types, a type discipline for communication protocols,
//! allowing compile-time verification of protocol adherence.

// Re-export all public items
pub mod proto;
pub mod chan;

// Phase 3 implementation
pub mod error;

// This will be uncommented in Phase 2
pub mod io;