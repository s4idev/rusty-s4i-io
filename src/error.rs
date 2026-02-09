//! Error types for rusty-s4i-io

use std::fmt;

/// Result type alias for library operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the rusty-s4i-io library
#[derive(Debug)]
pub enum Error {
    /// I/O error
    Io(std::io::Error),

    /// Connection error
    Connection(String),

    /// Configuration error
    Configuration(String),

    /// Protocol error
    Protocol(String),

    /// Timeout error
    Timeout,

    /// Not supported on this platform
    NotSupported(String),

    /// Generic error
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Connection(msg) => write!(f, "Connection error: {}", msg),
            Error::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            Error::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            Error::Timeout => write!(f, "Operation timed out"),
            Error::NotSupported(feature) => write!(f, "Feature not supported: {}", feature),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Connection("failed to connect".to_string());
        assert_eq!(err.to_string(), "Connection error: failed to connect");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }
}
