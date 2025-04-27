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

impl<T, P: Protocol, IO> Chan<crate::proto::Send<T, P>, IO>
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
    /// ```no_run
    /// # async fn example() -> Result<(), sez::error::Error> {
    /// use sez::chan::Chan;
    /// use sez::proto::{Send, End};
    /// use sez::io::AsyncSender;
    /// use futures_core::future::Future;
    /// use std::pin::Pin;
    /// use futures_core::task::{Context, Poll};
    /// use std::marker::Unpin;
    ///
    /// // Define a custom IO implementation
    /// struct MyIO {
    ///     value: Option<i32>
    /// }
    ///
    /// // Define a custom error type
    /// #[derive(Debug)]
    /// struct MyError;
    ///
    /// // Define a future for the async send operation
    /// struct SendFuture<T: Unpin + 'static> {
    ///     io: MyIO,
    ///     value: Option<T>,
    /// }
    ///
    /// impl<T: Unpin + 'static> Future for SendFuture<T> {
    ///     type Output = Result<(), MyError>;
    ///
    ///     fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    ///         let this = self.get_mut();
    ///         if let Some(value) = this.value.take() {
    ///             match value {
    ///                 v if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i32>() => {
    ///                     // This is a simplification for the doctest
    ///                     this.io.value = Some(42);
    ///                     Poll::Ready(Ok(()))
    ///                 },
    ///                 _ => Poll::Ready(Err(MyError))
    ///             }
    ///         } else {
    ///             Poll::Ready(Err(MyError))
    ///         }
    ///     }
    /// }
    ///
    /// // Implement AsyncSender for our custom IO
    /// impl AsyncSender<i32> for MyIO {
    ///     type Error = MyError;
    ///     type SendFuture<'a> = SendFuture<i32> where i32: 'a, Self: 'a;
    ///
    ///     fn send(&mut self, value: i32) -> Self::SendFuture<'_> {
    ///         SendFuture {
    ///             io: MyIO { value: None },
    ///             value: Some(value),
    ///         }
    ///     }
    /// }
    ///
    /// // Create a channel with a Send<i32, End> protocol
    /// let io = MyIO { value: None };
    /// let chan = Chan::<Send<i32, End>, _>::new(io);
    ///
    /// // Send a value and advance the protocol to End
    /// let chan = chan.send(42).await?;
    ///
    /// // Now the channel has protocol End
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(mut self, value: T) -> Result<Chan<P, IO>, crate::error::Error> {
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
            _marker: std::marker::PhantomData,
        })
    }
}

impl<T, P: Protocol, IO> Chan<crate::proto::Recv<T, P>, IO>
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
    /// ```no_run
    /// # async fn example() -> Result<(), sez::error::Error> {
    /// use sez::chan::Chan;
    /// use sez::proto::{Recv, End};
    /// use sez::io::AsyncReceiver;
    /// use futures_core::future::Future;
    /// use std::pin::Pin;
    /// use futures_core::task::{Context, Poll};
    /// use std::marker::Unpin;
    ///
    /// // Define a custom IO implementation
    /// struct MyIO {
    ///     value: Option<i32>
    /// }
    ///
    /// // Define a custom error type
    /// #[derive(Debug)]
    /// struct MyError;
    ///
    /// // Define a future for the async receive operation
    /// struct RecvFuture<T: Unpin> {
    ///     value: Option<T>,
    /// }
    ///
    /// impl<T: Unpin> Future for RecvFuture<T> {
    ///     type Output = Result<T, MyError>;
    ///
    ///     fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    ///         let this = self.get_mut();
    ///         match this.value.take() {
    ///             Some(value) => Poll::Ready(Ok(value)),
    ///             None => Poll::Ready(Err(MyError)),
    ///         }
    ///     }
    /// }
    ///
    /// // Implement AsyncReceiver for our custom IO
    /// impl AsyncReceiver<i32> for MyIO {
    ///     type Error = MyError;
    ///     type RecvFuture<'a> = RecvFuture<i32> where i32: 'a, Self: 'a;
    ///
    ///     fn recv(&mut self) -> Self::RecvFuture<'_> {
    ///         RecvFuture {
    ///             value: self.value.take(),
    ///         }
    ///     }
    /// }
    ///
    /// // Create a channel with a Recv<i32, End> protocol
    /// let io = MyIO { value: Some(42) };
    /// let chan = Chan::<Recv<i32, End>, _>::new(io);
    ///
    /// // Receive a value and advance the protocol to End
    /// let (value, chan) = chan.recv().await?;
    /// assert_eq!(value, 42);
    ///
    /// // Now the channel has protocol End
    /// # Ok(())
    /// # }
    /// ```
    pub async fn recv(mut self) -> Result<(T, Chan<P, IO>), crate::error::Error> {
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
                _marker: std::marker::PhantomData,
            },
        ))
    }
}

