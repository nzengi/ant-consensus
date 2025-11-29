use crate::core::types::{NodeId, ConsensusValue};
use crate::core::pheromone::Pheromone;
use serde::{Serialize, Deserialize};

/// Message types in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Pheromone broadcast
    PheromoneBroadcast {
        pheromone: Pheromone,
        sender: NodeId,
    },
    
    /// Ant agent movement
    AntMovement {
        ant_id: u64,
        from_node: NodeId,
        to_node: NodeId,
        carried_pheromone: Option<Pheromone>,
    },
    
    /// Neighbor discovery
    NeighborDiscovery {
        node_id: NodeId,
        neighbors: Vec<NodeId>,
    },
    
    /// Consensus announcement
    ConsensusAnnouncement {
        node_id: NodeId,
        value: ConsensusValue,
    },
    
    /// Heartbeat message
    Heartbeat {
        node_id: NodeId,
        timestamp: u64,
    },
}

impl Message {
    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self)
            .map_err(|e| format!("Serialization error: {}", e))
    }

    /// Deserialize message from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data)
            .map_err(|e| format!("Deserialization error: {}", e))
    }

    /// Get the sender node ID
    pub fn sender(&self) -> Option<NodeId> {
        match self {
            Message::PheromoneBroadcast { sender, .. } => Some(*sender),
            Message::AntMovement { from_node, .. } => Some(*from_node),
            Message::NeighborDiscovery { node_id, .. } => Some(*node_id),
            Message::ConsensusAnnouncement { node_id, .. } => Some(*node_id),
            Message::Heartbeat { node_id, .. } => Some(*node_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::ConsensusValue;

    #[test]
    fn test_message_serialization() {
        let value = ConsensusValue::from_string("test");
        let private_key = vec![0u8; 32];
        let pheromone = Pheromone::new(value.clone(), 1, &private_key).unwrap();
        
        let message = Message::PheromoneBroadcast {
            pheromone: pheromone.clone(),
            sender: 1,
        };

        let bytes = message.to_bytes().unwrap();
        let deserialized = Message::from_bytes(&bytes).unwrap();

        match deserialized {
            Message::PheromoneBroadcast { sender, .. } => {
                assert_eq!(sender, 1);
            }
            _ => panic!("Wrong message type"),
        }
    }
}

