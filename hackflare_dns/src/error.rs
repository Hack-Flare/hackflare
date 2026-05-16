use std::fmt;

/// Errors that can occur in the DNS crate.
#[derive(Debug)]
pub enum DnsError {
    /// No persistence backend was configured.
    PersistenceUnconfigured,
    /// A persistence backend operation failed.
    PersistenceOperation(String),
}

impl fmt::Display for DnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PersistenceUnconfigured => f.pad("no persistence backend configured"),
            Self::PersistenceOperation(msg) => write!(f, "persistence operation failed: {msg}"),
        }
    }
}

impl std::error::Error for DnsError {}
