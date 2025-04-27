//! Basic IO traits for session types.
//!
//! This module defines the fundamental IO traits that abstract over different
//! IO implementations. These traits provide a common interface for sending and
//! receiving values, which is essential for implementing the session type
//! communication channels.
//!
//! The module provides both synchronous and asynchronous versions of the IO traits:
//! - `Sender` and `Receiver` for synchronous operations
//! - `AsyncSender` and `AsyncReceiver` for asynchronous operations using futures

/// A trait for sending values of type T.
///
/// This trait abstracts over different ways of sending values, allowing the
/// session type system to work with various IO implementations.
///
/// # Type Parameters
///
/// * `T` - The type of value to be sent.
///
/// # Examples
///
/// ```
/// use sez::io::Sender;
///
/// // Define a custom sender type
/// struct MySender<T> {
///     value: Option<T>,
/// }
///
/// // Define a custom error type
/// #[derive(Debug)]
/// struct MySendError;
///
/// // Implement Sender for our custom type
/// impl<T> Sender<T> for MySender<T> {
///     type Error = MySendError;
///
///     fn send(&mut self, value: T) -> Result<(), Self::Error> {
///         self.value = Some(value);
///         Ok(())
///     }
/// }
///
/// // Now we can use the sender through the Sender trait
/// let mut sender = MySender { value: None };
/// let result = sender.send(42);
/// assert!(result.is_ok());
/// assert!(sender.value.is_some());
/// assert_eq!(sender.value.unwrap(), 42);
/// ```
pub trait Sender<T> {
    /// The error type that can occur during sending.
    ///
    /// This associated type allows different implementations to specify
    /// their own error types, providing flexibility while maintaining
    /// type safety.
    type Error;
    
    /// Send a value.
    ///
    /// This method sends a value of type `T` through the underlying
    /// communication mechanism.
    ///
    /// # Parameters
    ///
    /// * `value` - The value to send.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the value was successfully sent.
    /// * `Err(Self::Error)` if an error occurred during sending.
    fn send(&mut self, value: T) -> Result<(), Self::Error>;
}

/// A trait for receiving values of type T.
///
/// This trait abstracts over different ways of receiving values, allowing the
/// session type system to work with various IO implementations.
///
/// # Type Parameters
///
/// * `T` - The type of value to be received.
///
/// # Examples
///
/// ```
/// use sez::io::Receiver;
///
/// // Define a custom receiver type
/// struct MyReceiver<T> {
///     value: Option<T>,
/// }
///
/// // Define a custom error type
/// #[derive(Debug)]
/// struct MyRecvError;
///
/// // Implement Receiver for our custom type
/// impl<T> Receiver<T> for MyReceiver<T> {
///     type Error = MyRecvError;
///
///     fn recv(&mut self) -> Result<T, Self::Error> {
///         self.value.take().ok_or(MyRecvError)
///     }
/// }
///
/// // Now we can use the receiver through the Receiver trait
/// let mut receiver = MyReceiver { value: Some(42) };
/// let result = receiver.recv();
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), 42);
///
/// // The value has been taken, so a second receive should fail
/// let result = receiver.recv();
/// assert!(result.is_err());
/// ```
pub trait Receiver<T> {
    /// The error type that can occur during receiving.
    ///
    /// This associated type allows different implementations to specify
    /// their own error types, providing flexibility while maintaining
    /// type safety.
    type Error;
    
    /// Receive a value.
    ///
    /// This method receives a value of type `T` through the underlying
    /// communication mechanism.
    ///
    /// # Returns
    ///
    /// * `Ok(T)` if a value was successfully received.
    /// * `Err(Self::Error)` if an error occurred during receiving.
    fn recv(&mut self) -> Result<T, Self::Error>;
}

