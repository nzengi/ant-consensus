use crate::core::node_state::SharedNodeState;
use crate::network::message::Message;
use crate::network::multicast::NetworkManager;
use tokio::time::{interval, Duration};
use tracing::info;

/// Neighbor discovery service
pub struct NeighborDiscovery {
    node_state: SharedNodeState,
    network: NetworkManager,
}

impl NeighborDiscovery {
    /// Create a new neighbor discovery service
    pub fn new(node_state: SharedNodeState, network: NetworkManager) -> Self {
        Self {
            node_state,
            network,
        }
    }

    /// Start neighbor discovery
    pub async fn start(&self) {
        let node_state = self.node_state.clone();
        let network = self.network.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                let (node_id, neighbors) = {
                    let state = node_state.read().await;
                    (state.id, state.get_neighbors())
                };

                let message = Message::NeighborDiscovery {
                    node_id,
                    neighbors,
                };

                if let Err(e) = network.broadcast(message).await {
                    info!("Failed to broadcast neighbor discovery: {}", e);
                }
            }
        });
    }
}

