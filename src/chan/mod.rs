//! Channel implementations for session types.
//!
//! This module contains the channel implementations that allow
//! communication according to the protocols defined in the `proto` module.
//! It supports both binary session types and multiparty session types (MPST).

use std::marker::PhantomData;
use crate::proto::Protocol;
use crate::proto::roles::Role;
use crate::proto::compat::ProtocolCompat;

/// A channel with protocol `P` and underlying IO implementation `IO`.
///
/// The `Chan` type represents a communication channel that follows protocol `P`.
/// The `IO` type parameter represents the underlying communication primitive.
///
/// # Type Parameters
///
/// * `P` - The protocol type that this channel follows. Must implement the `Protocol` trait.
/// * `IO` - The underlying IO implementation that handles the actual communication.
///
/// # Examples
///
/// Creating a channel with a simple protocol:
///
///
/// Using the channel with custom IO implementations:
///
/// A channel with protocol `P`, role `R`, and underlying IO implementation `IO`.
///
/// The `Chan` type represents a communication channel that follows protocol `P`
/// from the perspective of role `R`. The `IO` type parameter represents the
/// underlying communication primitive.
///
/// This channel can be used with both binary session types and multiparty session types (MPST).
/// For MPST, the protocol `P` is typically the result of projecting a global protocol
/// for the role `R`.
///
/// # Type Parameters
///
/// * `P` - The protocol type that this channel follows. Must implement the `Protocol` trait.
/// * `R` - The role that this channel represents in the protocol. Must implement the `Role` trait.
/// * `IO` - The underlying IO implementation that handles the actual communication.
///
/// # Examples
///
/// Creating a channel with a simple protocol:
///
///
/// Using with MPST:
///
pub struct Chan<P: Protocol, R: Role, IO> {
    /// The underlying IO implementation.
    io: IO,
    /// The role that this channel represents.
    role: R,
    /// Phantom data to carry the protocol type.
    _marker: PhantomData<P>,
}

impl<P: Protocol, R: Role, IO> Chan<P, R, IO> {
    /// Create a new channel with the given IO implementation.
    ///
    /// # Parameters
    ///
    /// * `io` - The IO implementation to use for communication.
    ///
    /// # Returns
    ///
    /// A new `Chan` instance with the specified protocol type and IO implementation.
    ///
    /// # Examples
    ///
    /// Create a new channel with the given IO implementation and role.
    ///
    /// # Parameters
    ///
    /// * `io` - The IO implementation to use for communication.
    ///
    /// # Returns
    ///
    /// A new `Chan` instance with the specified protocol type, role, and IO implementation.
    ///
    /// # Examples
    ///
    pub fn new(io: IO) -> Self {
        Chan {
            io,
            role: R::default(),
            _marker: PhantomData,
        }
    }
    
    /// Get a reference to the role that this channel represents.
    ///
    /// # Returns
    ///
    /// A reference to the role that this channel represents.
    ///
    /// # Examples
    ///
    pub fn role(&self) -> &R {
        &self.role
    }

    /// Get a reference to the underlying IO implementation.
    ///
    /// # Returns
    ///
    /// A reference to the underlying IO implementation.
    ///
    /// # Examples
    ///
    pub fn io(&self) -> &IO {
        &self.io
    }

    /// Get a mutable reference to the underlying IO implementation.
    ///
    /// # Returns
    ///
    /// A mutable reference to the underlying IO implementation.
    ///
    /// # Examples
    ///
    pub fn io_mut(&mut self) -> &mut IO {
        &mut self.io
    }
    
    /// Create a new channel with the given IO implementation and a specific role instance.
    ///
    /// This is useful when you need to use a custom role instance rather than the default.
    ///
    /// # Parameters
    ///
    /// * `io` - The IO implementation to use for communication.
    /// * `role` - The specific role instance to use.
    ///
    /// # Returns
    ///
    /// A new `Chan` instance with the specified protocol type, role, and IO implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::chan::Chan;
    /// use sessrums::proto::{Protocol, End, RoleA};
    /// use std::sync::mpsc;
    ///
    /// // Create a channel with mpsc::Sender as the IO implementation and a specific role
    /// let (tx, _) = mpsc::channel::<i32>();
    /// let role = RoleA;
    /// let chan = Chan::<End, RoleA, _>::with_role(tx, role);
    /// ```
    pub fn with_role(io: IO, role: R) -> Self {
        Chan {
            io,
            role,
            _marker: PhantomData,
        }
    }
    
