use antcolony_consensus::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

/// Simple simulation example
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐜 AntColony Consensus - Simple Simulation");
    
    // Create a node
    let node_state = Arc::new(RwLock::new(NodeState::new(1)));
    
    // Create network manager (simplified for example)
    let network = NetworkManager::new(
        "239.255.0.1:5000".parse()?,
        5000,
        node_state.clone(),
    ).await?;
    
    // Create consensus engine
    let consensus_engine = ConsensusEngine::new(
        node_state.clone(),
        network.clone(),
    );
    
    // Start network
    tokio::spawn(async move {
        if let Err(e) = network.start().await {
            eprintln!("Network error: {}", e);
        }
    });
    
    // Start consensus engine
    tokio::spawn(async move {
        if let Err(e) = consensus_engine.run().await {
            eprintln!("Consensus error: {}", e);
        }
    });
    
    // Wait a bit
    sleep(Duration::from_secs(5)).await;
    
    // Propose a value
    let value = ConsensusValue::from_string("Hello, Consensus!");
    let private_key = vec![0u8; 32]; // Dummy key
    
    {
        let mut state = node_state.write().await;
        if let Ok(pheromone) = state.emit_pheromone(value.clone(), &private_key) {
            println!("Emitted pheromone for value: {}", value);
            if let Err(e) = network.send_pheromone(pheromone).await {
                eprintln!("Failed to send pheromone: {}", e);
            }
        }
    }
    
    // Wait for consensus
    sleep(Duration::from_secs(10)).await;
    
    // Check consensus
    if let Some(consensus) = consensus_engine.get_consensus().await {
        println!("✅ Consensus reached: {}", consensus);
    } else {
        println!("⏳ Consensus not yet reached");
    }
    
    Ok(())
}

