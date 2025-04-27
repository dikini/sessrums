//! Connection establishment for session types.
//!
//! This module provides functions and types for establishing connections between
//! two endpoints using session types. It includes wrappers for common stream types
//! that implement the `AsyncSender` and `AsyncReceiver` traits, allowing them to be
//! used with the session type system.

use crate::error::Error;
use crate::io::{AsyncReceiver, AsyncSender};
use crate::proto::Protocol;
use crate::chan::Chan;

use futures_core::future::Future;
use std::io;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A trait for types that can be used to establish a connection.
///
/// This trait is implemented by types that provide connection information
/// for establishing a connection between two endpoints.
///
/// # Type Parameters
///
/// * `IO` - The IO implementation type that will be used for communication.
///
pub trait ConnectInfo {
    /// The IO implementation type that will be used for communication.
    type IO;
    
    /// Establishes a connection using the provided connection information.
    ///
    /// # Returns
    ///
    /// A result containing the IO implementation if successful, or an error if the connection
    /// could not be established.
    fn connect(&self) -> std::io::Result<Self::IO>;
}

/// A wrapper for a bidirectional stream that implements both `AsyncSender` and `AsyncReceiver`.
///
/// This wrapper can be used to adapt any stream type that provides read and write methods
/// to work with the session type system.
///
/// # Type Parameters
///
/// * `S` - The underlying stream type.
/// * `T` - The type of values being sent and received.
///
/// # Examples
///
/// ```no_run
/// use sez::connect::StreamWrapper;
/// use sez::chan::Chan;
/// use sez::proto::{Send, Recv, End};
/// use std::net::TcpStream;
///
/// // Define a protocol
/// type MyProtocol = Send<String, Recv<i32, End>>;
///
/// // Create a TCP stream
/// let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
///
/// // Wrap the stream
/// let wrapper = StreamWrapper::<TcpStream, String>::new(stream);
///
/// // Create a channel with the wrapped stream
/// let chan = Chan::<MyProtocol, _>::new(wrapper);
/// ```
pub struct StreamWrapper<S, T> {
    stream: S,
    _marker: PhantomData<T>,
}

impl<S, T> StreamWrapper<S, T> {
    /// Creates a new stream wrapper.
    ///
    /// # Parameters
    ///
    /// * `stream` - The stream to wrap.
    ///
    /// # Returns
    ///
    /// A new `StreamWrapper` instance.
    pub fn new(stream: S) -> Self {
        StreamWrapper {
            stream,
            _marker: PhantomData,
        }
    }

    /// Gets a reference to the underlying stream.
    ///
    /// # Returns
    ///
    /// A reference to the underlying stream.
    pub fn stream(&self) -> &S {
        &self.stream
    }

    /// Gets a mutable reference to the underlying stream.
    ///
    /// # Returns
    ///
    /// A mutable reference to the underlying stream.
    pub fn stream_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    /// Consumes the wrapper and returns the underlying stream.
    ///
    /// # Returns
    ///
    /// The underlying stream.
    pub fn into_inner(self) -> S {
        self.stream
    }
}

/// A future for sending a value through a stream.
pub struct StreamSendFuture<'a, S, T> {
    stream: &'a mut S,
    value: Option<T>,
}

/// A future for receiving a value from a stream.
pub struct StreamRecvFuture<'a, S, T> {
    stream: &'a mut S,
    _marker: PhantomData<T>,
}

// Implementation for TCP streams
#[cfg(feature = "tcp")]
mod tcp {
    use super::*;
    use std::net::TcpStream;
    use std::io::{Read, Write};
    use serde::{Serialize, Deserialize};

