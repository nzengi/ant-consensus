use crate::core::types::{NodeId, ConsensusValue, Result, ConsensusError};
use crate::core::pheromone::{Pheromone, CONSENSUS_THRESHOLD};
use crate::core::ant_agent::AntAgent;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Default evaporation rate for pheromones
pub const DEFAULT_EVAPORATION_RATE: f64 = 0.01;

/// Maximum number of neighbors
pub const MAX_NEIGHBORS: usize = 32;

/// Node state - manages the state of a single node in the network
#[derive(Debug)]
pub struct NodeState {
    /// Unique identifier for this node
    pub id: NodeId,

    /// Current consensus value (if consensus reached)
    pub current_value: Option<ConsensusValue>,

    /// Pheromones stored at this node (grouped by value)
    pub pheromones: HashMap<ConsensusValue, Vec<Pheromone>>,

    /// Active ant agents at this node
    pub ants: Vec<AntAgent>,

    /// Known neighbor nodes
    pub neighbors: HashSet<NodeId>,

    /// Pheromone evaporation rate
    pub evaporation_rate: f64,

    /// Statistics
    pub stats: NodeStats,
}

/// Node statistics
#[derive(Debug, Default, Clone)]
pub struct NodeStats {
    pub pheromones_received: u64,
    pub pheromones_emitted: u64,
    pub ants_created: u64,
    pub consensus_reached: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
}

impl NodeState {
    /// Create a new node state
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            current_value: None,
            pheromones: HashMap::new(),
            ants: Vec::new(),
            neighbors: HashSet::new(),
            evaporation_rate: DEFAULT_EVAPORATION_RATE,
            stats: NodeStats::default(),
        }
    }

    /// Add a neighbor node
    pub fn add_neighbor(&mut self, neighbor: NodeId) {
        if neighbor != self.id {
            self.neighbors.insert(neighbor);
        }
    }

    /// Remove a neighbor node
    pub fn remove_neighbor(&mut self, neighbor: NodeId) {
        self.neighbors.remove(&neighbor);
    }

    /// Emit a pheromone with a consensus value
    pub fn emit_pheromone(
        &mut self,
        value: ConsensusValue,
        private_key: &[u8],
    ) -> Result<Pheromone> {
        let pheromone = Pheromone::new(value.clone(), self.id, private_key)?;
        
        self.pheromones
            .entry(value)
            .or_insert_with(Vec::new)
            .push(pheromone.clone());

        self.stats.pheromones_emitted += 1;
        Ok(pheromone)
    }

    /// Receive a pheromone from another node
    pub fn receive_pheromone(&mut self, pheromone: Pheromone) {
        let value = pheromone.value.clone();
        self.pheromones
            .entry(value)
            .or_insert_with(Vec::new)
            .push(pheromone);

        self.stats.pheromones_received += 1;
    }

    /// Evaporate all pheromones (reduce intensity over time)
    pub fn evaporate_pheromones(&mut self) {
        let mut to_remove = Vec::new();

        for (value, pheromones) in &mut self.pheromones {
            pheromones.retain_mut(|p| {
                p.evaporate(self.evaporation_rate);
                !p.should_remove()
            });

            if pheromones.is_empty() {
                to_remove.push(value.clone());
            }
        }

        for value in to_remove {
            self.pheromones.remove(&value);
        }
    }

    /// Check if consensus has been reached
    pub fn check_consensus(&mut self) -> Option<ConsensusValue> {
        // Find the value with the strongest pheromone trail
        let mut best_value: Option<(ConsensusValue, f64)> = None;

        for (value, pheromones) in &self.pheromones {
            // Calculate total intensity for this value
            let total_intensity: f64 = pheromones
                .iter()
                .map(|p| p.strength())
                .sum();

            // Average intensity
            let avg_intensity = total_intensity / pheromones.len() as f64;

            if let Some((_, best_intensity)) = best_value {
                if avg_intensity > best_intensity {
                    best_value = Some((value.clone(), avg_intensity));
                }
            } else {
                best_value = Some((value.clone(), avg_intensity));
            }
        }

        if let Some((value, intensity)) = best_value {
            if intensity >= CONSENSUS_THRESHOLD {
                self.current_value = Some(value.clone());
                self.stats.consensus_reached += 1;
                return Some(value);
            }
        }

        None
    }

    /// Get the strongest pheromone for a given value
    pub fn get_strongest_pheromone(&self, value: &ConsensusValue) -> Option<&Pheromone> {
        self.pheromones
            .get(value)?
            .iter()
            .max_by(|a, b| a.strength().partial_cmp(&b.strength()).unwrap())
    }

    /// Add an ant agent to this node
    pub fn add_ant(&mut self, ant: AntAgent) {
        self.ants.push(ant);
        self.stats.ants_created += 1;
    }

    /// Remove dead ants
    pub fn cleanup_dead_ants(&mut self) {
        self.ants.retain(|ant| ant.is_alive());
    }

    /// Update all ants (energy decay, movement)
    pub fn update_ants(&mut self) {
        for ant in &mut self.ants {
            ant.update_energy();
        }
        self.cleanup_dead_ants();
    }

    /// Get neighbor list as vector
    pub fn get_neighbors(&self) -> Vec<NodeId> {
        self.neighbors.iter().copied().collect()
    }

    /// Get statistics
    pub fn get_stats(&self) -> &NodeStats {
        &self.stats
    }
}

/// Type alias for shared node state
pub type SharedNodeState = Arc<RwLock<NodeState>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = NodeState::new(1);
        assert_eq!(node.id, 1);
        assert!(node.current_value.is_none());
        assert!(node.neighbors.is_empty());
    }

    #[test]
    fn test_add_neighbor() {
        let mut node = NodeState::new(1);
        node.add_neighbor(2);
        assert!(node.neighbors.contains(&2));
    }

    #[test]
    fn test_consensus_check() {
        let mut node = NodeState::new(1);
        // Consensus check with no pheromones should return None
        assert!(node.check_consensus().is_none());
    }
}