    /// Convert this channel to use a different protocol type.
    ///
    /// This is useful for converting between binary and multiparty session types.
    ///
    /// # Type Parameters
    ///
    /// * `Q` - The new protocol type.
    ///
    /// # Returns
    ///
    /// A new `Chan` instance with the specified protocol type.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::chan::Chan;
    /// use sessrums::proto::{Protocol, End, Send, RoleA};
    /// use sessrums::proto::compat::{BinaryWrapper, MPSTWrapper};
    /// use std::sync::mpsc;
    ///
    /// // Create a channel with a binary protocol
    /// let (tx, _) = mpsc::channel::<i32>();
    /// let chan = Chan::<Send<i32, End>, RoleA, _>::new(tx);
    ///
    /// // Convert it to use an MPST wrapper
    /// let mpst_chan = chan.convert::<MPSTWrapper<Send<i32, End>, RoleA>>();
    /// ```
    pub fn convert<Q: Protocol>(self) -> Chan<Q, R, IO> {
        Chan {
            io: self.io,
            role: self.role,
            _marker: PhantomData,
        }
    }
    
    /// Create a channel for a different role with the same protocol and IO.
    ///
    /// This is useful when you need to create channels for multiple roles in an MPST protocol.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The new role type.
    ///
    /// # Parameters
    ///
    /// * `role` - The new role instance.
    ///
    /// # Returns
    ///
    /// A new `Chan` instance with the specified role.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::chan::Chan;
    /// use sessrums::proto::{Protocol, End, RoleA, RoleB};
    /// use std::sync::mpsc;
    ///
    /// // Create a channel for RoleA
    /// let (tx, _) = mpsc::channel::<i32>();
    /// let chan_a = Chan::<End, RoleA, _>::new(tx);
    ///
    /// // Create a channel for RoleB with the same protocol and IO
    /// let role_b = RoleB;
    /// let chan_b = chan_a.for_role::<RoleB>(role_b);
    /// ```
    pub fn for_role<S: Role>(self, role: S) -> Chan<P, S, IO> {
        Chan {
            io: self.io,
            role,
            _marker: PhantomData,
        }
    }
}

/// Extension trait for channels with protocols that implement ProtocolCompat.
///
/// This trait provides methods for converting between binary and multiparty session types.
pub trait ChanCompat<P: Protocol + ProtocolCompat<R>, R: Role, IO> {
    /// Convert this channel to use a binary protocol.
    ///
    /// # Returns
    ///
    /// A new `Chan` instance with the binary protocol.
    fn to_binary(self) -> Chan<P::BinaryProtocol, R, IO>;
        
    /// Convert this channel to use a multiparty protocol.
    ///
    /// # Returns
    ///
    /// A new `Chan` instance with the multiparty protocol.
    fn from_binary<Q: Protocol>(binary_chan: Chan<Q, R, IO>) -> Chan<P, R, IO>
    where
        P: ProtocolCompat<R, BinaryProtocol = Q>;
}

impl<P: Protocol + ProtocolCompat<R>, R: Role, IO> ChanCompat<P, R, IO> for Chan<P, R, IO> {
    fn to_binary(self) -> Chan<P::BinaryProtocol, R, IO> {
        Chan {
            io: self.io,
            role: self.role,
            _marker: PhantomData,
        }
    }
    
    fn from_binary<Q: Protocol>(binary_chan: Chan<Q, R, IO>) -> Chan<P, R, IO>
    where
        P: ProtocolCompat<R, BinaryProtocol = Q>,
    {
        Chan {
            io: binary_chan.io,
            role: binary_chan.role,
            _marker: PhantomData,
        }
    }
}

impl<T, P: Protocol, R: Role, IO> Chan<crate::proto::Send<T, P>, R, IO>
where
    IO: crate::io::AsyncSender<T>,
    <IO as crate::io::AsyncSender<T>>::Error: std::fmt::Debug,
{
    /// Sends a value of type `T` over the channel and advances the protocol.
    ///
    /// This method consumes the channel and returns a new channel with the advanced protocol.
    /// The protocol advances from `Send<T, P>` to `P` after sending the value.
    ///
    /// This method uses the asynchronous `AsyncSender` trait, which allows for non-blocking
    /// send operations.
    ///
    /// # Parameters
    ///
    /// * `value` - The value to send.
    ///
    /// # Returns
    ///
    /// * `Ok(Chan<P, IO>)` - A new channel with the advanced protocol if the send operation succeeds.
    /// * `Err(Error)` - An error if the send operation fails.
    ///
    /// # Examples
    ///
    pub async fn send(mut self, value: T) -> Result<Chan<P, R, IO>, crate::error::Error> {
        // Send the value using the underlying IO implementation
        // and await the future returned by the async send method
        self.io_mut().send(value).await.map_err(|e| {
            // Convert the IO-specific error to our Error type
            // Since we don't have a specific conversion, we'll wrap it in an IO error
            crate::error::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Send error: {:?}", e),
            ))
        })?;

        // Return a new channel with the advanced protocol
        Ok(Chan {
            io: self.io,
            role: self.role,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<T, P: Protocol, R: Role, IO> Chan<crate::proto::Recv<T, P>, R, IO>
where
    IO: crate::io::AsyncReceiver<T>,
    <IO as crate::io::AsyncReceiver<T>>::Error: std::fmt::Debug,
{
    /// Receives a value of type `T` from the channel and advances the protocol.
    ///
    /// This method consumes the channel and returns the received value along with
    /// a new channel with the advanced protocol. The protocol advances from `Recv<T, P>`
    /// to `P` after receiving the value.
    ///
    /// This method uses the asynchronous `AsyncReceiver` trait, which allows for non-blocking
    /// receive operations.
    ///
    /// # Returns
    ///
    /// * `Ok((T, Chan<P, IO>))` - The received value and a new channel with the advanced protocol
    ///   if the receive operation succeeds.
    /// * `Err(Error)` - An error if the receive operation fails.
    ///
    /// # Examples
    ///
    pub async fn recv(mut self) -> Result<(T, Chan<P, R, IO>), crate::error::Error> {
        // Receive the value using the underlying IO implementation
        // and await the future returned by the async recv method
        let value = self.io_mut().recv().await.map_err(|e| {
            // Convert the IO-specific error to our Error type
            crate::error::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Receive error: {:?}", e),
            ))
        })?;

        // Return the received value and a new channel with the advanced protocol
        Ok((
            value,
            Chan {
                io: self.io,
                role: self.role,
                _marker: std::marker::PhantomData,
            },
        ))
    }
}

