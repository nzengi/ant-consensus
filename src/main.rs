use antcolony_consensus::*;
use clap::Parser;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

#[derive(Parser, Debug)]
#[command(name = "antconsensus")]
#[command(about = "AntColony Consensus - Blockchain-less distributed consensus system")]
struct Args {
    /// Node ID (unique identifier)
    #[arg(short, long, default_value = "1")]
    node_id: u32,

    /// Multicast address
    #[arg(short, long, default_value = "239.255.0.1:5000")]
    multicast_addr: String,

    /// Port for receiving messages
    #[arg(short, long, default_value = "5000")]
    port: u16,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("antcolony_consensus={}", log_level))
        .init();

    info!("🐜 AntColony Consensus Node {} starting...", args.node_id);

    // Create node state
    let node_state = Arc::new(RwLock::new(
        NodeState::new(args.node_id)
    ));

    // Initialize network layer
    let network = NetworkManager::new(
        args.multicast_addr.parse()?,
        args.port,
        node_state.clone(),
    ).await?;

    // Start consensus engine
    let consensus_engine = ConsensusEngine::new(
        node_state.clone(),
        network.clone(),
    );

    info!("Node {} initialized successfully", args.node_id);
    info!("Listening on multicast: {}", args.multicast_addr);
    info!("Press Ctrl+C to stop");

    // Start all services
    let network_handle = tokio::spawn(async move {
        if let Err(e) = network.start().await {
            error!("Network error: {}", e);
        }
    });

    let consensus_handle = tokio::spawn(async move {
        if let Err(e) = consensus_engine.run().await {
            error!("Consensus error: {}", e);
        }
    });

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    network_handle.abort();
    consensus_handle.abort();

    Ok(())
}

