use crate::core::types::{NodeId, AntId, ConsensusValue};
use crate::core::pheromone::Pheromone;
use rand::Rng;
use std::collections::HashSet;

/// Initial energy level for ants
pub const INITIAL_ANT_ENERGY: f64 = 100.0;

/// Energy decay rate per step
pub const ENERGY_DECAY_RATE: f64 = 0.1;

/// Minimum energy to stay alive
pub const MIN_ANT_ENERGY: f64 = 0.0;

/// Maximum number of nodes an ant can remember
pub const ANT_MEMORY_SIZE: usize = 256;

/// Ant agent - represents a mobile agent in the network
#[derive(Debug, Clone)]
pub struct AntAgent {
    /// Unique identifier for this ant
    pub id: AntId,
    
    /// Current node the ant is on
    pub current_node: NodeId,
    
    /// Pheromone the ant is carrying
    pub carried_pheromone: Option<Pheromone>,
    
    /// Memory of visited nodes (to avoid loops)
    pub visited_nodes: HashSet<NodeId>,
    
    /// Current energy level
    pub energy_level: f64,
    
    /// Starting node
    pub start_node: NodeId,
}

impl AntAgent {
    /// Create a new ant agent
    pub fn new(id: AntId, start_node: NodeId) -> Self {
        let mut visited = HashSet::new();
        visited.insert(start_node);

        Self {
            id,
            current_node: start_node,
            carried_pheromone: None,
            visited_nodes: visited,
            energy_level: INITIAL_ANT_ENERGY,
            start_node,
        }
    }

    /// Create an ant with a pheromone to carry
    pub fn with_pheromone(id: AntId, start_node: NodeId, pheromone: Pheromone) -> Self {
        let mut ant = Self::new(id, start_node);
        ant.carried_pheromone = Some(pheromone);
        ant
    }

    /// Update ant's energy (decreases over time)
    pub fn update_energy(&mut self) {
        self.energy_level -= ENERGY_DECAY_RATE;
    }

    /// Check if ant is still alive
    pub fn is_alive(&self) -> bool {
        self.energy_level > MIN_ANT_ENERGY
    }

    /// Select next node based on pheromone intensities
    /// Uses probabilistic selection (roulette wheel)
    pub fn select_next_node(
        &self,
        neighbors: &[NodeId],
        pheromone_intensities: &[(NodeId, f64)],
    ) -> Option<NodeId> {
        if neighbors.is_empty() {
            return None;
        }

        // Filter out visited nodes
        let available_neighbors: Vec<NodeId> = neighbors
            .iter()
            .filter(|&&node| !self.visited_nodes.contains(&node))
            .copied()
            .collect();

        if available_neighbors.is_empty() {
            // All neighbors visited, reset memory or return random
            return neighbors.first().copied();
        }

        // Calculate probabilities based on pheromone intensities
        let mut probabilities: Vec<(NodeId, f64)> = Vec::new();
        let mut total_intensity = 0.0;

        for &neighbor in &available_neighbors {
            let intensity = pheromone_intensities
                .iter()
                .find(|(id, _)| *id == neighbor)
                .map(|(_, intensity)| *intensity)
                .unwrap_or(0.1); // Default low intensity for unexplored paths

            probabilities.push((neighbor, intensity));
            total_intensity += intensity;
        }

        if total_intensity == 0.0 {
            // No pheromone trail, random selection
            let mut rng = rand::thread_rng();
            return available_neighbors.get(rng.gen_range(0..available_neighbors.len())).copied();
        }

        // Roulette wheel selection
        let mut rng = rand::thread_rng();
        let random_value = rng.gen::<f64>() * total_intensity;
        let mut cumulative = 0.0;

        for (node, intensity) in probabilities {
            cumulative += intensity;
            if random_value <= cumulative {
                return Some(node);
            }
        }

        // Fallback to first available
        available_neighbors.first().copied()
    }

    /// Move ant to a new node
    pub fn move_to(&mut self, node: NodeId) {
        self.visited_nodes.insert(node);
        self.current_node = node;

        // Limit memory size
        if self.visited_nodes.len() > ANT_MEMORY_SIZE {
            // Remove oldest entries (simple: remove start_node if present)
            self.visited_nodes.remove(&self.start_node);
        }
    }

    /// Drop pheromone at current location
    pub fn drop_pheromone(&mut self) -> Option<Pheromone> {
        self.carried_pheromone.take()
    }

    /// Pick up a pheromone
    pub fn pick_up_pheromone(&mut self, pheromone: Pheromone) {
        self.carried_pheromone = Some(pheromone);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pheromone::Pheromone;
    use crate::core::types::ConsensusValue;

    #[test]
    fn test_ant_creation() {
        let ant = AntAgent::new(1, 10);
        assert_eq!(ant.id, 1);
        assert_eq!(ant.current_node, 10);
        assert_eq!(ant.start_node, 10);
        assert_eq!(ant.energy_level, INITIAL_ANT_ENERGY);
        assert!(ant.is_alive());
    }

    #[test]
    fn test_ant_energy_decay() {
        let mut ant = AntAgent::new(1, 10);
        let initial_energy = ant.energy_level;
        
        ant.update_energy();
        assert!(ant.energy_level < initial_energy);
    }

    #[test]
    fn test_ant_node_selection() {
        let ant = AntAgent::new(1, 10);
        let neighbors = vec![11, 12, 13];
        let intensities = vec![(11, 0.5), (12, 0.3), (13, 0.2)];
        
        let next = ant.select_next_node(&neighbors, &intensities);
        assert!(next.is_some());
        assert!(neighbors.contains(&next.unwrap()));
    }
}