    impl<T> AsyncSender<T> for StreamWrapper<TcpStream, T>
    where
        T: Serialize + std::marker::Unpin,
    {
        type Error = Error;
        type SendFuture<'a> = StreamSendFuture<'a, TcpStream, T> where T: 'a, Self: 'a;

        fn send(&mut self, value: T) -> Self::SendFuture<'_> {
            StreamSendFuture {
                stream: &mut self.stream,
                value: Some(value),
            }
        }
    }

    impl<T> AsyncReceiver<T> for StreamWrapper<TcpStream, T>
    where
        T: for<'de> Deserialize<'de> + std::marker::Unpin,
    {
        type Error = Error;
        type RecvFuture<'a> = StreamRecvFuture<'a, TcpStream, T> where T: 'a, Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            StreamRecvFuture {
                stream: &mut self.stream,
                _marker: PhantomData,
            }
        }
    }

    impl<'a, T> Future for StreamSendFuture<'a, TcpStream, T>
    where
        T: Serialize + std::marker::Unpin,
    {
        type Output = Result<(), Error>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            
            if let Some(value) = this.value.take() {
                // Serialize the value
                let serialized = bincode::serialize(&value)
                    .map_err(|_e| Error::Serialization("Failed to serialize value"))?;
                
                // Write the length of the serialized data as a u32
                let len = serialized.len() as u32;
                let len_bytes = len.to_be_bytes();
                
                // Write the length
                this.stream.write_all(&len_bytes)
                    .map_err(|e| Error::Io(e))?;
                
                // Write the serialized data
                this.stream.write_all(&serialized)
                    .map_err(|e| Error::Io(e))?;
                
                // Flush the stream
                this.stream.flush()
                    .map_err(|e| Error::Io(e))?;
                
                Poll::Ready(Ok(()))
            } else {
                Poll::Ready(Err(Error::Protocol("Value already taken")))
            }
        }
    }

    impl<'a, T> Future for StreamRecvFuture<'a, TcpStream, T>
    where
        T: for<'de> Deserialize<'de> + std::marker::Unpin,
    {
        type Output = Result<T, Error>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            
            // Read the length of the serialized data
            let mut len_bytes = [0u8; 4];
            this.stream.read_exact(&mut len_bytes)
                .map_err(|e| Error::Io(e))?;
            
            let len = u32::from_be_bytes(len_bytes) as usize;
            
            // Read the serialized data
            let mut buffer = vec![0u8; len];
            this.stream.read_exact(&mut buffer)
                .map_err(|e| Error::Io(e))?;
            
            // Deserialize the data
            let value = bincode::deserialize(&buffer)
                .map_err(|_e| Error::Deserialization("Failed to deserialize value"))?;
            
            Poll::Ready(Ok(value))
        }
    }
}

// Implementation for Unix domain sockets
#[cfg(feature = "unix")]
mod unix {
    use super::*;
    use std::os::unix::net::UnixStream;
    use std::io::{Read, Write};
    use serde::{Serialize, Deserialize};

    impl<T> AsyncSender<T> for StreamWrapper<UnixStream, T>
    where
        T: Serialize + std::marker::Unpin,
    {
        type Error = Error;
        type SendFuture<'a> = StreamSendFuture<'a, UnixStream, T> where T: 'a, Self: 'a;

        fn send(&mut self, value: T) -> Self::SendFuture<'_> {
            StreamSendFuture {
                stream: &mut self.stream,
                value: Some(value),
            }
        }
    }

    impl<T> AsyncReceiver<T> for StreamWrapper<UnixStream, T>
    where
        T: for<'de> Deserialize<'de> + std::marker::Unpin,
    {
        type Error = Error;
        type RecvFuture<'a> = StreamRecvFuture<'a, UnixStream, T> where T: 'a, Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            StreamRecvFuture {
                stream: &mut self.stream,
                _marker: PhantomData,
            }
        }
    }

    impl<'a, T> Future for StreamSendFuture<'a, UnixStream, T>
    where
        T: Serialize + std::marker::Unpin,
    {
        type Output = Result<(), Error>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            
            if let Some(value) = this.value.take() {
                // Serialize the value
                let serialized = bincode::serialize(&value)
                    .map_err(|e| Error::Serialization("Failed to serialize value"))?;
                
                // Write the length of the serialized data as a u32
                let len = serialized.len() as u32;
                let len_bytes = len.to_be_bytes();
                
                // Write the length
                this.stream.write_all(&len_bytes)
                    .map_err(|e| Error::Io(e))?;
                
                // Write the serialized data
                this.stream.write_all(&serialized)
                    .map_err(|e| Error::Io(e))?;
                
                // Flush the stream
                this.stream.flush()
                    .map_err(|e| Error::Io(e))?;
                
                Poll::Ready(Ok(()))
            } else {
                Poll::Ready(Err(Error::Protocol("Value already taken")))
            }
        }
    }

    impl<'a, T> Future for StreamRecvFuture<'a, UnixStream, T>
    where
        T: for<'de> Deserialize<'de> + std::marker::Unpin,
    {
        type Output = Result<T, Error>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            
            // Read the length of the serialized data
            let mut len_bytes = [0u8; 4];
            this.stream.read_exact(&mut len_bytes)
                .map_err(|e| Error::Io(e))?;
            
            let len = u32::from_be_bytes(len_bytes) as usize;
            
            // Read the serialized data
            let mut buffer = vec![0u8; len];
            this.stream.read_exact(&mut buffer)
                .map_err(|e| Error::Io(e))?;
            
            // Deserialize the data
            let value = bincode::deserialize(&buffer)
                .map_err(|e| Error::Deserialization("Failed to deserialize value"))?;
            
            Poll::Ready(Ok(value))
        }
    }
}

