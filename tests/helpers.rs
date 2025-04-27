//! Helper functions and utilities for tests.
//!
//! This module provides helper functions and utilities to make it easier to write
//! tests for the session type system.

use sez::proto::Protocol;
use sez::chan::Chan;

/// Verify that two protocols are duals of each other.
///
/// This function will fail to compile if `P::Dual != Q` or `Q::Dual != P`.
/// It's useful for testing that the session type system correctly enforces duality.
///
/// # Type Parameters
///
/// * `P` - The first protocol type
/// * `Q` - The second protocol type
/// * `IO1` - The IO type for the first channel
/// * `IO2` - The IO type for the second channel
///
/// # Examples
///
/// ```
/// use sez::proto::{Send, Recv, End};
/// use sez::chan::Chan;
/// use tests::helpers::verify_dual_protocols;
///
/// // These protocols are duals of each other
/// type Proto1 = Send<i32, End>;
/// type Proto2 = Recv<i32, End>;
///
/// let chan1 = Chan::<Proto1, _>::new(());
/// let chan2 = Chan::<Proto2, _>::new(());
///
/// // This should compile
/// verify_dual_protocols(chan1, chan2);
/// ```
pub fn verify_dual_protocols<P, Q, IO1, IO2>(_chan1: Chan<P, IO1>, _chan2: Chan<Q, IO2>) 
where 
    P: Protocol,
    Q: Protocol,
    P::Dual: Protocol<Dual = P>,
    Q: Protocol<Dual = P>  // This constraint ensures Q is the dual of P
{
    // The function body is empty because the type constraints do all the work
}

/// Assert that a type implements the Protocol trait.
///
/// This function will fail to compile if `P` does not implement the `Protocol` trait.
///
/// # Type Parameters
///
/// * `P` - The type to check
///
/// # Examples
///
/// ```
/// use sez::proto::{Send, End};
/// use tests::helpers::assert_protocol;
///
/// // This should compile
/// assert_protocol::<Send<i32, End>>();
/// ```
pub fn assert_protocol<P>() 
where 
    P: Protocol
{
    // The function body is empty because the type constraint does all the work
}

/// Assert that two types have the correct duality relationship.
///
/// This function will fail to compile if `P::Dual != Q` or `Q::Dual != P`.
///
/// # Type Parameters
///
/// * `P` - The first protocol type
/// * `Q` - The second protocol type
///
/// # Examples
///
/// ```
/// use sez::proto::{Send, Recv, End};
/// use tests::helpers::assert_dual;
///
/// // This should compile
/// assert_dual::<Send<i32, End>, Recv<i32, End>>();
/// ```
pub fn assert_dual<P, Q>() 
where 
    P: Protocol<Dual = Q>,
    Q: Protocol<Dual = P>
{
    // The function body is empty because the type constraints do all the work
}

/// Create a mock channel for testing.
///
/// This function creates a channel with a specific protocol and IO type for testing.
///
/// # Type Parameters
///
/// * `P` - The protocol type
/// * `IO` - The IO type
///
/// # Returns
///
/// A new `Chan` instance with the specified protocol type and IO implementation.
///
/// # Examples
///
/// ```
/// use sez::proto::{Send, End};
/// use tests::helpers::mock_channel;
///
/// // Create a mock channel
/// let chan = mock_channel::<Send<i32, End>, ()>();
/// ```
pub fn mock_channel<P, IO>() -> Chan<P, IO>
where 
    P: Protocol,
    IO: Default
{
    Chan::new(IO::default())
}
