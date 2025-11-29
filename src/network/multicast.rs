use crate::core::node_state::{NodeState, SharedNodeState};
use crate::network::message::Message;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{info, error, debug};

/// Network manager for UDP multicast communication
pub struct NetworkManager {
    multicast_addr: SocketAddr,
    local_port: u16,
    node_state: SharedNodeState,
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new(
        multicast_addr: SocketAddr,
        local_port: u16,
        node_state: SharedNodeState,
    ) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel(1000);

        Ok(Self {
            multicast_addr,
            local_port,
            node_state,
            sender: tx,
            receiver: rx,
        })
    }

    /// Start the network manager
    pub async fn start(&self) -> Result<(), String> {
        let multicast_addr = self.multicast_addr;
        let local_port = self.local_port;
        let node_state = self.node_state.clone();
        let mut receiver = self.receiver.clone();
        let sender = self.sender.clone();

        // Spawn receiver task
        tokio::spawn(async move {
            let socket = match UdpSocket::bind(format!("0.0.0.0:{}", local_port)).await {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to bind socket: {}", e);
                    return;
                }
            };

            // Convert to std socket for multicast operations
            let std_socket = match socket.into_std() {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to convert to std socket: {}", e);
                    return;
                }
            };
            
            // Join multicast group
            if let Err(e) = std_socket.join_multicast_v4(
                multicast_addr.ip(),
                "0.0.0.0".parse().unwrap(),
            ) {
                error!("Failed to join multicast group: {}", e);
                return;
            }

            // Convert back to tokio socket
            let socket = match UdpSocket::from_std(std_socket) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to convert back to tokio socket: {}", e);
                    return;
                }
            };

            info!("Network receiver started on port {}", local_port);

            let mut buf = [0u8; 65507]; // Max UDP packet size

            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((size, addr)) => {
                        debug!("Received {} bytes from {}", size, addr);
                        
                        match Message::from_bytes(&buf[..size]) {
                            Ok(message) => {
                                // Process message
                                if let Err(e) = Self::handle_message(&message, &node_state).await {
                                    error!("Error handling message: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize message: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Receive error: {}", e);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });

        // Spawn sender task
        tokio::spawn(async move {
            let socket = match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to bind sender socket: {}", e);
                    return;
                }
            };

            info!("Network sender started");

            while let Some(message) = receiver.recv().await {
                match message.to_bytes() {
                    Ok(bytes) => {
                        if let Err(e) = socket.send_to(&bytes, multicast_addr).await {
                            error!("Failed to send message: {}", e);
                        } else {
                            debug!("Sent message to {}", multicast_addr);
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                    }
                }
            }
        });

        // Send periodic heartbeat
        let node_state_clone = self.node_state.clone();
        let sender_clone = self.sender.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                let node_id = {
                    let state = node_state_clone.read().await;
                    state.id
                };

                let heartbeat = Message::Heartbeat {
                    node_id,
                    timestamp: crate::utils::current_timestamp(),
                };

                if sender_clone.send(heartbeat).await.is_err() {
                    break;
                }
            }
        });

        Ok(())
    }

    /// Handle incoming message
    async fn handle_message(
        message: &Message,
        node_state: &SharedNodeState,
    ) -> Result<(), String> {
        match message {
            Message::PheromoneBroadcast { pheromone, sender } => {
                let mut state = node_state.write().await;
                
                // Don't process our own messages
                if sender == &state.id {
                    return Ok(());
                }

                // Add sender as neighbor
                state.add_neighbor(*sender);
                
                // Receive pheromone
                state.receive_pheromone(pheromone.clone());
                
                debug!("Received pheromone from node {}", sender);
            }
            
            Message::AntMovement { ant_id, to_node, carried_pheromone, .. } => {
                let mut state = node_state.write().await;
                
                // If ant arrived at this node
                if to_node == &state.id {
                    if let Some(pheromone) = carried_pheromone {
                        state.receive_pheromone(pheromone.clone());
                        debug!("Ant {} arrived with pheromone", ant_id);
                    }
                }
            }
            
            Message::NeighborDiscovery { node_id, neighbors } => {
                let mut state = node_state.write().await;
                
                if node_id != &state.id {
                    state.add_neighbor(*node_id);
                    for neighbor in neighbors {
                        state.add_neighbor(*neighbor);
                    }
                }
            }
            
            Message::ConsensusAnnouncement { node_id, value } => {
                let mut state = node_state.write().await;
                
                if node_id != &state.id {
                    info!("Node {} announced consensus: {}", node_id, value);
                    // Could trigger consensus verification
                }
            }
            
            Message::Heartbeat { node_id, .. } => {
                let mut state = node_state.write().await;
                
                if node_id != &state.id {
                    state.add_neighbor(*node_id);
                }
            }
        }

        Ok(())
    }

    /// Broadcast a message
    pub async fn broadcast(&self, message: Message) -> Result<(), String> {
        self.sender.send(message).await
            .map_err(|e| format!("Failed to send message: {}", e))
    }

    /// Send a pheromone
    pub async fn send_pheromone(&self, pheromone: crate::core::pheromone::Pheromone) -> Result<(), String> {
        let node_id = {
            let state = self.node_state.read().await;
            state.id
        };

        let message = Message::PheromoneBroadcast {
            pheromone,
            sender: node_id,
        };

        self.broadcast(message).await
    }
}

impl Clone for NetworkManager {
    fn clone(&self) -> Self {
        Self {
            multicast_addr: self.multicast_addr,
            local_port: self.local_port,
            node_state: self.node_state.clone(),
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