/// Establishes a connection between two endpoints using the given protocol.
///
/// This function creates a channel with the specified protocol and stream.
///
/// # Type Parameters
///
/// * `P` - The protocol type.
/// * `S` - The stream type.
/// * `T` - The type of values being sent and received.
///
/// # Parameters
///
/// * `stream` - The stream to use for communication.
///
/// # Returns
///
/// A channel with the specified protocol and stream.
///
/// # Examples
///
/// ```no_run
/// use sez::connect::connect;
/// use sez::proto::{Send, Recv, End};
/// use std::net::TcpStream;
///
/// // Define a protocol
/// type MyProtocol = Send<String, Recv<i32, End>>;
///
/// // Create a TCP stream
/// let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
///
/// // Establish a connection
/// let chan = connect::<MyProtocol, _, String>(stream);
/// ```
pub fn connect<P, S, T>(stream: S) -> Chan<P, StreamWrapper<S, T>>
where
    P: Protocol,
{
    let wrapper = StreamWrapper::new(stream);
    Chan::new(wrapper)
}

/// Establishes a server connection by accepting a connection from a listener.
///
/// This function accepts a connection from the given listener and creates a channel
/// with the specified protocol.
///
/// # Type Parameters
///
/// * `P` - The protocol type.
/// * `L` - The listener type.
/// * `S` - The stream type.
/// * `T` - The type of values being sent and received.
///
/// # Parameters
///
/// * `listener` - The listener to accept connections from.
///
/// # Returns
///
/// A result containing a channel with the specified protocol and the accepted stream.
///
#[cfg(feature = "tcp")]
pub fn accept<P, L, S, T>(listener: &L) -> io::Result<Chan<P, StreamWrapper<S, T>>>
where
    P: Protocol,
    L: std::net::ToSocketAddrs,
    S: From<std::net::TcpStream>,
{
    let tcp_listener = std::net::TcpListener::bind(listener)?;
    let (stream, _) = tcp_listener.accept()?;
    let stream = S::from(stream);
    Ok(connect::<P, S, T>(stream))
}

