// File: crates/icn_network/src/lib.rs

use icn_common::{IcnResult, IcnError, Transaction, NetworkStats};
use icn_blockchain::Block;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{info, warn, error};
use serde::{Serialize, Deserialize};
use rand::seq::SliceRandom;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Transaction(Transaction),
    Block(Block),
    PeerConnect(SocketAddr),
    PeerDisconnect(SocketAddr),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    Transaction(Transaction),
    Block(Block),
    NetworkUpdate(NetworkUpdate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkUpdate {
    pub timestamp: u64,
    pub connected_peers: Vec<SocketAddr>,
}

struct PeerInfo {
    last_seen: Instant,
}

pub struct NetworkManager {
    local_addr: SocketAddr,
    peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
    event_sender: mpsc::Sender<NetworkMessage>,
    event_receiver: mpsc::Receiver<NetworkMessage>,
    start_time: Option<Instant>,
    gossip_interval: Duration,
    gossip_peer_count: usize,
}

impl NetworkManager {
    pub fn new(local_addr: SocketAddr) -> Self {
        let (event_sender, event_receiver) = mpsc::channel(100); // Adjust buffer size as needed
        NetworkManager {
            local_addr,
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver,
            start_time: None,
            gossip_interval: Duration::from_secs(30), // Gossip every 30 seconds
            gossip_peer_count: 3, // Number of peers to gossip with each round
        }
    }

    pub async fn start(&mut self) -> IcnResult<()> {
        info!("Starting network on {}", self.local_addr);
        self.start_time = Some(Instant::now());

        let listener = TcpListener::bind(self.local_addr).await
            .map_err(|e| IcnError::Network(format!("Failed to bind to address: {}", e)))?;

        let peers = Arc::clone(&self.peers);
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                let peer_tx = event_sender.clone();
                let peer_peers = Arc::clone(&peers);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, addr, peer_tx, peer_peers).await {
                        error!("Error handling connection from {}: {}", addr, e);
                    }
                });
            }
        });

        // Start the gossip protocol
        let gossip_peers = Arc::clone(&self.peers);
        let gossip_sender = self.event_sender.clone();
        let gossip_interval = self.gossip_interval;
        let gossip_peer_count = self.gossip_peer_count;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(gossip_interval).await;
                if let Err(e) = Self::gossip_protocol(&gossip_peers, &gossip_sender, gossip_peer_count).await {
                    error!("Error in gossip protocol: {}", e);
                }
            }
        });

        info!("Network started successfully");
        Ok(())
    }

    async fn gossip_protocol(
        peers: &Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
        event_sender: &mpsc::Sender<NetworkMessage>,
        gossip_peer_count: usize,
    ) -> IcnResult<()> {
        let gossip_peers = {
            let peers_lock = peers.read().unwrap();
            peers_lock.keys().cloned().collect::<Vec<SocketAddr>>()
        };

        if gossip_peers.is_empty() {
            return Ok(());
        }

        let selected_peers = Self::select_random_peers(&gossip_peers, gossip_peer_count);

        let gossip_message = GossipMessage::NetworkUpdate(NetworkUpdate {
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            connected_peers: gossip_peers,
        });

        for peer in selected_peers {
            if let Err(e) = Self::send_gossip_message(&peer, &gossip_message).await {
                warn!("Failed to send gossip message to {}: {}", peer, e);
            }
        }

        Ok(())
    }

    fn select_random_peers(peers: &[SocketAddr], count: usize) -> Vec<SocketAddr> {
        let mut rng = rand::thread_rng();
        peers.choose_multiple(&mut rng, count.min(peers.len())).cloned().collect()
    }

    async fn send_gossip_message(peer: &SocketAddr, message: &GossipMessage) -> IcnResult<()> {
        let mut stream = TcpStream::connect(peer).await
            .map_err(|e| IcnError::Network(format!("Failed to connect to peer {}: {}", peer, e)))?;

        let serialized_message = bincode::serialize(&message)
            .map_err(|e| IcnError::Network(format!("Failed to serialize gossip message: {}", e)))?;

        stream.write_all(&serialized_message).await
            .map_err(|e| IcnError::Network(format!("Failed to send gossip message to peer {}: {}", peer, e)))?;

        Ok(())
    }

    pub async fn stop(&mut self) -> IcnResult<()> {
        info!("Stopping network");
        self.start_time = None;
        Ok(())
    }

    pub fn get_connected_peers(&self) -> Vec<SocketAddr> {
        self.peers.read().unwrap().keys().cloned().collect()
    }

    pub fn get_uptime(&self) -> Duration {
        self.start_time.map_or(Duration::from_secs(0), |start| start.elapsed())
    }

    pub async fn connect_to_peer(&mut self, peer_addr: SocketAddr) -> IcnResult<()> {
        if self.peers.read().unwrap().contains_key(&peer_addr) {
            return Ok(());  // Already connected
        }

        let stream = TcpStream::connect(peer_addr).await
            .map_err(|e| IcnError::Network(format!("Failed to connect to peer {}: {}", peer_addr, e)))?;

        let peers = Arc::clone(&self.peers);
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer_addr, event_sender, peers).await {
                error!("Error handling connection to {}: {}", peer_addr, e);
            }
        });

        self.peers.write().unwrap().insert(peer_addr, PeerInfo { last_seen: Instant::now() });
        self.event_sender.send(NetworkMessage::PeerConnect(peer_addr)).await
            .map_err(|e| IcnError::Network(format!("Failed to send peer connected event: {}", e)))?;

        info!("Connected to peer: {}", peer_addr);
        Ok(())
    }

    pub async fn disconnect_from_peer(&mut self, peer_addr: &SocketAddr) -> IcnResult<()> {
        self.peers.write().unwrap().remove(peer_addr);
        self.event_sender.send(NetworkMessage::PeerDisconnect(*peer_addr)).await
            .map_err(|e| IcnError::Network(format!("Failed to send peer disconnected event: {}", e)))?;
        Ok(())
    }

    pub async fn broadcast_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        let message = NetworkMessage::Transaction(transaction);
        self.broadcast_message(message).await
    }

    pub async fn broadcast_block(&self, block: Block) -> IcnResult<()> {
        let message = NetworkMessage::Block(block);
        self.broadcast_message(message).await
    }

    async fn broadcast_message(&self, message: NetworkMessage) -> IcnResult<()> {
        let peers = self.peers.read().unwrap();
        for peer_addr in peers.keys() {
            if let Err(e) = self.send_message_to_peer(*peer_addr, message.clone()).await {
                warn!("Failed to send message to peer {}: {}", peer_addr, e);
            }
        }
        Ok(())
    }

    async fn send_message_to_peer(&self, peer_addr: SocketAddr, message: NetworkMessage) -> IcnResult<()> {
        let mut stream = TcpStream::connect(peer_addr).await
            .map_err(|e| IcnError::Network(format!("Failed to connect to peer {}: {}", peer_addr, e)))?;

        let serialized_message = bincode::serialize(&message)
            .map_err(|e| IcnError::Network(format!("Failed to serialize message: {}", e)))?;

        stream.write_all(&serialized_message).await
            .map_err(|e| IcnError::Network(format!("Failed to send message to peer {}: {}", peer_addr, e)))?;

        Ok(())
    }

    pub async fn receive_event(&mut self) -> Option<NetworkMessage> {
        self.event_receiver.recv().await
    }

    pub async fn get_network_stats(&self) -> NetworkStats {
        NetworkStats {
            node_count: self.peers.read().unwrap().len(),
            total_transactions: 0, // Implement tracking logic
            active_proposals: 0,   // Implement tracking logic
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    event_sender: mpsc::Sender<NetworkMessage>,
    peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
) -> IcnResult<()> {
    let (mut reader, mut writer) = stream.split();
    let mut buffer = vec![0; 1024]; // Use a fixed-size buffer

    loop {
        let bytes_read = reader.read(&mut buffer).await
            .map_err(|e| IcnError::Network(format!("Failed to read from stream: {}", e)))?;

        if bytes_read == 0 {
            // Connection closed
            peers.write().unwrap().remove(&addr);
            event_sender.send(NetworkMessage::PeerDisconnect(addr)).await
                .map_err(|e| IcnError::Network(format!("Failed to send peer disconnected event: {}", e)))?;
            break;
        }

        let message: NetworkMessage = bincode::deserialize(&buffer[..bytes_read])
            .map_err(|e| IcnError::Network(format!("Failed to deserialize message: {}", e)))?;

        match &message {
            NetworkMessage::Transaction(_) | NetworkMessage::Block(_) => {
                event_sender.send(message).await
                    .map_err(|e| IcnError::Network(format!("Failed to send message to main thread: {}", e)))?;
            }
            NetworkMessage::PeerConnect(new_peer) | NetworkMessage::PeerDisconnect(new_peer) => {
                if new_peer != &addr {
                    event_sender.send(message).await
                        .map_err(|e| IcnError::Network(format!("Failed to send peer event to main thread: {}", e)))?;
                }
            }
        }

        // Update last_seen timestamp for the peer
        if let Some(peer_info) = peers.write().unwrap().get_mut(&addr) {
            peer_info.last_seen = Instant::now();
        }

        buffer.clear();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_network_operations() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut manager1 = NetworkManager::new("127.0.0.1:8000".parse().unwrap());
            let mut manager2 = NetworkManager::new("127.0.0.1:8001".parse().unwrap());

            manager1.start().await.unwrap();
            manager2.start().await.unwrap();

            manager1.connect_to_peer("127.0.0.1:8001".parse().unwrap()).await.unwrap();

            // Wait a bit for the connection to be established
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            assert_eq!(manager1.get_connected_peers().len(), 1);
            assert_eq!(manager2.get_connected_peers().len(), 1);

            let transaction = Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 100.0,
                currency_type: icn_common::CurrencyType::BasicNeeds,
                timestamp: chrono::Utc::now().timestamp(),
                signature: None,
            };

            manager1.broadcast_transaction(transaction.clone()).await.unwrap();

            // Wait a bit for the message to be processed
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            if let Some(NetworkMessage::Transaction(received_tx)) = manager2.receive_event().await {
                assert_eq!(received_tx, transaction);
            } else {
                panic!("Did not receive expected transaction");
            }

            assert!(manager1.stop().await.is_ok());
            assert!(manager2.stop().await.is_ok());
        });
    }

    #[test]
    fn test_gossip_protocol() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut manager1 = NetworkManager::new("127.0.0.1:8002".parse().unwrap());
            let mut manager2 = NetworkManager::new("127.0.0.1:8003".parse().unwrap());
            let mut manager3 = NetworkManager::new("127.0.0.1:8004".parse().unwrap());

            manager1.start().await.unwrap();
            manager2.start().await.unwrap();
            manager3.start().await.unwrap();

            manager1.connect_to_peer("127.0.0.1:8003".parse().unwrap()).await.unwrap();
            manager1.connect_to_peer("127.0.0.1:8004".parse().unwrap()).await.unwrap();

            // Wait for gossip protocol to run
            tokio::time::sleep(std::time::Duration::from_secs(35)).await;

            assert_eq!(manager1.get_connected_peers().len(), 2);
            assert_eq!(manager2.get_connected_peers().len(), 1);
            assert_eq!(manager3.get_connected_peers().len(), 1);

            // Check if manager2 and manager3 received network updates
            let received_update2 = manager2.receive_event().await;
            let received_update3 = manager3.receive_event().await;

            assert!(matches!(received_update2, Some(NetworkMessage::PeerConnect(_))));
            assert!(matches!(received_update3, Some(NetworkMessage::PeerConnect(_))));

            manager1.stop().await.unwrap();
            manager2.stop().await.unwrap();
            manager3.stop().await.unwrap();
        });
    }

    #[test]
    fn test_peer_disconnect() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut manager1 = NetworkManager::new("127.0.0.1:8005".parse().unwrap());
            let mut manager2 = NetworkManager::new("127.0.0.1:8006".parse().unwrap());

            manager1.start().await.unwrap();
            manager2.start().await.unwrap();

            manager1.connect_to_peer("127.0.0.1:8006".parse().unwrap()).await.unwrap();

            // Wait a bit for the connection to be established
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            assert_eq!(manager1.get_connected_peers().len(), 1);
            assert_eq!(manager2.get_connected_peers().len(), 1);

            // Simulate manager2 disconnecting
            manager2.stop().await.unwrap();

            // Wait a bit for the disconnection to be detected
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            assert_eq!(manager1.get_connected_peers().len(), 0);

            // Check if manager1 received a peer disconnect message
            let received = manager1.receive_event().await;
            assert!(matches!(received, Some(NetworkMessage::PeerDisconnect(_))));

            manager1.stop().await.unwrap();
        });
    }
}