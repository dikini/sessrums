//! # sessrums: Session Types EZ
//!
//! A Rust library for asynchronous session types with minimal dependencies,
//! focusing on expressing the process calculus in the types using Rust's type system features,
//! including `const generics`.
//!
//! ## Overview
//!
//! This library implements session types, a type discipline for communication protocols,
//! allowing compile-time verification of protocol adherence.
//!
//! ## Features
//!
//! - **Type-level Protocol Definitions**: Define communication protocols at the type level
//! - **Compile-time Protocol Verification**: Ensure protocol adherence at compile time
//! - **Asynchronous Communication**: Support for asynchronous communication
//! - **Minimal Dependencies**: Focus on using Rust's type system features
//! - **Ergonomic API**: Type aliases, helper functions, and macros for improved ergonomics
//!

// Re-export all public items
pub mod proto;
pub mod chan;
pub mod error;
pub mod io;
pub mod connect;
pub mod api;

/// Macro to define a protocol type with a more concise syntax.
///
/// This macro allows defining complex protocol types with a more readable syntax.
/// It supports the following protocol combinators:
///
/// * `send(T)` - Send a value of type T
/// * `recv(T)` - Receive a value of type T
/// * `choose(P1, P2)` - Choose between protocols P1 and P2
/// * `offer(P1, P2)` - Offer a choice between protocols P1 and P2
/// * `end` - End the protocol
///
#[macro_export]
macro_rules! protocol {
    // Base cases
    (end) => { $crate::proto::End };
    (send($t:ty) >> end) => { $crate::proto::Send<$t, $crate::proto::End> };
    (recv($t:ty) >> end) => { $crate::proto::Recv<$t, $crate::proto::End> };
    
    // Recursive cases
    (send($t:ty) >> $($rest:tt)*) => { $crate::proto::Send<$t, protocol!($($rest)*)> };
    (recv($t:ty) >> $($rest:tt)*) => { $crate::proto::Recv<$t, protocol!($($rest)*)> };
    
    // Choice and offer
    (choose($($p1:tt)*), $($p2:tt)*) => { $crate::proto::Choose<protocol!($($p1)*), protocol!($($p2)*)> };
    (offer($($p1:tt)*), $($p2:tt)*) => { $crate::proto::Offer<protocol!($($p1)*), protocol!($($p2)*)> };
}

/// Macro to create a client-server protocol pair with a more concise syntax.
///
/// This macro creates a pair of protocol types for a client and server, ensuring
/// that they are duals of each other.
///
/// ```
#[macro_export]
macro_rules! protocol_pair {
    (
        $(#[$attr:meta])*
        $vis:vis $name:ident<$($param:ident),*> {
            client: $($client:tt)*,
            server: $($server:tt)*
        }
    ) => {
        $(#[$attr])*
        $vis mod $name {
            use super::*;
            
            /// The client side of the protocol.
            pub type Client<$($param),*> = $crate::protocol!($($client)*);
            
            /// The server side of the protocol.
            pub type Server<$($param),*> = $crate::protocol!($($server)*);
            
            // Verify at compile time that the protocols are duals of each other
            #[allow(dead_code)]
            fn verify_duality<$($param: 'static),*>()
            where
                Client<$($param),*>: $crate::proto::Protocol,
                Server<$($param),*>: $crate::proto::Protocol,
            {
                fn assert_dual<P, Q>()
                where
                    P: $crate::proto::Protocol,
                    Q: $crate::proto::Protocol,
                    P::Dual: $crate::proto::Protocol<Dual = P>,
                    Q: $crate::proto::Protocol<Dual = P>,
                {}
                
                assert_dual::<Client<$($param),*>, Server<$($param),*>>();
            }
        }
    };
}