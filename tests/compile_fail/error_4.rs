//! Error Example 4: Unexpected End
//!
//! This example demonstrates a protocol that should fail to compile because
//! the client sends an i32 and terminates, but the server expects to send a bool
//! after receiving the i32.
//!
//! # Why This Protocol Fails
//!
//! In session types, both parties must agree on the entire communication sequence.
//! When one party expects the session to end but the other expects it to continue,
//! this creates a protocol mismatch that would cause runtime errors if allowed to compile.
//!
//! In this example:
//! - Client protocol: Send<i32, End> (send an i32, then end)
//! - Server protocol: Recv<i32, Send<bool, End>> (receive an i32, send a bool, then end)
//!
//! After the initial i32 exchange, the client expects to terminate the session,
//! but the server expects to continue by sending a bool. This mismatch in continuation
//! protocols (End vs Send<bool, End>) would lead to a runtime error where the server
//! attempts to send data to a client that has already closed the connection.
//!
//! # How Session Types Prevent This Error
//!
//! The session type system prevents this error at compile time by enforcing duality between
//! communicating parties. For any protocol P, there must exist a dual protocol P::Dual that
//! represents the complementary behavior with matching continuations.
//!
//! For Send<T, P>, the dual is Recv<T, P::Dual>. This means:
//! - The dual of Send<i32, End> is Recv<i32, End>
//! - The dual of Recv<i32, Send<bool, End>> is Send<i32, Recv<bool, End>>
//!
//! When we try to create a channel pair with non-dual protocols that have mismatched
//! continuations, the type system rejects it, preventing protocol errors at compile time.
//!
//! # Visual Representation of the Protocol
//!
//! ```text
//! Client                 Server
//!   |                      |
//!   |  !i32                |  ?i32
//!   | -------------------> |
//!   |                      |
//!  End                     |  !bool
//!   |                      |
//!   |       MISMATCH       |
//!   |                      |
//!   X                     End
//! ```
//!
//! Legend:
//! - ?T: Receive a value of type T
//! - !T: Send a value of type T
//! - MISMATCH: Protocol continuation mismatch
//! - X: Client has ended the session
//!
//! # Correct Version (For Reference)
//!
//! A correct version would have matching continuations:
//!
//! ```text
//! Client                 Server
//!   |                      |
//!   |  !i32                |  ?i32
//!   | -------------------> |
//!   |                      |
//!   |  ?bool               |  !bool
//!   | <------------------- |
//!   |                      |
//!  End                    End
//! ```

use sez::proto::{Protocol, Send, Recv, End};
use sez::chan::Chan;

// Define the protocol types
type ClientProto = Send<i32, End>;
type ServerProto = Recv<i32, Send<bool, End>>; // Not the dual of ClientProto! Continuation mismatch

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
    // the protocols are not duals of each other due to continuation mismatch.
    
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
    
    // 1. Client would send an i32 and close its connection
    // 2. Server would receive the i32 and then try to send a bool
    // 3. This would cause a runtime error, as the client has already
    //    closed the connection and is not expecting to receive anything
    
    // Fortunately, the session type system prevents this at compile time
    // by enforcing duality with matching continuations between the client and server protocols.
}