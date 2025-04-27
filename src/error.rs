//! Error types for session types.
//!
//! This module defines the error types that can occur during protocol communication.
//! These errors represent various failure conditions that might arise when using
//! session-typed channels for communication.

use std::error::Error as StdError;
use std::fmt;
use std::io;

/// Errors that can occur during protocol communication.
///
/// This enum represents the various error conditions that might arise when
/// using session-typed channels for communication.
///
/// # Examples
///
/// ```
/// use sessrums::error::Error;
/// use std::io;
///
/// // Creating an IO error
/// let io_err = io::Error::new(io::ErrorKind::Other, "IO operation failed");
/// let protocol_err = Error::Io(io_err);
///
/// // Creating a protocol error
/// let protocol_err = Error::Protocol("Unexpected message received");
///
/// // Creating a connection error
/// let conn_err = Error::Connection("Connection closed unexpectedly");
///
/// // Creating a serialization error
/// let ser_err = Error::Serialization("Failed to serialize data");
///
/// // Creating a deserialization error
/// let deser_err = Error::Deserialization("Failed to deserialize data");
///
/// // Creating a closed channel error
/// let closed_err = Error::ChannelClosed;
/// ```
#[derive(Debug)]
pub enum Error {
    /// An error occurred in the underlying IO implementation.
    ///
    /// This variant represents errors that occur during the actual sending or
    /// receiving of data through the underlying IO implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::error::Error;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::ConnectionReset, "Connection reset by peer");
    /// let err = Error::Io(io_err);
    /// ```
    Io(io::Error),