impl<L: Protocol, R1: Protocol, R2: Role, IO> Chan<crate::proto::Offer<L, R1>, R2, IO>
where
    IO: crate::io::AsyncReceiver<bool>,
    <IO as crate::io::AsyncReceiver<bool>>::Error: std::fmt::Debug,
{
    /// Offers a choice between two continuations, `L` and `R`, and processes the chosen branch.
    ///
    /// This method allows the other party to choose between two possible continuations of the
    /// protocol. It receives a boolean indicator from the other party and then calls either
    /// function `f` or function `g` based on the choice.
    ///
    /// # Type Parameters
    ///
    /// * `F` - A function type that takes `Chan<L, R2, IO>` and returns `Result<T, Error>`
    /// * `G` - A function type that takes `Chan<R1, R2, IO>` and returns `Result<T, Error>`
    /// * `T` - The return type of both functions `f` and `g`
    ///
    /// # Parameters
    ///
    /// * `f` - The function to call if the left branch is chosen
    /// * `g` - The function to call if the right branch is chosen
    ///
    /// # Returns
    ///
    /// * `Ok(T)` - The result of calling either function `f` or function `g`
    /// * `Err(Error)` - An error if the receive operation fails or if the chosen branch function returns an error
    ///
    /// # Examples
    ///
    pub async fn offer<F, G, T>(mut self, f: F, g: G) -> Result<T, crate::error::Error>
    where
        F: FnOnce(Chan<L, R2, IO>) -> Result<T, crate::error::Error>,
        G: FnOnce(Chan<R1, R2, IO>) -> Result<T, crate::error::Error>,
    {
        // Receive a boolean value indicating which branch to take
        let choice = self.io_mut().recv().await.map_err(|e| {
            crate::error::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Offer error: {:?}", e),
            ))
        })?;

        // Call either function f or function g based on the choice
        if choice {
            // Left branch chosen
            f(Chan {
                io: self.io,
                role: self.role,
                _marker: PhantomData,
            })
        } else {
            // Right branch chosen
            g(Chan {
                io: self.io,
                role: self.role,
                _marker: PhantomData,
            })
        }
    }
}

impl<R: Role, IO> Chan<crate::proto::End, R, IO> {
    /// Closes the channel, indicating that the communication session has ended.
    ///
    /// This method consumes the channel and returns nothing on success, indicating
    /// that the protocol has been completed successfully.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The channel was closed successfully.
    /// * `Err(Error)` - An error if the close operation fails.
    ///
    /// # Examples
    ///
    pub fn close(self) -> Result<(), crate::error::Error> {
        // The End protocol doesn't require any specific action to close
        // We just consume the channel and return Ok(())
        Ok(())
    }
}

