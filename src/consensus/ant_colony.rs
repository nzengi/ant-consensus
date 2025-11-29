use crate::core::node_state::{NodeState, SharedNodeState};
use crate::core::types::{ConsensusValue, NodeId, AntId};
use crate::core::pheromone::Pheromone;
use crate::core::ant_agent::AntAgent;
use crate::network::NetworkManager;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{interval, Duration};
use tracing::{info, debug, warn};

/// Ant colony consensus algorithm implementation
pub struct AntColonyConsensus {
    node_state: SharedNodeState,
    network: NetworkManager,
    next_ant_id: AtomicU64,
}

impl AntColonyConsensus {
    /// Create a new ant colony consensus instance
    pub fn new(node_state: SharedNodeState, network: NetworkManager) -> Self {
        Self {
            node_state,
            network,
            next_ant_id: AtomicU64::new(1),
        }
    }

    /// Propose a consensus value
    pub async fn propose_value(
        &self,
        value: ConsensusValue,
        private_key: &[u8],
    ) -> Result<(), String> {
        let mut state = self.node_state.write().await;
        
        // Emit pheromone with the proposed value
        let pheromone = state.emit_pheromone(value.clone(), private_key)
            .map_err(|e| format!("Failed to emit pheromone: {}", e))?;

        drop(state);

        // Broadcast pheromone to network
        self.network.send_pheromone(pheromone).await?;

        // Create ant agents to explore the network
        self.create_explorer_ants(value).await?;

        info!("Proposed consensus value: {}", value);
        Ok(())
    }

    /// Create explorer ants to spread the pheromone
    async fn create_explorer_ants(&self, value: ConsensusValue) -> Result<(), String> {
        let (node_id, neighbors, private_key) = {
            let state = self.node_state.read().await;
            let neighbors = state.get_neighbors();
            (state.id, neighbors, vec![0u8; 32]) // TODO: Get actual private key
        };

        if neighbors.is_empty() {
            return Ok(());
        }

        // Create multiple ants (one per neighbor initially)
        let num_ants = neighbors.len().min(5); // Limit to 5 ants

        for _ in 0..num_ants {
            let ant_id = self.next_ant_id.fetch_add(1, Ordering::Relaxed);
            
            // Create pheromone for ant to carry
            let mut state = self.node_state.write().await;
            let pheromone = state.emit_pheromone(value.clone(), &private_key)
                .map_err(|e| format!("Failed to create pheromone: {}", e))?;
            drop(state);

            // Create ant with pheromone
            let ant = AntAgent::with_pheromone(ant_id, node_id, pheromone);
            
            // Add ant to node
            let mut state = self.node_state.write().await;
            state.add_ant(ant);
        }

        Ok(())
    }

    /// Run the consensus algorithm step
    pub async fn step(&self) -> Result<Option<ConsensusValue>, String> {
        let mut state = self.node_state.write().await;

        // Evaporate pheromones
        state.evaporate_pheromones();

        // Update ants
        state.update_ants();

        // Check for consensus
        let consensus = state.check_consensus();
        
        drop(state);

        // If consensus reached, announce it
        if let Some(value) = &consensus {
            self.announce_consensus(value.clone()).await?;
        }

        // Move ants
        self.move_ants().await?;

        Ok(consensus)
    }

    /// Move ants to neighboring nodes
    async fn move_ants(&self) -> Result<(), String> {
        let (ants_to_move, neighbors, node_id) = {
            let state = self.node_state.read().await;
            let ants: Vec<_> = state.ants.iter()
                .filter(|ant| ant.is_alive())
                .map(|ant| (ant.id, ant.current_node, ant.carried_pheromone.clone()))
                .collect();
            (ants, state.get_neighbors(), state.id)
        };

        for (ant_id, current_node, carried_pheromone) in ants_to_move {
            if current_node != node_id {
                continue; // Ant is not at this node
            }

            // Get pheromone intensities for neighbors
            let pheromone_intensities = self.get_pheromone_intensities().await;

            // Select next node
            let mut state = self.node_state.write().await;
            if let Some(ant) = state.ants.iter_mut().find(|a| a.id == ant_id) {
                if let Some(next_node) = ant.select_next_node(&neighbors, &pheromone_intensities) {
                    // Move ant
                    ant.move_to(next_node);

                    // Send ant movement message
                    let message = crate::network::message::Message::AntMovement {
                        ant_id,
                        from_node: node_id,
                        to_node: next_node,
                        carried_pheromone: ant.carried_pheromone.clone(),
                    };

                    drop(state);

                    if let Err(e) = self.network.broadcast(message).await {
                        warn!("Failed to broadcast ant movement: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get pheromone intensities for all neighbors
    async fn get_pheromone_intensities(&self) -> Vec<(NodeId, f64)> {
        let state = self.node_state.read().await;
        let mut intensities = Vec::new();

        for neighbor in &state.neighbors {
            // Calculate average intensity for this neighbor's pheromones
            // This is simplified - in reality, we'd query the neighbor
            let avg_intensity = 0.5; // Placeholder
            intensities.push((*neighbor, avg_intensity));
        }

        intensities
    }

    /// Announce consensus to the network
    async fn announce_consensus(&self, value: ConsensusValue) -> Result<(), String> {
        let node_id = {
            let state = self.node_state.read().await;
            state.id
        };

        let message = crate::network::message::Message::ConsensusAnnouncement {
            node_id,
            value,
        };

        self.network.broadcast(message).await?;
        Ok(())
    }
}