impl<L: Protocol, R: Protocol, IO> Chan<crate::proto::Offer<L, R>, IO>
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
    /// * `F` - A function type that takes `Chan<L, IO>` and returns `Result<T, Error>`
    /// * `G` - A function type that takes `Chan<R, IO>` and returns `Result<T, Error>`
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
    /// ```no_run
    /// # async fn example() -> Result<(), sez::error::Error> {
    /// use sez::chan::Chan;
    /// use sez::proto::{Offer, Send, Recv, End};
    /// use sez::io::AsyncReceiver;
    /// use futures_core::future::Future;
    /// use std::pin::Pin;
    /// use futures_core::task::{Context, Poll};
    /// use std::marker::Unpin;
    ///
    /// // Define protocols for the left and right branches
    /// type LeftProto = Send<String, End>;
    /// type RightProto = Send<i32, End>;
    /// type OfferProto = Offer<LeftProto, RightProto>;
    ///
    /// // Define a custom IO implementation
    /// struct MyIO {
    ///     choice: Option<bool>,
    ///     string_value: Option<String>,
    ///     int_value: Option<i32>,
    /// }
    ///
    /// // Define a custom error type
    /// #[derive(Debug)]
    /// struct MyError;
    ///
    /// // Define a future for the async receive operation
    /// struct RecvFuture<T> {
    ///     value: Option<T>,
    /// }
    ///
    /// impl<T: Unpin> Future for RecvFuture<T> {
    ///     type Output = Result<T, MyError>;
    ///
    ///     fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    ///         let this = self.get_mut();
    ///         match this.value.take() {
    ///             Some(value) => Poll::Ready(Ok(value)),
    ///             None => Poll::Ready(Err(MyError)),
    ///         }
    ///     }
    /// }
    ///
    /// // Implement AsyncReceiver for our custom IO
    /// impl AsyncReceiver<bool> for MyIO {
    ///     type Error = MyError;
    ///     type RecvFuture<'a> = RecvFuture<bool> where bool: 'a, Self: 'a;
    ///
    ///     fn recv(&mut self) -> Self::RecvFuture<'_> {
    ///         RecvFuture {
    ///             value: self.choice.take(),
    ///         }
    ///     }
    /// }
    ///
    /// // Create a channel with an Offer protocol
    /// let io = MyIO {
    ///     choice: Some(true), // The other party will choose the left branch
    ///     string_value: Some("Hello".to_string()),
    ///     int_value: Some(42),
    /// };
    /// let chan = Chan::<OfferProto, _>::new(io);
    ///
    /// // Define handlers for the left and right branches as non-async functions
    /// // that return Result directly
    /// let handle_left = |_chan: Chan<LeftProto, MyIO>| -> Result<String, sez::error::Error> {
    ///     // In a real implementation, we would send a string and then end
    ///     // But for this example, we just return a result directly
    ///     Ok("Left branch completed".to_string())
    /// };
    ///
    /// let handle_right = |_chan: Chan<RightProto, MyIO>| -> Result<String, sez::error::Error> {
    ///     // In a real implementation, we would send an integer and then end
    ///     // But for this example, we just return a result directly
    ///     Ok("Right branch completed".to_string())
    /// };
    ///
    /// // Offer a choice between the left and right branches
    /// let result = chan.offer(handle_left, handle_right).await?;
    /// println!("Result: {}", result);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn offer<F, G, T>(mut self, f: F, g: G) -> Result<T, crate::error::Error>
    where
        F: FnOnce(Chan<L, IO>) -> Result<T, crate::error::Error>,
        G: FnOnce(Chan<R, IO>) -> Result<T, crate::error::Error>,
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
                _marker: PhantomData,
            })
        } else {
            // Right branch chosen
            g(Chan {
                io: self.io,
                _marker: PhantomData,
            })
        }
    }
}

