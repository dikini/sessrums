//! Basic IO traits for session types.
//!
//! This module defines the fundamental IO traits that abstract over different
//! IO implementations. These traits provide a common interface for sending and
//! receiving values, which is essential for implementing the session type
//! communication channels.
//!
//! The traits defined here are intentionally simple and synchronous. Asynchronous
//! versions will be added in Phase 4 of the project.

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