/// A trait for asynchronously receiving values of type T.
///
/// This trait abstracts over different ways of receiving values asynchronously,
/// allowing the session type system to work with various async IO implementations.
/// It uses the `Future` trait from the `futures-core` crate to represent
/// asynchronous operations.
///
/// # Type Parameters
///
/// * `T` - The type of value to be received.
///
/// # Examples
///
/// ```
/// use sez::io::AsyncReceiver;
/// use futures_core::future::Future;
/// use std::pin::Pin;
/// use futures_core::task::{Context, Poll};
/// use std::marker::Unpin;
///
/// // Define a custom async receiver type
/// struct MyAsyncReceiver<T> {
///     value: Option<T>,
/// }
///
/// // Define a custom error type
/// #[derive(Debug)]
/// struct MyRecvError;
///
/// // Define a simple future for our async receive operation
/// struct RecvFuture<T> {
///     receiver: MyAsyncReceiver<T>,
/// }
///
/// impl<T: Unpin> Future for RecvFuture<T> {
///     type Output = Result<T, MyRecvError>;
///
///     fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
///         // In a real implementation, this would check if a value is available
///         // and potentially register a waker if not
///         let this = self.get_mut();
///         match this.receiver.value.take() {
///             Some(value) => Poll::Ready(Ok(value)),
///             None => Poll::Ready(Err(MyRecvError)),
///         }
///     }
/// }
///
/// // Implement AsyncReceiver for our custom type
/// impl<T: Unpin> AsyncReceiver<T> for MyAsyncReceiver<T> {
///     type Error = MyRecvError;
///     type RecvFuture<'a> = RecvFuture<T> where T: 'a, Self: 'a;
///
///     fn recv(&mut self) -> Self::RecvFuture<'_> {
///         RecvFuture {
///             receiver: MyAsyncReceiver { value: self.value.take() },
///         }
///     }
/// }
///
/// // Now we can use the receiver through the AsyncReceiver trait
/// # async fn example() {
/// let mut receiver = MyAsyncReceiver { value: Some(42) };
/// let result = receiver.recv().await;
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), 42);
///
/// // The value has been taken, so a second receive should fail
/// let result = receiver.recv().await;
/// assert!(result.is_err());
/// # }
/// ```
pub trait AsyncReceiver<T> {
    /// The error type that can occur during receiving.
    ///
    /// This associated type allows different implementations to specify
    /// their own error types, providing flexibility while maintaining
    /// type safety.
    type Error;
    
    /// The future returned by the `recv` method.
    ///
    /// This associated type represents the asynchronous operation of receiving
    /// a value. It must implement the `Future` trait with an output type of
    /// `Result<T, Self::Error>`.
    type RecvFuture<'a>: futures_core::Future<Output = Result<T, Self::Error>> + 'a
    where
        T: 'a,
        Self: 'a;
    
    /// Asynchronously receive a value.
    ///
    /// This method receives a value of type `T` through the underlying
    /// communication mechanism asynchronously.
    ///
    /// # Returns
    ///
    /// A future that resolves to:
    /// * `Ok(T)` if a value was successfully received.
    /// * `Err(Self::Error)` if an error occurred during receiving.
    fn recv(&mut self) -> Self::RecvFuture<'_>;
}

