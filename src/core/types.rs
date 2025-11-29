use serde::{Serialize, Deserialize};
use std::fmt;

/// Node identifier
pub type NodeId = u32;

/// Ant agent identifier
pub type AntId = u64;

/// Timestamp in seconds since epoch
pub type Timestamp = u64;

/// Consensus value - represents the value nodes are trying to agree on
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConsensusValue {
    /// SHA-256 hash of the value
    pub hash: [u8; 32],
}

impl ConsensusValue {
    /// Create a new consensus value from bytes
    pub fn from_bytes(data: &[u8]) -> Self {
        use crate::crypto::hash_sha256;
        let hash = hash_sha256(data);
        Self { hash }
    }

    /// Create a consensus value from a string
    pub fn from_string(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    /// Get the hash as a hex string
    pub fn to_hex(&self) -> String {
        self.hash.iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

impl fmt::Display for ConsensusValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Error types for the consensus system
#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Consensus timeout")]
    Timeout,

    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ConsensusError>;

