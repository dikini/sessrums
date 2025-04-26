//! Error Example 1: Deadlock (Recv/Recv)
//!
//! This example demonstrates a protocol that should fail to compile because
//! both client and server try to receive an i32 first, which would cause a deadlock.
//!
//! The session type system should prevent this at compile time by enforcing duality.

use sez::proto::{Protocol, Recv, End};
use sez::chan::Chan;

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
}

fn main() {
    create_invalid_channel_pair();
}