/// A trait for asynchronously sending values of type T.
///
/// This trait abstracts over different ways of sending values asynchronously,
/// allowing the session type system to work with various async IO implementations.
/// It uses the `Future` trait from the `futures-core` crate to represent
/// asynchronous operations.
///
/// # Type Parameters
///
/// * `T` - The type of value to be sent.
///
/// # Examples
///
/// ```
/// use sez::io::AsyncSender;
/// use futures_core::future::Future;
/// use std::pin::Pin;
/// use futures_core::task::{Context, Poll};
/// use std::marker::Unpin;
///
/// // Define a custom async sender type
/// struct MyAsyncSender<T> {
///     value: Option<T>,
/// }
///
/// // Define a custom error type
/// #[derive(Debug)]
/// struct MySendError;
///
/// // Define a simple future for our async send operation
/// struct SendFuture<T> {
///     sender: MyAsyncSender<T>,
///     value: Option<T>,
/// }
///
/// impl<T: Unpin> Future for SendFuture<T> {
///     type Output = Result<(), MySendError>;
///
///     fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
///         // In a real implementation, this would check if the send operation is ready
///         // and potentially register a waker if not
///         let this = self.get_mut();
///         let value = this.value.take().expect("value already taken");
///         this.sender.value = Some(value);
///         Poll::Ready(Ok(()))
///     }
/// }
///
/// // Implement AsyncSender for our custom type
/// impl<T: Unpin> AsyncSender<T> for MyAsyncSender<T> {
///     type Error = MySendError;
///     type SendFuture<'a> = SendFuture<T> where T: 'a, Self: 'a;
///
///     fn send(&mut self, value: T) -> Self::SendFuture<'_> {
///         SendFuture {
///             sender: MyAsyncSender { value: None },
///             value: Some(value),
///         }
///     }
/// }
///
/// // Now we can use the sender through the AsyncSender trait
/// # async fn example() {
/// let mut sender = MyAsyncSender { value: None };
/// let result = sender.send(42).await;
/// assert!(result.is_ok());
/// # }
/// ```
pub trait AsyncSender<T> {
    /// The error type that can occur during sending.
    ///
    /// This associated type allows different implementations to specify
    /// their own error types, providing flexibility while maintaining
    /// type safety.
    type Error;
    
    /// The future returned by the `send` method.
    ///
    /// This associated type represents the asynchronous operation of sending
    /// a value. It must implement the `Future` trait with an output type of
    /// `Result<(), Self::Error>`.
    type SendFuture<'a>: futures_core::Future<Output = Result<(), Self::Error>> + 'a
    where
        T: 'a,
        Self: 'a;
    
    /// Asynchronously send a value.
    ///
    /// This method sends a value of type `T` through the underlying
    /// communication mechanism asynchronously.
    ///
    /// # Parameters
    ///
    /// * `value` - The value to send.
    ///
    /// # Returns
    ///
    /// A future that resolves to:
    /// * `Ok(())` if the value was successfully sent.
    /// * `Err(Self::Error)` if an error occurred during sending.
    fn send(&mut self, value: T) -> Self::SendFuture<'_>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;

    // Implement Sender for mpsc::Sender
    impl<T> Sender<T> for mpsc::Sender<T> {
        type Error = mpsc::SendError<T>;

        fn send(&mut self, value: T) -> Result<(), Self::Error> {
            mpsc::Sender::send(self, value)
        }
    }

    // Implement Receiver for mpsc::Receiver
    impl<T> Receiver<T> for mpsc::Receiver<T> {
        type Error = mpsc::RecvError;

        fn recv(&mut self) -> Result<T, Self::Error> {
            mpsc::Receiver::recv(self)
        }
    }

    #[test]
    fn test_sender_receiver_with_mpsc() {
        // Create a channel
        let (mut tx, mut rx) = mpsc::channel::<i32>();
        
        // Send a value using the Sender trait
        let send_result = Sender::send(&mut tx, 42);
        assert!(send_result.is_ok());
        
        // Receive the value using the Receiver trait
        let recv_result = Receiver::recv(&mut rx);
        assert!(recv_result.is_ok());
        assert_eq!(recv_result.unwrap(), 42);
    }

    #[test]
    fn test_sender_receiver_with_threads() {
        // Create a channel
        let (mut tx, rx) = mpsc::channel::<String>();
        
        // Clone the sender for the thread
        let mut thread_tx = tx.clone();
        
        // Spawn a thread that sends a message
        let handle = thread::spawn(move || {
            let message = String::from("Hello from another thread!");
            let result = Sender::send(&mut thread_tx, message);
            assert!(result.is_ok());
        });
        
        // Send a message from the main thread
        let main_message = String::from("Hello from main thread!");
        let result = Sender::send(&mut tx, main_message);
        assert!(result.is_ok());
        
        // Wait for the thread to complete
        handle.join().unwrap();
        
        // Implement a simple receiver that counts messages
        struct MessageCounter {
            rx: mpsc::Receiver<String>,
            count: usize,
        }
        
        impl Receiver<String> for MessageCounter {
            type Error = mpsc::RecvError;
            
            fn recv(&mut self) -> Result<String, Self::Error> {
                let result = self.rx.recv();
                if result.is_ok() {
                    self.count += 1;
                }
                result
            }
        }
        
        // Create a message counter
        let mut counter = MessageCounter { rx, count: 0 };
        
        // Receive the first message
        let result1 = Receiver::recv(&mut counter);
        assert!(result1.is_ok());
        
        // Receive the second message
        let result2 = Receiver::recv(&mut counter);
        assert!(result2.is_ok());
        
        // Check that we received both messages
        assert_eq!(counter.count, 2);
        
        // Check that we received the expected messages
        // (order may vary since they come from different threads)
        let messages = vec![result1.unwrap(), result2.unwrap()];
        assert!(messages.contains(&String::from("Hello from main thread!")));
        assert!(messages.contains(&String::from("Hello from another thread!")));
    }

