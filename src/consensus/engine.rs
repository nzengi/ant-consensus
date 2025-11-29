use crate::core::node_state::SharedNodeState;
use crate::core::types::ConsensusValue;
use crate::consensus::ant_colony::AntColonyConsensus;
use crate::network::NetworkManager;
use tokio::time::{interval, Duration};
use tracing::{info, error};

/// Consensus engine - main coordinator for consensus operations
pub struct ConsensusEngine {
    ant_colony: AntColonyConsensus,
    node_state: SharedNodeState,
}

impl ConsensusEngine {
    /// Create a new consensus engine
    pub fn new(node_state: SharedNodeState, network: NetworkManager) -> Self {
        let ant_colony = AntColonyConsensus::new(node_state.clone(), network);
        
        Self {
            ant_colony,
            node_state,
        }
    }

    /// Run the consensus engine
    pub async fn run(&self) -> Result<(), String> {
        info!("Consensus engine started");

        let mut interval = interval(Duration::from_millis(100)); // 10 steps per second

        loop {
            interval.tick().await;

            match self.ant_colony.step().await {
                Ok(Some(value)) => {
                    info!("🎉 Consensus reached: {}", value);
                    
                    // Update node state with consensus value
                    {
                        let mut state = self.node_state.write().await;
                        state.current_value = Some(value);
                    }
                }
                Ok(None) => {
                    // No consensus yet, continue
                }
                Err(e) => {
                    error!("Consensus step error: {}", e);
                }
            }
        }
    }

    /// Propose a value for consensus
    pub async fn propose(&self, value: ConsensusValue, private_key: &[u8]) -> Result<(), String> {
        self.ant_colony.propose_value(value, private_key).await
    }

    /// Get current consensus value (if any)
    pub async fn get_consensus(&self) -> Option<ConsensusValue> {
        let state = self.node_state.read().await;
        state.current_value.clone()
    }
}

