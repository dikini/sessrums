//! Compatibility layer between binary and multiparty session types.
//!
//! This module provides types and traits to ensure backward compatibility
//! between binary session types and multiparty session types (MPST).

use crate::proto::Protocol;
use crate::proto::roles::Role;
use std::marker::PhantomData;

/// A trait for converting between binary and multiparty session types.
///
/// This trait allows for seamless integration between the existing binary
/// session types and the new multiparty session types.
pub trait ProtocolCompat<R: Role> {
    /// The binary protocol type that is compatible with this multiparty protocol.
    type BinaryProtocol: Protocol;
    
    /// Converts a multiparty protocol to a binary protocol.
    fn to_binary(self) -> Self::BinaryProtocol;
    
    /// Converts a binary protocol to a multiparty protocol.
    fn from_binary(binary: Self::BinaryProtocol) -> Self;
}

/// A wrapper for binary session types to make them compatible with MPST.
///
/// This wrapper allows existing binary session types to be used in MPST contexts.
pub struct BinaryWrapper<P: Protocol, R: Role>(P, PhantomData<R>);

impl<P: Protocol, R: Role> BinaryWrapper<P, R> {
    /// Creates a new binary wrapper for the given protocol and role.
    pub fn new(protocol: P) -> Self {
        BinaryWrapper(protocol, PhantomData)
    }
    
    /// Unwraps the binary protocol.
    pub fn unwrap(self) -> P {
        self.0
    }
}

impl<P: Protocol, R: Role> Protocol for BinaryWrapper<P, R> {
    type Dual = BinaryWrapper<P::Dual, R>;
}

/// A wrapper for MPST local types to make them compatible with binary session types.
///
/// This wrapper allows MPST local types to be used in binary session type contexts.
pub struct MPSTWrapper<P, R: Role>(P, PhantomData<R>);

impl<P, R: Role> MPSTWrapper<P, R> {
    /// Creates a new MPST wrapper for the given protocol and role.
    pub fn new(protocol: P) -> Self {
        MPSTWrapper(protocol, PhantomData)
    }
    
    /// Unwraps the MPST local protocol.
    pub fn unwrap(self) -> P {
        self.0
    }
}

impl<P, R: Role> Protocol for MPSTWrapper<P, R>
where
    P: 'static
{
    type Dual = MPSTWrapper<P, R>;
}

// Implement ProtocolCompat for common protocol types
// These implementations allow for seamless conversion between binary and multiparty protocols

// Implementation for Send
impl<T, P: Protocol, R: Role> ProtocolCompat<R> for crate::proto::send::Send<T, P> {
    type BinaryProtocol = Self;
    
    fn to_binary(self) -> Self::BinaryProtocol {
        self
    }
    
    fn from_binary(binary: Self::BinaryProtocol) -> Self {
        binary
    }
}

// Implementation for Recv
impl<T, P: Protocol, R: Role> ProtocolCompat<R> for crate::proto::recv::Recv<T, P> {
    type BinaryProtocol = Self;
    
    fn to_binary(self) -> Self::BinaryProtocol {
        self
    }
    
    fn from_binary(binary: Self::BinaryProtocol) -> Self {
        binary
    }
}

// Implementation for End
impl<R: Role> ProtocolCompat<R> for crate::proto::end::End {
    type BinaryProtocol = Self;
    
    fn to_binary(self) -> Self::BinaryProtocol {
        self
    }
    
    fn from_binary(binary: Self::BinaryProtocol) -> Self {
        binary
    }
}

// Implementation for Choose
impl<L: Protocol, R1: Protocol, R2: Role> ProtocolCompat<R2> for crate::proto::choose::Choose<L, R1> {
    type BinaryProtocol = Self;
    
    fn to_binary(self) -> Self::BinaryProtocol {
        self
    }
    
    fn from_binary(binary: Self::BinaryProtocol) -> Self {
        binary
    }
}

// Implementation for Offer
impl<L: Protocol, R1: Protocol, R2: Role> ProtocolCompat<R2> for crate::proto::offer::Offer<L, R1> {
    type BinaryProtocol = Self;
    
    fn to_binary(self) -> Self::BinaryProtocol {
        self
    }
    
    fn from_binary(binary: Self::BinaryProtocol) -> Self {
        binary
    }
}

// Implementation for Rec
impl<P: Protocol, R: Role> ProtocolCompat<R> for crate::proto::rec::Rec<P> {
    type BinaryProtocol = Self;
    
    fn to_binary(self) -> Self::BinaryProtocol {
        self
    }
    
    fn from_binary(binary: Self::BinaryProtocol) -> Self {
        binary
    }
}

// Implementation for Var
impl<const N: usize, R: Role> ProtocolCompat<R> for crate::proto::var::Var<N> {
    type BinaryProtocol = Self;
    
    fn to_binary(self) -> Self::BinaryProtocol {
        self
    }
    
    fn from_binary(binary: Self::BinaryProtocol) -> Self {
        binary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::send::Send;
    use crate::proto::recv::Recv;
    use crate::proto::end::End;
    use crate::proto::roles::{RoleA, RoleB};
    
    #[test]
    fn test_binary_wrapper() {
        // Create a binary protocol
        type BinaryProtocol = Send<i32, Recv<String, End>>;
        
        // Wrap it for use with MPST
        let wrapped = BinaryWrapper::<BinaryProtocol, RoleA>::new(Default::default());
        
        // NOTE: protocol_name assertion removed as it's not part of the wrapper API
    }
    
    #[test]
    fn test_mpst_wrapper() {
        // Create an MPST local protocol (using binary types for simplicity in this test)
        type MPSTLocalProtocol = Send<i32, Recv<String, End>>;
        
        // Wrap it for use with binary session types
        let wrapped = MPSTWrapper::<MPSTLocalProtocol, RoleA>::new(Default::default());
        
        // NOTE: protocol_name assertion removed as it's not part of the wrapper API
    }
    
    #[test]
    fn test_protocol_compat() {
        // Test conversion for Send
        let send_protocol: Send<i32, End> = Default::default();
        let binary = <Send<i32, End> as ProtocolCompat<RoleA>>::to_binary(send_protocol); // Specify RoleA
        let _mpst = <Send<i32, End> as ProtocolCompat<RoleA>>::from_binary(binary); // Specify RoleA
        
        // Test conversion for Recv
        let recv_protocol: Recv<String, End> = Default::default();
        let binary = <Recv<String, End> as ProtocolCompat<RoleA>>::to_binary(recv_protocol); // Specify RoleA
        let _mpst = <Recv<String, End> as ProtocolCompat<RoleA>>::from_binary(binary); // Specify RoleA
        
        // Test conversion for End
        let end_protocol: End = Default::default(); // End is an empty struct, Default works
        let binary = <End as ProtocolCompat<RoleA>>::to_binary(end_protocol); // Specify RoleA
        let _mpst = <End as ProtocolCompat<RoleA>>::from_binary(binary); // Specify RoleA, assign to _mpst
    }
}