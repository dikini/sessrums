//! Error Example 2: Deadlock (Send/Send)
//!
//! This example demonstrates a protocol that should fail to compile because
//! both client and server try to send an i32 first, which would cause a deadlock.
//!
//! # Why This Protocol Fails
//!
//! In session types, communication between two parties must be complementary to avoid deadlocks.
//! When one party sends a message, the other party must receive a message. This complementary
//! relationship is formalized through the concept of "duality".
//!
//! In this example:
//! - Client protocol: Send<i32, End> (send an i32, then end)
//! - Server protocol: Send<i32, End> (send an i32, then end)
//!
//! Both parties are trying to send a message, but neither is receiving one. This creates a
//! deadlock situation where both parties might block trying to send messages that will
//! never be received.
//!
//! # How Session Types Prevent This Error
//!
//! The session type system prevents this error at compile time by enforcing duality between
//! communicating parties. For any protocol P, there must exist a dual protocol P::Dual that
//! represents the complementary behavior.
//!
//! For Send<T, P>, the dual is Recv<T, P::Dual>. This means:
//! - The dual of Send<i32, End> is Recv<i32, End>
//! - The dual of Recv<i32, End> is Send<i32, End>
//! - The dual of End is End (termination is symmetric)
//!
//! When we try to create a channel pair with non-dual protocols, the type system rejects it,
//! preventing the deadlock at compile time.
//!
//! # Visual Representation of the Protocol
//!
//! ```text
//! Client                 Server
//!   |                      |
//!   |  !i32                |  !i32
//!   | <---- DEADLOCK ----> |
//!   |                      |
//!  End                    End
//! ```
//!
//! Legend:
//! - ?T: Receive a value of type T
//! - !T: Send a value of type T
//! - DEADLOCK: Both parties trying to send, neither receiving
//!
//! # Correct Version (For Reference)
//!
//! A correct version would have complementary actions:
//!
//! ```text
//! Client                 Server
//!   |                      |
//!   |  !i32                |  ?i32
//!   | -------------------> |
//!   |                      |
//!  End                    End
//! ```

use sez::proto::{Protocol, Send, End};
use sez::chan::Chan;

// Define the protocol types
type ClientProto = Send<i32, End>;
type ServerProto = Send<i32, End>; // Not the dual of ClientProto! Should be Recv<i32, End>

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
    // the protocols are not duals of each other.
    
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
    // 2. Server would also try to send an i32
    // 3. Depending on the underlying channel implementation:
    //    - Both might block forever if the channels are synchronous
    //    - The channels might fill up if they're buffered
    //    - The messages might be lost if no one is receiving
    
    // Fortunately, the session type system prevents this at compile time
    // by enforcing duality between the client and server protocols.
}