    // Test with a custom implementation
    #[test]
    fn test_custom_sender_receiver() {
        // Define a simple in-memory sender/receiver pair
        struct MemorySender<T> {
            value: Option<T>,
        }
        
        struct MemoryReceiver<T> {
            value: Option<T>,
        }
        
        // Custom error types
        #[derive(Debug)]
        struct MemorySendError;
        
        #[derive(Debug)]
        struct MemoryRecvError;
        
        impl<T> Sender<T> for MemorySender<T> {
            type Error = MemorySendError;
            
            fn send(&mut self, value: T) -> Result<(), Self::Error> {
                self.value = Some(value);
                Ok(())
            }
        }
        
        impl<T> Receiver<T> for MemoryReceiver<T> {
            type Error = MemoryRecvError;
            
            fn recv(&mut self) -> Result<T, Self::Error> {
                self.value.take().ok_or(MemoryRecvError)
            }
        }
        
        // Create a sender/receiver pair
        let mut sender = MemorySender { value: None };
        let mut receiver = MemoryReceiver { value: None };
        
        // Send a value
        let send_result = sender.send(42);
        assert!(send_result.is_ok());
        
        // Transfer the value (simulating a channel)
        receiver.value = sender.value.take();
        
        // Receive the value
        let recv_result = receiver.recv();
        assert!(recv_result.is_ok());
        assert_eq!(recv_result.unwrap(), 42);
        
        // Try to receive again (should fail)
        let recv_result2 = receiver.recv();
        assert!(recv_result2.is_err());
    }
}

#[cfg(test)]
mod async_tests {
    use super::*;
    use futures_core::future::Future;
    use std::pin::Pin;
    use futures_core::task::{Context, Poll};
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;

    // Simple in-memory async sender/receiver implementation for testing
    struct AsyncMemorySender<T> {
        value: Arc<Mutex<Option<T>>>,
    }

    struct AsyncMemoryReceiver<T> {
        value: Arc<Mutex<Option<T>>>,
    }

    #[derive(Debug)]
    struct AsyncMemorySendError;

    #[derive(Debug)]
    struct AsyncMemoryRecvError;

    // Future implementations
    struct AsyncMemorySendFuture<T> {
        value: Option<T>,
        shared: Arc<Mutex<Option<T>>>,
    }

    impl<T> Future for AsyncMemorySendFuture<T> {
        type Output = Result<(), AsyncMemorySendError>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            // Get a mutable reference to the unpinned fields
            let this = unsafe { self.get_unchecked_mut() };
            
