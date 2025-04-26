//! Error Example 1: Deadlock (Recv/Recv)
//!
//! This example demonstrates a protocol that should fail to compile because
//! both client and server try to receive an i32 first, which would cause a deadlock.
//!
//! The session type system should prevent this at compile time by enforcing duality.

use std::marker::PhantomData;
use sez::proto::{Protocol, Recv, End};
use sez::chan::Chan;

// Define the protocol types
type ClientProto = Recv<i32, End>;
type ServerProto = Recv<i32, End>; // Not the dual of ClientProto!

// This function attempts to create a pair of channels with non-dual protocols
fn create_invalid_channel_pair() {
    // This should fail to compile because ServerProto is not the dual of ClientProto
    let client_chan: Chan<ClientProto, ()> = Chan {
        io: (),
        _phantom_p: PhantomData,
    };
    
    let server_chan: Chan<ServerProto, ()> = Chan {
        io: (),
        _phantom_p: PhantomData,
    };
    
    // In a real implementation, we would use a function like:
    // let (client_chan, server_chan) = session_channel((), ());
    // But that would enforce duality, so we're manually creating the channels here
    // to demonstrate the error.
    
    // The type system should prevent this code from compiling because
    // the protocols are not duals of each other.
}

fn main() {
    create_invalid_channel_pair();
}