//! Error Example 3: Type Mismatch
//!
//! This example demonstrates a protocol that should fail to compile because
//! the client sends an i32, but the server expects to receive a String.
//!
//! # Why This Protocol Fails
//!
//! In session types, communication between two parties must have matching types to ensure
//! type safety. When one party sends a message of type T, the other party must receive a
//! message of the same type T. This type matching is enforced through the concept of "duality".
//!
//! In this example:
//! - Client protocol: Send<i32, End> (send an i32, then end)
//! - Server protocol: Recv<String, End> (receive a String, then end)
//!
//! The client is sending an i32, but the server is expecting a String. This creates a
//! type mismatch that would cause runtime errors if allowed to compile.
//!
//! # How Session Types Prevent This Error
//!
//! The session type system prevents this error at compile time by enforcing duality between
//! communicating parties. For any protocol P, there must exist a dual protocol P::Dual that
//! represents the complementary behavior with matching types.
//!
//! For Send<T, P>, the dual is Recv<T, P::Dual> (with the same type T). This means:
//! - The dual of Send<i32, End> is Recv<i32, End> (not Recv<String, End>)
//! - The dual of Recv<String, End> is Send<String, End> (not Send<i32, End>)
//!
//! When we try to create a channel pair with non-dual protocols that have mismatched types,
//! the type system rejects it, preventing type errors at compile time.
//!
//! # Visual Representation of the Protocol
//!
//! ```text
//! Client                 Server
//!   |                      |
//!   |  !i32                |  ?String
//!   | <---- MISMATCH ----> |
//!   |                      |
//!  End                    End
//! ```
//!
//! Legend:
//! - ?T: Receive a value of type T
//! - !T: Send a value of type T
//! - MISMATCH: Type mismatch between sent and received values
//!
//! # Correct Version (For Reference)
//!
//! A correct version would have matching types:
//!
//! ```text
//! Client                 Server
//!   |                      |
//!   |  !i32                |  ?i32
//!   | -------------------> |
//!   |                      |
//!  End                    End
//! ```

use sessrums::proto::{Protocol, Send, Recv, End};
use sessrums::chan::Chan;

// Define the protocol types
type ClientProto = Send<i32, End>;
type ServerProto = Recv<String, End>; // Not the dual of ClientProto! Type mismatch: String vs i32

// This function attempts to create a pair of channels with non-dual protocols
fn create_invalid_channel_pair() {
    // This should fail to compile because ServerProto is not the dual of ClientProto
    let client_chan = Chan::<ClientProto, _>::new(());
    let server_chan = Chan::<ServerProto, _>::new(());
    
    // In a real implementation, we would use a function like:
    // let (client_chan, server_chan) = session_channel::<ClientProto, ServerProto>((), ());
    // But that would enforce duality, so we're manually creating the channels here
    // to demonstrate the error.
    
    // The type system should prevent this code from compiling because
    // the protocols are not duals of each other due to type mismatch.
    
    // This function would verify that the protocols are duals of each other
    verify_dual_protocols(client_chan, server_chan);
}

// This function verifies that two protocols are duals of each other
// It should fail to compile if P::Dual != Q
fn verify_dual_protocols<P, Q, IO1, IO2>(_chan1: Chan<P, IO1>, _chan2: Chan<Q, IO2>)
where
    P: Protocol,
    Q: Protocol,
    P::Dual: Protocol<Dual = P>,
    Q: Protocol<Dual = P>  // This is the key constraint that should fail
{
    // The function body doesn't matter, as the type constraints will cause a compile error
    // if the protocols are not duals of each other
    
    // In a real session type system, this verification would happen automatically
    // when creating a channel pair, ensuring that the two endpoints have compatible
    // protocols that won't deadlock.
}

fn main() {
    create_invalid_channel_pair();
    
    // If this code were to compile and run (which it shouldn't),
    // the following would happen at runtime:
    
    // 1. Client would try to send an i32
    // 2. Server would try to receive a String
    // 3. This would cause a type error at runtime, as the binary representation
    //    of an i32 cannot be safely interpreted as a String
    
    // Fortunately, the session type system prevents this at compile time
    // by enforcing duality with matching types between the client and server protocols.
}