impl<L: Protocol, R1: Protocol, R2: Role, IO> Chan<crate::proto::Choose<L, R1>, R2, IO>
where
    IO: crate::io::AsyncSender<bool>,
    <IO as crate::io::AsyncSender<bool>>::Error: std::fmt::Debug,
{
    /// Chooses the left branch of a `Choose<L, R>` protocol and advances the protocol.
    ///
    /// This method sends a boolean indicator (true) to the other party to indicate
    /// the left choice and returns a channel with the left continuation protocol.
    ///
    /// # Returns
    ///
    /// * `Ok(Chan<L, IO>)` - A new channel with the left continuation protocol if the send operation succeeds.
    /// * `Err(Error)` - An error if the send operation fails.
    ///
    /// # Examples
    ///
    pub async fn choose_left(mut self) -> Result<Chan<L, R2, IO>, crate::error::Error> {
        // Send a boolean value (true) indicating the left branch
        self.io_mut().send(true).await.map_err(|e| {
            crate::error::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Choose error: {:?}", e),
            ))
        })?;

        // Return a new channel with the left protocol
        Ok(Chan {
            io: self.io,
            role: self.role,
            _marker: PhantomData,
        })
    }

    /// Chooses the right branch of a `Choose<L, R>` protocol and advances the protocol.
    ///
    /// This method sends a boolean indicator (false) to the other party to indicate
    /// the right choice and returns a channel with the right continuation protocol.
    ///
    /// # Returns
    ///
    /// * `Ok(Chan<R, IO>)` - A new channel with the right continuation protocol if the send operation succeeds.
    /// * `Err(Error)` - An error if the send operation fails.
    ///
    /// # Examples
    ///
    pub async fn choose_right(mut self) -> Result<Chan<R1, R2, IO>, crate::error::Error> {
        // Send a boolean value (false) indicating the right branch
        self.io_mut().send(false).await.map_err(|e| {
            crate::error::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Choose error: {:?}", e),
            ))
        })?;

        // Return a new channel with the right protocol
        Ok(Chan {
            io: self.io,
            role: self.role,
            _marker: PhantomData,
        })
    }
}

// Implementation for recursive protocols
impl<P: Protocol, R: Role, IO> Chan<crate::proto::Rec<P>, R, IO> {
    /// Unwraps a recursive protocol, allowing the inner protocol to be used.
    ///
    /// This method transforms a `Chan<Rec<P>, IO>` into a `Chan<P, IO>`, essentially
    /// "entering" the recursive protocol to access its inner structure.
    ///
    /// # Returns
    ///
    /// A new channel with the unwrapped protocol.
    ///
    /// # Examples
    ///
    pub fn enter(self) -> Chan<P, R, IO> {
        // Simply transform the channel to use the inner protocol
        Chan {
            io: self.io,
            role: self.role,
            _marker: PhantomData,
        }
    }
}

// Implementation for variable references at depth 0
impl<R: Role, IO> Chan<crate::proto::Var<0>, R, IO> {
    /// Converts a variable reference at depth 0 back to a recursive protocol.
    ///
    /// This method handles the base case of recursion, transforming a `Chan<Var<0>, IO>`
    /// into a `Chan<Rec<P>, IO>`, which allows for continuing the recursive protocol.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The protocol type to wrap in `Rec<P>`.
    ///
    /// # Returns
    ///
    /// A new channel with the recursive protocol.
    ///
    /// # Examples
    ///
    pub fn zero<P: Protocol>(self) -> Chan<crate::proto::Rec<P>, R, IO> {
        // Transform the channel to use the recursive protocol
        Chan {
            io: self.io,
            role: self.role,
            _marker: PhantomData,
        }
    }
}

// Helper traits for recursion with const generics

/// A trait for incrementing recursion indices.
///
/// This trait is used to increment the depth of variable references in recursive protocols.
/// It's particularly useful when working with nested recursive protocols.
// Note: The Inc and Dec traits are temporarily disabled due to limitations
// with const generics in the current Rust version.
// These will be re-enabled in a future version.

/// A trait for incrementing recursion indices.
pub trait Inc {
    /// The type with incremented recursion index.
    type Result;
}

/// A trait for decrementing recursion indices.
pub trait Dec {
    /// The type with decremented recursion index.
    type Result;
}

// Placeholder implementations that will be expanded in the future

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{Send, Recv, End, RoleA};
    use crate::io::{Sender, Receiver};
    use std::sync::mpsc;

    #[test]
    fn test_chan_creation() {
        // Test creating a channel with a simple protocol
        type SimpleProtocol = Send<i32, End>;
        let (tx, _) = mpsc::channel::<i32>();
        let _chan = Chan::<SimpleProtocol, RoleA, _>::new(tx);
        
        // Just verify that the channel was created successfully
        // We'll test actual communication in later phases
    }

    #[test]
    fn test_chan_with_complex_protocol() {
        // Test creating a channel with a more complex protocol
        type ComplexProtocol = Send<i32, Recv<String, Send<bool, End>>>;
        let (tx, _) = mpsc::channel::<i32>();
        let _chan = Chan::<ComplexProtocol, RoleA, _>::new(tx);
        
        // Just verify that the channel was created successfully
    }

    #[test]
    fn test_chan_io_access() {
        // Test accessing the underlying IO implementation
        let (tx, rx) = mpsc::channel::<i32>();
        let mut chan_tx = Chan::<End, RoleA, _>::new(tx);
        let mut chan_rx = Chan::<End, RoleA, _>::new(rx);
        
        // Send a value using the underlying IO implementation
        chan_tx.io_mut().send(42).unwrap();
        
        // Receive the value using the underlying IO implementation
        let value = chan_rx.io_mut().recv().unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_chan_with_custom_io() {
        // Define a custom IO implementation for testing
        struct TestIO<T> {
            value: Option<T>,
        }
        
        // Define a custom error type
        #[derive(Debug)]
        struct TestError;
        
        // Implement Sender for TestIO
        impl<T> Sender<T> for TestIO<T> {
            type Error = TestError;
            
            fn send(&mut self, value: T) -> Result<(), Self::Error> {
                self.value = Some(value);
                Ok(())
            }
        }
        
        // Create a channel with our custom IO implementation
        let io = TestIO { value: None };
        let mut chan = Chan::<Send<i32, End>, RoleA, _>::new(io);
        
        // Send a value using the underlying IO implementation
        chan.io_mut().send(42).unwrap();
        
        // Verify that the value was stored in our custom IO implementation
        assert!(chan.io().value.is_some());
        assert_eq!(chan.io().value.as_ref().unwrap(), &42);
    }
}