impl<IO> Chan<crate::proto::End, IO> {
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
    /// ```
    /// # fn example() -> Result<(), sez::error::Error> {
    /// use sez::chan::Chan;
    /// use sez::proto::End;
    /// use std::sync::mpsc;
    ///
    /// // Create a channel with an End protocol
    /// let (tx, _) = mpsc::channel::<i32>();
    /// let chan = Chan::<End, _>::new(tx);
    ///
    /// // Close the channel
    /// chan.close()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn close(self) -> Result<(), crate::error::Error> {
        // The End protocol doesn't require any specific action to close
        // We just consume the channel and return Ok(())
        Ok(())
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

// Tests for the send, recv, and close methods
#[cfg(test)]
mod protocol_methods {
    use super::*;
    use crate::proto::{Send, Recv, End};
    use crate::io::{AsyncSender, AsyncReceiver};
    use futures_core::future::Future;
    use std::pin::Pin;
    use futures_core::task::{Context, Poll};
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
        
        // Test the left branch
        {
            // Create a channel with Offer protocol and choice set to true (left)
            let io = TestOfferIO { choice: Some(true) };
            let chan = Chan::<OfferProto, _>::new(io);
            
            // Define handlers for left and right branches
            let handle_left = |_chan: Chan<LeftProto, TestOfferIO>| -> Result<String, crate::error::Error> {
                // We don't actually send here since our TestOfferIO doesn't support String
                // Just verify we got the correct branch
                Ok("Left branch taken".to_string())
            };
            
            let handle_right = |_: Chan<RightProto, TestOfferIO>| -> Result<String, crate::error::Error> {
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
            let chan = Chan::<OfferProto, _>::new(io);
            
            // Define handlers for left and right branches
            let handle_left = |_: Chan<LeftProto, TestOfferIO>| -> Result<String, crate::error::Error> {
                // This should not be called
                panic!("Left branch handler should not be called when right branch is chosen");
            };
            
            let handle_right = |_chan: Chan<RightProto, TestOfferIO>| -> Result<String, crate::error::Error> {
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
            let chan = Chan::<OfferProto, _>::new(io);
            
            // Define handlers for left and right branches
            let handle_left = |_: Chan<LeftProto, TestOfferIO>| -> Result<String, crate::error::Error> {
                Ok("Left branch taken".to_string())
            };
            
            let handle_right = |_: Chan<RightProto, TestOfferIO>| -> Result<String, crate::error::Error> {
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
        let chan = Chan::<Send<i32, End>, _>::new(io);
        
        // Send a value
        let result = chan.send(42).await;
        assert!(result.is_ok());
        
        // Check that the protocol advanced to End
        let chan = result.unwrap();
        let _: Chan<End, _> = chan; // Type check
    }

    #[tokio::test]
    async fn test_recv_method() {
        // Create a channel with Recv<i32, End> protocol
        let io = TestIO { value: Some(42) };
        let chan = Chan::<Recv<i32, End>, _>::new(io);
        
        // Receive a value
        let result = chan.recv().await;
        assert!(result.is_ok());
        
        // Check that we received the correct value and the protocol advanced to End
        let (value, chan) = result.unwrap();
        assert_eq!(value, 42);
        let _: Chan<End, _> = chan; // Type check
    }

    #[tokio::test]
    async fn test_recv_error() {
        // Create a channel with Recv<i32, End> protocol but no value to receive
        let io = TestIO::<i32> { value: None };
        let chan = Chan::<Recv<i32, End>, _>::new(io);
        
        // Attempt to receive a value (should fail)
        let result = chan.recv().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_close_method() {
        // Create a channel with End protocol
        let io = TestIO::<i32> { value: None };
        let chan = Chan::<End, _>::new(io);
        
        // Close the channel
        let result = chan.close();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_protocol_sequence() {
        // Test a sequence of protocol steps: Send<i32, Recv<String, End>>
        
        // Create a channel with the initial protocol
        let io = TestIO::<i32> { value: None };
        let chan = Chan::<Send<i32, Recv<String, End>>, _>::new(io);
        
        // Send an i32
        let chan = chan.send(42).await.unwrap();
        
        // The protocol should now be Recv<String, End>
        let _: Chan<Recv<String, End>, _> = chan;
        
        // We need to create a new TestIO for the next step since the types change
        let io_string = TestIO::<String> { value: Some("Hello".to_string()) };
        let chan = Chan::<Recv<String, End>, _>::new(io_string);
        
        // Receive a String
        let (value, chan) = chan.recv().await.unwrap();
        assert_eq!(value, "Hello");
        
        // The protocol should now be End
        let _: Chan<End, _> = chan;
        
        // Close the channel
        let result = chan.close();
        assert!(result.is_ok());
    }
}