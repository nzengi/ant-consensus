use antcolony_consensus::core::*;
use antcolony_consensus::crypto::*;

#[test]
fn test_consensus_value_creation() {
    let value1 = ConsensusValue::from_string("test");
    let value2 = ConsensusValue::from_string("test");
    let value3 = ConsensusValue::from_string("different");

    // Same string should produce same hash
    assert_eq!(value1.hash, value2.hash);
    
    // Different strings should produce different hashes
    assert_ne!(value1.hash, value3.hash);
}

#[test]
fn test_node_state_operations() {
    let mut node = NodeState::new(1);
    
    // Add neighbors
    node.add_neighbor(2);
    node.add_neighbor(3);
    
    assert_eq!(node.get_neighbors().len(), 2);
    assert!(node.get_neighbors().contains(&2));
    assert!(node.get_neighbors().contains(&3));
}

#[test]
fn test_pheromone_evaporation() {
    let value = ConsensusValue::from_string("test");
    let private_key = vec![0u8; 32];
    
    let mut pheromone = Pheromone::new(value, 1, &private_key).unwrap();
    let initial_intensity = pheromone.intensity;
    
    pheromone.evaporate(0.1);
    assert!(pheromone.intensity < initial_intensity);
    
    // Evaporate multiple times
    for _ in 0..100 {
        pheromone.evaporate(0.01);
    }
    
    assert!(pheromone.intensity < 0.5);
}

#[test]
fn test_ant_agent_lifecycle() {
    let mut ant = AntAgent::new(1, 10);
    
    assert_eq!(ant.current_node, 10);
    assert!(ant.is_alive());
    
    // Decay energy
    let initial_energy = ant.energy_level;
    ant.update_energy();
    assert!(ant.energy_level < initial_energy);
    
    // Move to new node
    ant.move_to(11);
    assert_eq!(ant.current_node, 11);
    assert!(ant.visited_nodes.contains(&11));
}

#[test]
fn test_message_serialization() {
    use antcolony_consensus::network::Message;
    
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

