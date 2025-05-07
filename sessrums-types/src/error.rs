//! Error types for session type operations.
use thiserror::Error;
use std::io;
use std::sync::PoisonError;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Transport error: {0}")]
    Transport(#[from] io::Error),

    #[error("Serialization/Deserialization error: {0}")]
    Serialization(bincode::Error),

    #[error("Protocol violation: {0}")]
    ProtocolViolation(String),

    #[error("Channel closed unexpectedly")]
    UnexpectedClose,

    #[error("Operation timed out after {0:?}")]
    Timeout(std::time::Duration),

    #[error("Lock was poisoned")]
    LockPoisoned,
}

impl From<bincode::Error> for SessionError {
    fn from(err: bincode::Error) -> Self {
        SessionError::Serialization(err)
    }
}

impl<T> From<PoisonError<T>> for SessionError {
    fn from(_err: PoisonError<T>) -> Self {
        SessionError::LockPoisoned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_error_messages() {
        // Test protocol violation
        let err = SessionError::ProtocolViolation(
            "Cannot send after session end".to_string()
        );
        assert!(err.to_string().contains("Protocol violation"));

        // Test timeout
        let err = SessionError::Timeout(Duration::from_secs(5));
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn test_error_conversion() {
        // Test io::Error conversion
        let io_err = io::Error::new(io::ErrorKind::ConnectionReset, "Connection reset");
        let session_err: SessionError = io_err.into();
        assert!(matches!(session_err, SessionError::Transport(_)));
    }
}