/// Establishes a connection with a specific protocol using the provided connection information.
///
/// This function uses the provided connection information to establish a connection and
/// creates a channel with the specified protocol.
///
/// # Type Parameters
///
/// * `P` - The protocol type.
/// * `IO` - The IO implementation type.
/// * `C` - The connection info type.
///
/// # Parameters
///
/// * `conn_info` - The connection information to use for establishing the connection.
///
/// # Returns
///
/// A result containing a channel with the specified protocol if successful, or an error
/// if the connection could not be established.
///
/// # Examples
///
/// ```no_run
/// use sez::connect::{ConnectInfo, connect_with_protocol};
/// use sez::proto::{Send, Recv, End};
/// use sez::chan::Chan;
/// use std::net::{SocketAddr, TcpStream};
/// use sez::connect::StreamWrapper;
///
/// // Define a protocol
/// type MyProtocol = Send<String, Recv<i32, End>>;
///
/// // Define a connection info type
/// struct TcpConnectInfo {
///     addr: SocketAddr,
/// }
///
/// impl TcpConnectInfo {
///     fn new(addr: SocketAddr) -> Self {
///         TcpConnectInfo { addr }
///     }
/// }
///
/// impl ConnectInfo for TcpConnectInfo {
///     type IO = StreamWrapper<TcpStream, String>;
///
///     fn connect(&self) -> std::io::Result<Self::IO> {
///         let stream = TcpStream::connect(&self.addr)?;
///         Ok(StreamWrapper::new(stream))
///     }
/// }
///
/// // Use the connection info to establish a connection
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let addr = "127.0.0.1:8080".parse::<SocketAddr>()?;
/// let conn_info = TcpConnectInfo::new(addr);
/// let chan = connect_with_protocol::<MyProtocol, _, _>(conn_info).await?;
/// # Ok(())
/// # }
/// ```
pub async fn connect_with_protocol<P, IO, C>(conn_info: C) -> Result<Chan<P, IO>, Error>
where
    P: Protocol,
    C: ConnectInfo<IO = IO>,
{
    match conn_info.connect() {
        Ok(io) => Ok(Chan::new(io)),
        Err(_) => Err(Error::Connection("Failed to establish connection")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{Send, Recv, End};
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    // A mock stream for testing
    struct MockStream {
        read_data: Vec<u8>,
        write_data: Vec<u8>,
    }

    impl MockStream {
        fn new(read_data: Vec<u8>) -> Self {
            MockStream {
                read_data,
                write_data: Vec::new(),
            }
        }
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let n = std::cmp::min(buf.len(), self.read_data.len());
            buf[..n].copy_from_slice(&self.read_data[..n]);
            self.read_data = self.read_data.split_off(n);
            Ok(n)
        }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_data.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // Test the StreamWrapper
    #[test]
    fn test_stream_wrapper() {
        let stream = MockStream::new(Vec::new());
        let wrapper = StreamWrapper::<MockStream, i32>::new(stream);
        
        assert!(wrapper.stream().read_data.is_empty());
        assert!(wrapper.stream().write_data.is_empty());
    }

    // Test the connect function
    #[test]
    fn test_connect() {
        let stream = MockStream::new(Vec::new());
        let chan = connect::<End, _, i32>(stream);
        
        // Check that the channel was created successfully
        assert!(chan.io().stream().read_data.is_empty());
        assert!(chan.io().stream().write_data.is_empty());
    }

    // Integration test with actual TCP streams
    #[cfg(feature = "tcp")]
    // This test is disabled because it requires a running server
    // It's included as an example of how to use the connection functions
    #[cfg(feature = "tcp")]
    #[ignore]
    #[tokio::test]
    async fn test_tcp_integration() {
        // Define a protocol for string messages in both directions
        type ServerProto = Recv<String, Send<String, End>>;
        type ClientProto = Send<String, Recv<String, End>>;
        
        // Start a server in a separate task
        let server_task = tokio::spawn(async {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            
            // Signal that the server is ready
            println!("Server listening on {}", addr);
            
            // Accept a connection
            let (stream, _) = listener.accept().unwrap();
            let wrapper = StreamWrapper::<TcpStream, String>::new(stream);
            let chan = Chan::<ServerProto, _>::new(wrapper);
            
            // Receive a string
            let (msg, chan) = chan.recv().await.unwrap();
            assert_eq!(msg, "Hello, server!");
            
            // Send a string response
            let chan = chan.send("Hello, client!".to_string()).await.unwrap();
            
            // Close the channel
            chan.close().unwrap();
        });
        
        // Give the server time to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Connect to the server
        let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        let wrapper = StreamWrapper::<TcpStream, String>::new(stream);
        let chan = Chan::<ClientProto, _>::new(wrapper);
        
        // Send a string
        let chan = chan.send("Hello, server!".to_string()).await.unwrap();
        
        // Receive a string response
        let (response, chan) = chan.recv().await.unwrap();
        assert_eq!(response, "Hello, client!");
        
        // Close the channel
        chan.close().unwrap();
        
        // Wait for the server to finish
        server_task.await.unwrap();
    }
}