            if let Some(value) = this.value.take() {
                let mut shared = this.shared.lock().unwrap();
                *shared = Some(value);
                Poll::Ready(Ok(()))
            } else {
                Poll::Ready(Err(AsyncMemorySendError))
            }
        }
    }

    struct AsyncMemoryRecvFuture<T> {
        shared: Arc<Mutex<Option<T>>>,
    }

    impl<T> Future for AsyncMemoryRecvFuture<T> {
        type Output = Result<T, AsyncMemoryRecvError>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut shared = self.shared.lock().unwrap();
            match shared.take() {
                Some(value) => Poll::Ready(Ok(value)),
                None => Poll::Ready(Err(AsyncMemoryRecvError)),
            }
        }
    }

    // Implement AsyncSender for AsyncMemorySender
    impl<T> AsyncSender<T> for AsyncMemorySender<T> {
        type Error = AsyncMemorySendError;
        type SendFuture<'a> = AsyncMemorySendFuture<T> where T: 'a, Self: 'a;

        fn send(&mut self, value: T) -> Self::SendFuture<'_> {
            AsyncMemorySendFuture {
                value: Some(value),
                shared: Arc::clone(&self.value),
            }
        }
    }

    // Implement AsyncReceiver for AsyncMemoryReceiver
    impl<T> AsyncReceiver<T> for AsyncMemoryReceiver<T> {
        type Error = AsyncMemoryRecvError;
        type RecvFuture<'a> = AsyncMemoryRecvFuture<T> where T: 'a, Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            AsyncMemoryRecvFuture {
                shared: Arc::clone(&self.value),
            }
        }
    }

    // Implement AsyncSender for tokio mpsc::Sender
    impl<T> AsyncSender<T> for mpsc::Sender<T>
    where
        T: Send + 'static,
    {
        type Error = mpsc::error::SendError<T>;
        
        type SendFuture<'a> = Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'a>> where T: 'a, Self: 'a;

        fn send(&mut self, value: T) -> Self::SendFuture<'_> {
            let sender = self.clone();
            Box::pin(async move {
                sender.send(value).await
            })
        }
    }

    // Implement AsyncReceiver for tokio mpsc::Receiver
    // Custom error type for tokio mpsc::Receiver
    #[derive(Debug)]
    pub struct TokioRecvError;
    
    impl<T> AsyncReceiver<T> for mpsc::Receiver<T>
    where
        T: Send + 'static,
    {
        type Error = TokioRecvError;
        
        type RecvFuture<'a> = Pin<Box<dyn Future<Output = Result<T, Self::Error>> + Send + 'a>> where T: 'a, Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            let receiver = self;
            Box::pin(async move {
                receiver.recv().await.ok_or(TokioRecvError)
            })
        }
    }

    #[tokio::test]
    async fn test_async_memory_sender_receiver() {
        // Create a shared memory location
        let shared = Arc::new(Mutex::new(None::<i32>));
        
        // Create sender and receiver
        let mut sender = AsyncMemorySender { value: Arc::clone(&shared) };
        let mut receiver = AsyncMemoryReceiver { value: Arc::clone(&shared) };
        
        // Send a value
        let send_result = sender.send(42).await;
        assert!(send_result.is_ok());
        
        // Receive the value
        let recv_result = receiver.recv().await;
        assert!(recv_result.is_ok());
        assert_eq!(recv_result.unwrap(), 42);
        
        // Try to receive again (should fail)
        let recv_result2 = receiver.recv().await;
        assert!(recv_result2.is_err());
    }

    #[tokio::test]
    async fn test_async_sender_receiver_with_tokio_mpsc() {
        // Create a tokio channel
        let (mut tx, mut rx) = mpsc::channel::<i32>(10);
        
        // Send a value using the AsyncSender trait
        let send_result = AsyncSender::send(&mut tx, 42).await;
        assert!(send_result.is_ok());
        
        // Receive the value using the AsyncReceiver trait
        let recv_result = AsyncReceiver::recv(&mut rx).await;
        assert!(recv_result.is_ok());
        assert_eq!(recv_result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_async_sender_receiver_with_multiple_values() {
        // Create a tokio channel
        let (mut tx, mut rx) = mpsc::channel::<i32>(10);
        
        // Send multiple values
        for i in 0..5 {
            let send_result = AsyncSender::send(&mut tx, i).await;
            assert!(send_result.is_ok());
        }
        
        // Receive the values
        for i in 0..5 {
            let recv_result = AsyncReceiver::recv(&mut rx).await;
            assert!(recv_result.is_ok());
            assert_eq!(recv_result.unwrap(), i);
        }
    }
}