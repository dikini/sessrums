//! Channel implementations for session types.
//!
//! This module contains the channel implementations that allow
//! communication according to the protocols defined in the `proto` module.

use std::marker::PhantomData;
use crate::proto::Protocol;

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
/// ```
/// use sez::chan::Chan;
/// use sez::proto::{Protocol, Send, Recv, End};
/// use std::sync::mpsc;
///
/// // Define a protocol: Send an i32, then receive a String, then end
/// type MyProtocol = Send<i32, Recv<String, End>>;
///
/// // Create a channel using mpsc::Sender as the IO implementation
/// let (tx, _) = mpsc::channel::<i32>();
/// let chan = Chan::<MyProtocol, _>::new(tx);
///
/// // Access the underlying IO implementation
/// let io_ref = chan.io();
/// ```
///
/// Using the channel with custom IO implementations:
///
/// ```
/// use sez::chan::Chan;
/// use sez::proto::{Protocol, End};
/// use sez::io::{Sender, Receiver};
///
/// // A custom IO implementation
/// struct MyIO {
///     value: Option<i32>
/// }
///
/// // Define a custom error type
/// #[derive(Debug)]
/// struct MyError;
///
/// // Implement Sender for our custom IO
/// impl Sender<i32> for MyIO {
///     type Error = MyError;
///
///     fn send(&mut self, value: i32) -> Result<(), Self::Error> {
///         self.value = Some(value);
///         Ok(())
///     }
/// }
///
/// // Create a channel with our custom IO implementation
/// let io = MyIO { value: None };
/// let chan = Chan::<End, _>::new(io);
/// ```
pub struct Chan<P: Protocol, IO> {
    /// The underlying IO implementation.
    io: IO,
    /// Phantom data to carry the protocol type.
    _marker: PhantomData<P>,
}

impl<P: Protocol, IO> Chan<P, IO> {
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
    /// ```
    /// use sez::chan::Chan;
    /// use sez::proto::{Protocol, End};
    /// use std::sync::mpsc;
    ///
    /// // Create a channel with mpsc::Sender as the IO implementation
    /// let (tx, _) = mpsc::channel::<i32>();
    /// let chan = Chan::<End, _>::new(tx);
    /// ```
    pub fn new(io: IO) -> Self {
        Chan {
            io,
            _marker: PhantomData,
        }
    }

    /// Get a reference to the underlying IO implementation.
    ///
    /// # Returns
    ///
    /// A reference to the underlying IO implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sez::chan::Chan;
    /// use sez::proto::{Protocol, End};
    /// use std::sync::mpsc;
    ///
    /// // Create a channel with mpsc::Sender as the IO implementation
    /// let (tx, _) = mpsc::channel::<i32>();
    /// let chan = Chan::<End, _>::new(tx);
    ///
    /// // Get a reference to the underlying IO implementation
    /// let io_ref = chan.io();
    /// ```
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
    /// ```
    /// use sez::chan::Chan;
    /// use sez::proto::{Protocol, End};
    /// use sez::io::Sender;
    /// use std::sync::mpsc;
    ///
    /// // Create a channel with mpsc::Sender as the IO implementation
    /// let (mut tx, _) = mpsc::channel::<i32>();
    /// let mut chan = Chan::<End, _>::new(tx);
    ///
    /// // Get a mutable reference to the underlying IO implementation
    /// let io_mut = chan.io_mut();
    /// ```
    pub fn io_mut(&mut self) -> &mut IO {
        &mut self.io
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{Send, Recv, End};
    use crate::io::{Sender, Receiver};
    use std::sync::mpsc;

    #[test]
    fn test_chan_creation() {
        // Test creating a channel with a simple protocol
        type SimpleProtocol = Send<i32, End>;
        let (tx, _) = mpsc::channel::<i32>();
        let _chan = Chan::<SimpleProtocol, _>::new(tx);
        
        // Just verify that the channel was created successfully
        // We'll test actual communication in later phases
    }

    #[test]
    fn test_chan_with_complex_protocol() {
        // Test creating a channel with a more complex protocol
        type ComplexProtocol = Send<i32, Recv<String, Send<bool, End>>>;
        let (tx, _) = mpsc::channel::<i32>();
        let _chan = Chan::<ComplexProtocol, _>::new(tx);
        
        // Just verify that the channel was created successfully
    }

    #[test]
    fn test_chan_io_access() {
        // Test accessing the underlying IO implementation
        let (tx, rx) = mpsc::channel::<i32>();
        let mut chan_tx = Chan::<End, _>::new(tx);
        let mut chan_rx = Chan::<End, _>::new(rx);
        
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
        let mut chan = Chan::<Send<i32, End>, _>::new(io);
        
        // Send a value using the underlying IO implementation
        chan.io_mut().send(42).unwrap();
        
        // Verify that the value was stored in our custom IO implementation
        assert!(chan.io().value.is_some());
        assert_eq!(chan.io().value.as_ref().unwrap(), &42);
    }
}