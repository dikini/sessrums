//! Error Example 1: Deadlock (Recv/Recv)
//!
//! This example demonstrates a protocol that should fail to compile because
//! both client and server try to receive an i32 first, which would cause a deadlock.
//!
//! # Why This Protocol Fails
//!
//! In session types, communication between two parties must be complementary to avoid deadlocks.
//! When one party receives a message, the other party must send a message. This complementary
//! relationship is formalized through the concept of "duality".
//!
//! In this example:
//! - Client protocol: Recv<i32, End> (receive an i32, then end)
//! - Server protocol: Recv<i32, End> (receive an i32, then end)
//!
//! Both parties are waiting to receive a message, but neither is sending one. This creates a
//! deadlock situation where both parties are blocked forever waiting for messages that will
//! never arrive.
//!
//! # How Session Types Prevent This Error
//!
//! The session type system prevents this error at compile time by enforcing duality between
//! communicating parties. For any protocol P, there must exist a dual protocol P::Dual that
//! represents the complementary behavior.
//!
//! For Recv<T, P>, the dual is Send<T, P::Dual>. This means:
//! - The dual of Recv<i32, End> is Send<i32, End>
//! - The dual of Send<i32, End> is Recv<i32, End>
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
//!   |  ?i32                |  ?i32
//!   | <---- DEADLOCK ----> |
//!   |                      |
//!  End                    End
//! ```
//!
//! Legend:
//! - ?T: Receive a value of type T
//! - !T: Send a value of type T
//! - DEADLOCK: Both parties waiting to receive, neither sending
//!
//! # Correct Version (For Reference)
//!
//! A correct version would have complementary actions:
//!
//! ```text
//! Client                 Server
//!   |                      |
//!   |  ?i32                |  !i32
//!   | <------------------- |
//!   |                      |
//!  End                    End
//! ```

use sessrums::proto::{Protocol, Recv, End};
use sessrums::chan::Chan;

// Define the protocol types
type ClientProto = Recv<i32, End>;
type ServerProto = Recv<i32, End>; // Not the dual of ClientProto! Should be Send<i32, End>

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
    
    // 1. Client would block waiting to receive an i32
    // 2. Server would also block waiting to receive an i32
    // 3. Neither would ever proceed, resulting in a deadlock
    
    // Fortunately, the session type system prevents this at compile time
    // by enforcing duality between the client and server protocols.
}