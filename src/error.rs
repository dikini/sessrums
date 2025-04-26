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
/// use sez::error::Error;
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
    Io(io::Error),

    /// A protocol violation occurred.
    ///
    /// This variant represents errors related to protocol violations, such as
    /// unexpected messages or type mismatches.
    Protocol(&'static str),

    /// A connection error occurred.
    ///
    /// This variant represents errors related to connection establishment or
    /// termination.
    Connection(&'static str),

    /// A serialization error occurred.
    ///
    /// This variant represents errors that occur when serializing data to be
    /// sent over the channel.
    Serialization(&'static str),

    /// A deserialization error occurred.
    ///
    /// This variant represents errors that occur when deserializing received
    /// data.
    Deserialization(&'static str),

    /// The channel was closed.
    ///
    /// This variant represents the error that occurs when attempting to
    /// communicate on a closed channel.
    ChannelClosed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            Error::Connection(msg) => write!(f, "Connection error: {}", msg),
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::Deserialization(msg) => write!(f, "Deserialization error: {}", msg),
            Error::ChannelClosed => write!(f, "Channel closed"),
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
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(format!("{}", err), "Channel closed");
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
    }
}