    /// A protocol violation occurred.
    ///
    /// This variant represents errors related to protocol violations, such as
    /// unexpected messages or type mismatches.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::error::Error;
    ///
    /// let err = Error::Protocol("Received unexpected message type");
    /// ```
    Protocol(&'static str),

    /// A connection error occurred.
    ///
    /// This variant represents errors related to connection establishment or
    /// termination.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::error::Error;
    ///
    /// let err = Error::Connection("Failed to establish connection");
    /// ```
    Connection(&'static str),

    /// A serialization error occurred.
    ///
    /// This variant represents errors that occur when serializing data to be
    /// sent over the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::error::Error;
    ///
    /// let err = Error::Serialization("Failed to serialize complex data structure");
    /// ```
    Serialization(&'static str),

    /// A deserialization error occurred.
    ///
    /// This variant represents errors that occur when deserializing received
    /// data.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::error::Error;
    ///
    /// let err = Error::Deserialization("Failed to deserialize received data");
    /// ```
    Deserialization(&'static str),

    /// The channel was closed.
    ///
    /// This variant represents the error that occurs when attempting to
    /// communicate on a closed channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::error::Error;
    ///
    /// let err = Error::ChannelClosed;
    /// ```
    ChannelClosed,

    /// A timeout occurred during a communication operation.
    ///
    /// This variant represents errors that occur when a communication operation
    /// times out.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::error::Error;
    /// use std::time::Duration;
    ///
    /// let err = Error::Timeout(Duration::from_secs(30));
    /// ```
    Timeout(std::time::Duration),

    /// An error occurred during protocol negotiation.
    ///
    /// This variant represents errors that occur during the negotiation phase
    /// of a protocol, such as version mismatches or unsupported features.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::error::Error;
    ///
    /// let err = Error::Negotiation("Protocol version mismatch");
    /// ```
    Negotiation(&'static str),

    /// An error occurred due to a protocol state mismatch.
    ///
    /// This variant represents errors that occur when the protocol state
    /// doesn't match the expected state for an operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sessrums::error::Error;
    ///
    /// let err = Error::StateMismatch("Expected Send state, but protocol is in Recv state");
    /// ```
    StateMismatch(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            Error::Connection(msg) => write!(f, "Connection error: {}", msg),
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::Deserialization(msg) => write!(f, "Deserialization error: {}", msg),
            Error::ChannelClosed => write!(f, "Channel closed: The channel has been closed and cannot be used for further communication"),
            Error::Timeout(duration) => write!(f, "Timeout error: Operation timed out after {:?}", duration),
            Error::Negotiation(msg) => write!(f, "Protocol negotiation error: {}", msg),
            Error::StateMismatch(msg) => write!(f, "Protocol state mismatch: {}", msg),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }

    fn description(&self) -> &str {
        match self {
            Error::Io(_) => "IO error during communication",
            Error::Protocol(_) => "Protocol violation error",
            Error::Connection(_) => "Connection establishment or termination error",
            Error::Serialization(_) => "Data serialization error",
            Error::Deserialization(_) => "Data deserialization error",
            Error::ChannelClosed => "Channel closed error",
            Error::Timeout(_) => "Communication timeout error",
            Error::Negotiation(_) => "Protocol negotiation error",
            Error::StateMismatch(_) => "Protocol state mismatch error",
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

/// Convert a timeout duration to a session type error.
impl From<std::time::Duration> for Error {
    fn from(duration: std::time::Duration) -> Self {
        Error::Timeout(duration)
    }
}

/// Result type for session type operations.
///
/// This type alias makes it easier to work with results that may contain
/// session type errors.
///
/// # Examples
///
/// ```
/// use sessrums::error::{Error, Result};
///
/// fn example() -> Result<()> {
///     // Some operation that might fail
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_error_display() {
        // Test the Display implementation for each error variant
        let io_err = io::Error::new(io::ErrorKind::Other, "test IO error");
        let err = Error::Io(io_err);
        assert!(format!("{}", err).contains("IO error"));

        let err = Error::Protocol("test protocol error");
        assert_eq!(format!("{}", err), "Protocol error: test protocol error");

        let err = Error::Connection("test connection error");
        assert_eq!(format!("{}", err), "Connection error: test connection error");

        let err = Error::Serialization("test serialization error");
        assert_eq!(format!("{}", err), "Serialization error: test serialization error");

        let err = Error::Deserialization("test deserialization error");
        assert_eq!(format!("{}", err), "Deserialization error: test deserialization error");

        let err = Error::ChannelClosed;
        assert!(format!("{}", err).contains("Channel closed"));

        let err = Error::Timeout(Duration::from_secs(10));
        assert!(format!("{}", err).contains("Timeout error"));
        assert!(format!("{}", err).contains("10s"));

        let err = Error::Negotiation("version mismatch");
        assert_eq!(format!("{}", err), "Protocol negotiation error: version mismatch");

        let err = Error::StateMismatch("expected Send state");
        assert_eq!(format!("{}", err), "Protocol state mismatch: expected Send state");
    }

    #[test]
    fn test_error_from_io_error() {
        // Test the From<io::Error> implementation
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = Error::from(io_err);
        
        match err {
            Error::Io(e) => {
                assert_eq!(e.kind(), io::ErrorKind::NotFound);
                assert_eq!(e.to_string(), "file not found");
            },
            _ => panic!("Expected Error::Io variant"),
        }
    }

    #[test]
    fn test_error_from_duration() {
        // Test the From<Duration> implementation
        let duration = Duration::from_secs(30);
        let err = Error::from(duration);
        
        match err {
            Error::Timeout(d) => {
                assert_eq!(d, Duration::from_secs(30));
            },
            _ => panic!("Expected Error::Timeout variant"),
        }
    }

    #[test]
    fn test_error_source() {
        // Test the source method for Error::Io
        let io_err = io::Error::new(io::ErrorKind::Other, "test IO error");
        let err = Error::Io(io_err);
        assert!(err.source().is_some());

        // Test the source method for other variants
        let err = Error::Protocol("test protocol error");
        assert!(err.source().is_none());

        let err = Error::Connection("test connection error");
        assert!(err.source().is_none());

        let err = Error::Serialization("test serialization error");
        assert!(err.source().is_none());

        let err = Error::Deserialization("test deserialization error");
        assert!(err.source().is_none());

        let err = Error::ChannelClosed;
        assert!(err.source().is_none());

        let err = Error::Timeout(Duration::from_secs(10));
        assert!(err.source().is_none());

        let err = Error::Negotiation("version mismatch");
        assert!(err.source().is_none());

        let err = Error::StateMismatch("expected Send state");
        assert!(err.source().is_none());
    }

    #[test]
    fn test_result_type_alias() {
        // Test that the Result type alias works correctly
        fn returns_ok() -> Result<i32> {
            Ok(42)
        }

        fn returns_err() -> Result<i32> {
            Err(Error::ChannelClosed)
        }

        let ok_result = returns_ok();
        assert!(ok_result.is_ok());
        assert_eq!(ok_result.unwrap(), 42);

        let err_result = returns_err();
        assert!(err_result.is_err());
        match err_result.unwrap_err() {
            Error::ChannelClosed => {},
            _ => panic!("Expected Error::ChannelClosed"),
        }
    }
}