// Tests for the send, recv, and close methods
#[cfg(test)]
mod protocol_methods {
    use super::*;
    use crate::proto::{Send, Recv, End};
    use crate::proto::roles::RoleA;
    use crate::io::{AsyncSender, AsyncReceiver};
    use futures_core::future::Future;
    use std::pin::Pin;
    use futures_core::task::{Context, Poll};
    // PhantomData is used in the Chan struct creation
    use std::marker::PhantomData;

    // Custom IO implementation for testing
    struct TestIO<T> {
        value: Option<T>,
    }

    // Custom error type
    #[derive(Debug)]
    struct TestError;

    // Define futures for async operations
    struct TestSendFuture<T> {
        io: TestIO<T>,
        value: Option<T>,
    }
    
    #[tokio::test]
    async fn test_offer_method() {
        use super::*;
        use crate::proto::{Offer, Send, End};
        
        // Define test protocols
        type LeftProto = Send<String, End>;
        type RightProto = Send<i32, End>;
        type OfferProto = Offer<LeftProto, RightProto>;
        
        // Custom IO implementation for testing
        struct TestOfferIO {
            choice: Option<bool>,
        }
        
        // Custom error type
        #[derive(Debug)]
        struct TestOfferError;
        
        // Define future for async receive operation
        struct TestOfferRecvFuture {
            value: Option<bool>,
        }
        
        impl Future for TestOfferRecvFuture {
            type Output = Result<bool, TestOfferError>;
            
            fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                let this = unsafe { self.get_unchecked_mut() };
                match this.value.take() {
                    Some(value) => Poll::Ready(Ok(value)),
                    None => Poll::Ready(Err(TestOfferError)),
                }
            }
        }
        
        // Implement AsyncReceiver for TestOfferIO
        impl AsyncReceiver<bool> for TestOfferIO {
            type Error = TestOfferError;
            type RecvFuture<'a> = TestOfferRecvFuture where bool: 'a, Self: 'a;
            
