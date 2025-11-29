use crate::crypto::signing::{sign_message, verify_signature, PublicKey, Signature};
use crate::core::types::{ConsensusValue, NodeId, Timestamp};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Pheromone intensity threshold for consensus
pub const CONSENSUS_THRESHOLD: f64 = 0.8;

/// Minimum pheromone intensity before removal
pub const MIN_PHEROMONE_INTENSITY: f64 = 0.01;

/// Initial pheromone intensity when emitted
pub const INITIAL_PHEROMONE_INTENSITY: f64 = 1.0;

/// Pheromone structure - represents a digital trail left by nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pheromone {
    /// Timestamp when pheromone was created
    pub timestamp: Timestamp,
    
    /// Intensity of the pheromone (0.0 to 1.0)
    pub intensity: f64,
    
    /// Node that emitted this pheromone
    pub source: NodeId,
    
    /// Consensus value this pheromone represents
    pub value: ConsensusValue,
    
    /// Digital signature for verification
    pub signature: Signature,
}

impl Pheromone {
    /// Create a new pheromone with a consensus value
    pub fn new(
        value: ConsensusValue,
        source: NodeId,
        private_key: &[u8],
    ) -> crate::core::types::Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| crate::core::types::ConsensusError::Internal(e.to_string()))?
            .as_secs();

        let message = Self::create_message(&value, timestamp, source);
        // For now, create a dummy signature since we need proper key management
        let signature = if private_key.is_empty() {
            vec![0u8; 64] // Dummy signature
        } else {
            sign_message(&message, private_key)
                .map_err(|e| crate::core::types::ConsensusError::Crypto(e.to_string()))?
        };

        Ok(Self {
            timestamp,
            intensity: INITIAL_PHEROMONE_INTENSITY,
            source,
            value,
            signature,
        })
    }

    /// Verify the pheromone's signature
    pub fn verify(&self, public_key: &PublicKey) -> bool {
        let message = Self::create_message(&self.value, self.timestamp, self.source);
        verify_signature(&message, &self.signature, public_key).unwrap_or(false)
    }

    /// Evaporate the pheromone (reduce intensity)
    pub fn evaporate(&mut self, rate: f64) {
        self.intensity *= 1.0 - rate;
    }

    /// Get the current strength of the pheromone
    pub fn strength(&self) -> f64 {
        self.intensity
    }

    /// Check if pheromone is strong enough for consensus
    pub fn is_strong_enough(&self) -> bool {
        self.intensity >= CONSENSUS_THRESHOLD
    }

    /// Check if pheromone should be removed (too weak)
    pub fn should_remove(&self) -> bool {
        self.intensity < MIN_PHEROMONE_INTENSITY
    }

    /// Create message for signing
    fn create_message(value: &ConsensusValue, timestamp: Timestamp, source: NodeId) -> Vec<u8> {
        let mut message = Vec::new();
        message.extend_from_slice(&value.hash);
        message.extend_from_slice(&timestamp.to_be_bytes());
        message.extend_from_slice(&source.to_be_bytes());
        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pheromone_creation() {
        let value = ConsensusValue::from_string("test");
        let private_key = vec![0u8; 32]; // Dummy key for testing
        
        let pheromone = Pheromone::new(value.clone(), 1, &private_key);
        assert!(pheromone.is_ok());
        
        let p = pheromone.unwrap();
        assert_eq!(p.source, 1);
        assert_eq!(p.value, value);
        assert_eq!(p.intensity, INITIAL_PHEROMONE_INTENSITY);
    }

    #[test]
    fn test_pheromone_evaporation() {
        let value = ConsensusValue::from_string("test");
        let private_key = vec![0u8; 32];
        
        let mut pheromone = Pheromone::new(value, 1, &private_key).unwrap();
        let initial_intensity = pheromone.intensity;
        
        pheromone.evaporate(0.1);
        assert!(pheromone.intensity < initial_intensity);
    }
}