            fn recv(&mut self) -> Self::RecvFuture<'_> {
                TestOfferRecvFuture {
                    value: self.choice.take(),
                }
            }
        }
        
        #[cfg(test)]
        mod choose_methods_tests {
            use super::*;
            use crate::proto::{Choose, Send, End};
            use crate::io::AsyncSender;
            use futures_core::future::Future;
            use std::pin::Pin;
            use futures_core::task::{Context, Poll};
            use std::marker::Unpin;
            use std::sync::{Arc, Mutex};
        
            // Define test protocols
            type LeftProto = Send<String, End>;
            type RightProto = Send<i32, End>;
            type ChooseProto = Choose<LeftProto, RightProto>;
        
            // Define a test IO implementation that can be used for testing choose methods
            #[derive(Clone)]
            struct TestChooseIO {
                choice: Arc<Mutex<Option<bool>>>,
                sent_string: Arc<Mutex<Option<String>>>,
                sent_int: Arc<Mutex<Option<i32>>>,
            }
        
            // Define a custom error type
            #[derive(Debug)]
            struct TestError;
        
            // Define futures for async operations
            struct TestSendFuture<T> {
                value: Option<T>,
                io: TestChooseIO,
            }
        
            impl<T: Clone + Unpin + 'static> Future for TestSendFuture<T> {
                type Output = Result<(), TestError>;
        
                fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                    let this = self.get_mut();
                    if let Some(value) = this.value.take() {
                        // Store the value based on its type
                        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<bool>() {
                            if let Ok(mut choice) = this.io.choice.lock() {
                                // This is a bit of a hack, but it works for testing
                                let bool_value = unsafe { std::mem::transmute_copy::<T, bool>(&value) };
                                *choice = Some(bool_value);
                            }
                        } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<String>() {
                            if let Ok(mut sent_string) = this.io.sent_string.lock() {
                                // This is a bit of a hack, but it works for testing
                                let string_value = unsafe { std::mem::transmute_copy::<T, String>(&value) };
                                *sent_string = Some(string_value);
                            }
                        } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i32>() {
                            if let Ok(mut sent_int) = this.io.sent_int.lock() {
                                // This is a bit of a hack, but it works for testing
                                let int_value = unsafe { std::mem::transmute_copy::<T, i32>(&value) };
                                *sent_int = Some(int_value);
                            }
                        }
                        Poll::Ready(Ok(()))
                    } else {
                        Poll::Ready(Err(TestError))
                    }
                }
            }
        
            // Implement AsyncSender for TestChooseIO
            impl<T: Clone + Unpin + 'static> AsyncSender<T> for TestChooseIO {
                type Error = TestError;
                type SendFuture<'a> = TestSendFuture<T> where T: 'a, Self: 'a;
        
                fn send(&mut self, value: T) -> Self::SendFuture<'_> {
                    TestSendFuture {
                        value: Some(value),
                        io: self.clone(),
                    }
                }
            }
        
            #[tokio::test]
            async fn test_choose_left() {
                // Create a test IO implementation
                let io = TestChooseIO {
                    choice: Arc::new(Mutex::new(None)),
                    sent_string: Arc::new(Mutex::new(None)),
                    sent_int: Arc::new(Mutex::new(None)),
                };
        
                // Create a channel with a Choose protocol
                let chan = Chan::<ChooseProto, RoleA, _>::new(io.clone());
        
                // Choose the left branch
                let chan = chan.choose_left().await.unwrap();
        
                // Verify that the choice was sent correctly
                assert_eq!(*io.choice.lock().unwrap(), Some(true));
        
                // Send a string on the left branch
                let test_string = "Hello, left branch!".to_string();
                let chan = chan.send(test_string.clone()).await.unwrap();
        
                // Verify that the string was sent correctly
                assert_eq!(*io.sent_string.lock().unwrap(), Some(test_string));
        
                // Close the channel
                chan.close().unwrap();
            }
        
            #[tokio::test]
            async fn test_choose_right() {
                // Create a test IO implementation
                let io = TestChooseIO {
                    choice: Arc::new(Mutex::new(None)),
                    sent_string: Arc::new(Mutex::new(None)),
                    sent_int: Arc::new(Mutex::new(None)),
                };
        
                // Create a channel with a Choose protocol
                let chan = Chan::<ChooseProto, RoleA, _>::new(io.clone());
        
                // Choose the right branch
                let chan = chan.choose_right().await.unwrap();
        
                // Verify that the choice was sent correctly
                assert_eq!(*io.choice.lock().unwrap(), Some(false));
        
                // Send an integer on the right branch
                let test_int = 42;
                let chan = chan.send(test_int).await.unwrap();
        
                // Verify that the integer was sent correctly
                assert_eq!(*io.sent_int.lock().unwrap(), Some(test_int));
        
                // Close the channel
                chan.close().unwrap();
            }
        
            #[tokio::test]
            async fn test_choose_error_handling() {
                // Create a test IO implementation that will cause an error
                struct ErrorIO;
        
                #[derive(Debug)]
                struct ErrorIOError;
        
                struct ErrorSendFuture;
        
                impl Future for ErrorSendFuture {
                    type Output = Result<(), ErrorIOError>;
        
                    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                        Poll::Ready(Err(ErrorIOError))
                    }
                }
        
                impl AsyncSender<bool> for ErrorIO {
                    type Error = ErrorIOError;
                    type SendFuture<'a> = ErrorSendFuture where Self: 'a;
        
                    fn send(&mut self, _value: bool) -> Self::SendFuture<'_> {
                        ErrorSendFuture
                    }
                }
        
                // Create a channel with a Choose protocol
                let chan = Chan::<ChooseProto, RoleA, _>::new(ErrorIO);
        
                // Try to choose the left branch, which should fail
                let result = chan.choose_left().await;
                assert!(result.is_err());
        
                // Create another channel for testing choose_right
                let chan = Chan::<ChooseProto, RoleA, _>::new(ErrorIO);
        
                // Try to choose the right branch, which should also fail
                let result = chan.choose_right().await;
                assert!(result.is_err());
            }
        }
        
        // Test the left branch
        {
            // Create a channel with Offer protocol and choice set to true (left)
            let io = TestOfferIO { choice: Some(true) };
            let chan = Chan::<OfferProto, RoleA, _>::new(io);
            
            // Define handlers for left and right branches
            let handle_left = |_chan: Chan<LeftProto, RoleA, TestOfferIO>| -> Result<String, crate::error::Error> {
                // We don't actually send here since our TestOfferIO doesn't support String
                // Just verify we got the correct branch
                Ok("Left branch taken".to_string())
            };
            
            let handle_right = |_: Chan<RightProto, RoleA, TestOfferIO>| -> Result<String, crate::error::Error> {
                // This should not be called
                panic!("Right branch handler should not be called when left branch is chosen");
            };
            
            // Offer a choice
            let result = chan.offer(handle_left, handle_right).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "Left branch taken");
        }
        
        // Test the right branch
        {
            // Create a channel with Offer protocol and choice set to false (right)
            let io = TestOfferIO { choice: Some(false) };
            let chan = Chan::<OfferProto, RoleA, _>::new(io);
            
            // Define handlers for left and right branches
            let handle_left = |_: Chan<LeftProto, RoleA, TestOfferIO>| -> Result<String, crate::error::Error> {
                // This should not be called
                panic!("Left branch handler should not be called when right branch is chosen");
            };
            
            let handle_right = |_chan: Chan<RightProto, RoleA, TestOfferIO>| -> Result<String, crate::error::Error> {
                // We don't actually send here since our TestOfferIO doesn't support i32 for this test
                // Just verify we got the correct branch
                Ok("Right branch taken".to_string())
            };
            
            // Offer a choice
            let result = chan.offer(handle_left, handle_right).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "Right branch taken");
        }
        
        // Test error handling
        {
            // Create a channel with Offer protocol but no value to receive
            let io = TestOfferIO { choice: None };
            let chan = Chan::<OfferProto, RoleA, _>::new(io);
            
            // Define handlers for left and right branches
            let handle_left = |_: Chan<LeftProto, RoleA, TestOfferIO>| -> Result<String, crate::error::Error> {
                Ok("Left branch taken".to_string())
            };
            
            let handle_right = |_: Chan<RightProto, RoleA, TestOfferIO>| -> Result<String, crate::error::Error> {
                Ok("Right branch taken".to_string())
            };
            
            // Attempt to offer a choice (should fail)
            let result = chan.offer(handle_left, handle_right).await;
            assert!(result.is_err());
        }
    }

    struct TestRecvFuture<T> {
        value: Option<T>,
    }

    // Implement Future for TestSendFuture
    impl<T: Clone + std::marker::Unpin> Future for TestSendFuture<T> {
        type Output = Result<(), TestError>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if let Some(value) = this.value.take() {
                this.io.value = Some(value);
                Poll::Ready(Ok(()))
            } else {
                Poll::Ready(Err(TestError))
            }
        }
    }

    // Implement Future for TestRecvFuture
    impl<T: Clone + std::marker::Unpin> Future for TestRecvFuture<T> {
        type Output = Result<T, TestError>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            match this.value.take() {
                Some(value) => Poll::Ready(Ok(value)),
                None => Poll::Ready(Err(TestError)),
            }
        }
    }

    // Implement AsyncSender for TestIO
    impl<T: Clone + std::marker::Unpin + 'static> AsyncSender<T> for TestIO<T> {
        type Error = TestError;
        type SendFuture<'a> = TestSendFuture<T> where T: 'a, Self: 'a;

        fn send(&mut self, value: T) -> Self::SendFuture<'_> {
            TestSendFuture {
                io: TestIO { value: None },
                value: Some(value),
            }
        }
    }
    
    // Implement AsyncReceiver for TestIO
    impl<T: Clone + std::marker::Unpin + 'static> AsyncReceiver<T> for TestIO<T> {
        type Error = TestError;
        type RecvFuture<'a> = TestRecvFuture<T> where T: 'a, Self: 'a;
        
        fn recv(&mut self) -> Self::RecvFuture<'_> {
            TestRecvFuture {
                value: self.value.clone(),
            }
        }
    }

    #[tokio::test]
    async fn test_send_method() {
        // Create a channel with Send<i32, End> protocol
        let io = TestIO { value: None };
        let chan = Chan::<Send<i32, End>, RoleA, _>::new(io);
        
        // Send a value
        let result = chan.send(42).await;
        assert!(result.is_ok());
        
        // Check that the protocol advanced to End
        let chan = result.unwrap();
        let _: Chan<End, RoleA, _> = chan; // Type check
    }

    #[tokio::test]
    async fn test_recv_method() {
        // Create a channel with Recv<i32, End> protocol
        let io = TestIO { value: Some(42) };
        let chan = Chan::<Recv<i32, End>, RoleA, _>::new(io);
        
        // Receive a value
        let result = chan.recv().await;
        assert!(result.is_ok());
        
        // Check that we received the correct value and the protocol advanced to End
        let (value, chan) = result.unwrap();
        assert_eq!(value, 42);
        let _: Chan<End, RoleA, _> = chan; // Type check
    }

    #[tokio::test]
    async fn test_recv_error() {
        // Create a channel with Recv<i32, End> protocol but no value to receive
        let io = TestIO::<i32> { value: None };
        let chan = Chan::<Recv<i32, End>, RoleA, _>::new(io);
        
        // Attempt to receive a value (should fail)
        let result = chan.recv().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_close_method() {
        // Create a channel with End protocol
        let io = TestIO::<i32> { value: None };
        let chan = Chan::<End, RoleA, _>::new(io);
        
        // Close the channel
        let result = chan.close();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_protocol_sequence() {
        // Test a sequence of protocol steps: Send<i32, Recv<String, End>>
        
        // Create a channel with the initial protocol
        let io = TestIO::<i32> { value: None };
        let chan = Chan::<Send<i32, Recv<String, End>>, RoleA, _>::new(io);
        
        // Send an i32
        let chan = chan.send(42).await.unwrap();
        
        // The protocol should now be Recv<String, End>
        let _: Chan<Recv<String, End>, RoleA, _> = chan;
        
        // We need to create a new TestIO for the next step since the types change
        let io_string = TestIO::<String> { value: Some("Hello".to_string()) };
        let chan = Chan::<Recv<String, End>, RoleA, _>::new(io_string);
        
        // Receive a String
        let (value, chan) = chan.recv().await.unwrap();
        assert_eq!(value, "Hello");
        
        // The protocol should now be End
        let _: Chan<End, RoleA, _> = chan;
        
        // Close the channel
        let result = chan.close();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_recursive_protocols() {
        use crate::proto::{Rec, Var, Send, End};

        // Define a simple recursive protocol: Rec<Send<i32, Var<0>>>
        // This protocol repeatedly sends an i32 and then loops back to itself
        type LoopingSend = Rec<Send<i32, Var<0>>>;

        // Create a channel with the recursive protocol
        let io = TestIO::<i32> { value: None };
        let chan = Chan::<LoopingSend, RoleA, _>::new(io);

        // Enter the recursive protocol
        let chan = chan.enter();

        // The protocol should now be Send<i32, Var<0>>
        let _: Chan<Send<i32, Var<0>>, RoleA, _> = chan;

        // Create a new channel for the next step
        let io = TestIO::<i32> { value: None };
        let chan = Chan::<Send<i32, Var<0>>, RoleA, _>::new(io);

        // Send an i32
        let chan = chan.send(42).await.unwrap();

        // The protocol should now be Var<0>
        let _: Chan<Var<0>, RoleA, _> = chan;

        // Use zero to loop back to the recursive protocol
        let chan = chan.zero::<Send<i32, Var<0>>>();

        // The protocol should now be Rec<Send<i32, Var<0>>>
        let _: Chan<Rec<Send<i32, Var<0>>>, RoleA, _> = chan;

        // We can enter the recursive protocol again
        let chan = chan.enter();

        // The protocol should now be Send<i32, Var<0>> again
        let _: Chan<Send<i32, Var<0>>, RoleA, _> = chan;
    }


    #[tokio::test]
    async fn test_nested_recursive_protocols() {
        use crate::proto::{Rec, Var, Send, End};

        // Define a nested recursive protocol:
        // Outer recursion: Rec<Send<i32, Inner>>
        // Inner recursion: Rec<Send<String, Var<0>>>
        type InnerRec = Rec<Send<String, Var<0>>>;
        type OuterRec = Rec<Send<i32, InnerRec>>;

        // Create a channel with the outer recursive protocol
        let io = TestIO::<i32> { value: None };
        let chan = Chan::<OuterRec, RoleA, _>::new(io);

        // Enter the outer recursive protocol
        let chan = chan.enter();

        // The protocol should now be Send<i32, InnerRec>
        let _: Chan<Send<i32, InnerRec>, RoleA, _> = chan;

        // Create a new channel for the next step
        let io = TestIO::<i32> { value: None };
        let chan = Chan::<Send<i32, InnerRec>, RoleA, _>::new(io);

        // Send an i32
        let chan = chan.send(42).await.unwrap();

        // The protocol should now be InnerRec
        let _: Chan<InnerRec, RoleA, _> = chan;

        // Enter the inner recursive protocol
        let chan = chan.enter();

        // The protocol should now be Send<String, Var<0>>
        let _: Chan<Send<String, Var<0>>, RoleA, _> = chan;

        // Create a new channel for the next step
        let io = TestIO::<String> { value: None };
        let chan = Chan::<Send<String, Var<0>>, RoleA, _>::new(io);

        // Send a String
        let chan = chan.send("Hello".to_string()).await.unwrap();

        // The protocol should now be Var<0>
        let _: Chan<Var<0>, RoleA, _> = chan;

        // Use zero to loop back to the inner recursive protocol
        let chan = chan.zero::<Send<String, Var<0>>>();

        // The protocol should now be Rec<Send<String, Var<0>>>
        let _: Chan<Rec<Send<String, Var<0>>>, RoleA, _> = chan;
    }
}