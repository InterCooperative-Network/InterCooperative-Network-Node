// File: icn_networking/src/lib.rs

use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_native_tls::{TlsAcceptor, TlsConnector};
use native_tls::{Identity, TlsConnector as NativeTlsConnector};
use thiserror::Error;
use tokio::sync::{RwLock, Mutex};
use log::{info, error, warn, debug};
use std::time::{Duration, Instant};

/// Custom error type for the networking module.
#[derive(Error, Debug)]
pub enum NetworkingError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("TLS error: {0}")]
    Tls(#[from] native_tls::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Lock error: {0}")]
    Lock(String),
    #[error("Timeout error: {0}")]
    Timeout(String),
}

/// Type alias for results returned by networking functions.
pub type NetworkingResult<T> = Result<T, NetworkingError>;

/// Represents a peer in the network.
#[derive(Debug, Clone)]
struct Peer {
    address: String,
    stream: Arc<Mutex<tokio_native_tls::TlsStream<TcpStream>>>,
}

/// The `Networking` struct is responsible for managing peer-to-peer network connections
/// in a secure manner using TLS (Transport Layer Security).
#[derive(Clone)]
pub struct Networking {
    peers: Arc<RwLock<Vec<Peer>>>,
    identity: Option<Arc<Identity>>,
    max_peers: usize,
    connection_timeout: Duration,
}

impl Networking {
    /// Creates a new instance of the `Networking` struct.
    pub fn new(max_peers: usize, connection_timeout: Duration) -> Self {
        Networking {
            peers: Arc::new(RwLock::new(vec![])),
            identity: None,
            max_peers,
            connection_timeout,
        }
    }

    /// Loads a TLS identity from a certificate and key file.
    pub fn load_tls_identity(cert_path: &str, key_path: &str) -> NetworkingResult<Arc<Identity>> {
        let identity = match (File::open(cert_path), File::open(key_path)) {
            (Ok(mut cert_file), Ok(mut key_file)) => {
                let mut cert = Vec::new();
                let mut key = Vec::new();
                cert_file.read_to_end(&mut cert)?;
                key_file.read_to_end(&mut key)?;
                Identity::from_pkcs8(&cert, &key)?
            }
            _ => return Err(NetworkingError::Network("Certificate or key file missing".into())),
        };
        Ok(Arc::new(identity))
    }

    /// Starts a TLS server listening on the specified address.
    pub async fn start_server(&mut self, address: &str, identity: Arc<Identity>) -> NetworkingResult<()> {
        self.identity = Some(identity.clone());
        let acceptor = TlsAcceptor::from(native_tls::TlsAcceptor::new(identity.as_ref().clone())?);
        let listener = TcpListener::bind(address).await?;

        info!("Server started on {}", address);

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    let acceptor = acceptor.clone();
                    let networking = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = networking.handle_client_connection(stream, peer_addr, acceptor).await {
                            error!("Error handling client: {:?}", e);
                        }
                    });
                }
                Err(e) => error!("Failed to accept TCP connection: {:?}", e),
            }
        }
    }

    /// Connects to a peer at the specified address using TLS.
    pub async fn connect_to_peer(&self, address: &str) -> NetworkingResult<()> {
        let connector = TlsConnector::from(NativeTlsConnector::new()?);
        let stream = tokio::time::timeout(
            self.connection_timeout,
            TcpStream::connect(address)
        ).await.map_err(|_| NetworkingError::Timeout(format!("Connection to {} timed out", address)))??;

        let tls_stream = connector.connect(address, stream).await?;
        let tls_stream = Arc::new(Mutex::new(tls_stream));

        let new_peer = Peer {
            address: address.to_string(),
            stream: tls_stream,
        };

        {
            let mut peers_guard = self.peers.write().await;
            if peers_guard.len() >= self.max_peers {
                return Err(NetworkingError::Network("Max peer limit reached".into()));
            }
            peers_guard.push(new_peer);
        }

        info!("Connected to peer at {}", address);
        Ok(())
    }

    /// Broadcasts a message to all connected peers.
    pub async fn broadcast_message(&self, message: &str) -> NetworkingResult<()> {
        let peers_snapshot = self.peers.read().await.clone();

        for peer in peers_snapshot.iter() {
            let peer_clone = peer.clone();
            let message_copy = message.to_string();
            let result = tokio::spawn(async move {
                let mut locked_stream = peer_clone.stream.lock().await;
                locked_stream.write_all(message_copy.as_bytes()).await
                    .map_err(|e| NetworkingError::Io(e))
            }).await;

            if let Err(e) = result {
                error!("Failed to send message to peer {}: {:?}", peer.address, e);
                self.remove_peer(&peer.address).await?;
            }
        }

        Ok(())
    }

    /// Removes a disconnected or faulty peer from the list.
    pub async fn remove_peer(&self, address: &str) -> NetworkingResult<()> {
        let mut peers = self.peers.write().await;
        peers.retain(|p| p.address != address);
        warn!("Removed disconnected peer: {}", address);
        Ok(())
    }

    /// Stops the networking component and disconnects all peers.
    pub async fn stop(&self) -> NetworkingResult<()> {
        let mut peers = self.peers.write().await;

        for peer in peers.drain(..) {
            let mut locked_stream = peer.stream.lock().await;
            if let Err(e) = locked_stream.shutdown().await {
                error!("Failed to close peer connection {}: {:?}", peer.address, e);
            }
        }

        info!("Networking component stopped.");
        Ok(())
    }

    /// Handles an incoming client connection.
    async fn handle_client_connection(
        &self,
        stream: TcpStream,
        peer_addr: std::net::SocketAddr,
        acceptor: TlsAcceptor,
    ) -> NetworkingResult<()> {
        let tls_stream = acceptor.accept(stream).await?;
        let tls_stream = Arc::new(Mutex::new(tls_stream));

        let new_peer = Peer {
            address: peer_addr.to_string(),
            stream: tls_stream.clone(),
        };

        {
            let mut peers_guard = self.peers.write().await;
            if peers_guard.len() >= self.max_peers {
                return Err(NetworkingError::Network("Max peer limit reached".into()));
            }
            peers_guard.push(new_peer);
        }

        self.handle_peer_communication(tls_stream, peer_addr.to_string()).await
    }

    /// Handles ongoing communication with a peer.
    async fn handle_peer_communication(
        &self,
        stream: Arc<Mutex<tokio_native_tls::TlsStream<TcpStream>>>,
        peer_address: String,
    ) -> NetworkingResult<()> {
        let mut buffer = [0; 1024];

        loop {
            let mut locked_stream = stream.lock().await;

            match tokio::time::timeout(Duration::from_secs(30), locked_stream.read(&mut buffer)).await {
                Ok(Ok(0)) => {
                    info!("Peer {} disconnected gracefully", peer_address);
                    break;
                }
                Ok(Ok(n)) => {
                    let message = String::from_utf8_lossy(&buffer[..n]);
                    debug!("Received message from {}: {}", peer_address, message);
                    self.process_message(&peer_address, &message).await?;
                }
                Ok(Err(e)) => {
                    error!("Error reading from peer {}: {:?}", peer_address, e);
                    break;
                }
                Err(_) => {
                    warn!("Read timeout from peer {}", peer_address);
                    // Optionally implement a ping mechanism here
                }
            }
        }

        self.remove_peer(&peer_address).await
    }

    /// Processes a received message.
    async fn process_message(&self, sender: &str, message: &str) -> NetworkingResult<()> {
        // Implement message processing logic here
        // For now, we'll just echo the message back to all peers except the sender
        let response = format!("Echo from {}: {}", sender, message);
        let peers_snapshot = self.peers.read().await.clone();

        for peer in peers_snapshot.iter() {
            if peer.address != sender {
                let mut locked_stream = peer.stream.lock().await;
                locked_stream.write_all(response.as_bytes()).await
                    .map_err(|e| NetworkingError::Io(e))?;
            }
        }

        Ok(())
    }

    /// Returns the number of connected peers.
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Returns a list of connected peer addresses.
    pub async fn get_peer_addresses(&self) -> Vec<String> {
        self.peers.read().await.iter().map(|p| p.address.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_networking_instance() {
        let networking = Networking::new(10, Duration::from_secs(5));
        assert_eq!(networking.peer_count().await, 0);
        assert!(networking.identity.is_none());
    }

    #[test]
    fn test_load_tls_identity() {
        let cert_path = "path/to/cert.pem";
        let key_path = "path/to/key.pem";

        let result = Networking::load_tls_identity(cert_path, key_path);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_to_peer() {
        let networking = Networking::new(10, Duration::from_secs(1));
        let result = networking.connect_to_peer("127.0.0.1:1234").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let networking = Networking::new(10, Duration::from_secs(5));
        let result = networking.broadcast_message("Test message").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop_networking() {
        let networking = Networking::new(10, Duration::from_secs(5));
        let result = networking.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_peer_limit() {
        let max_peers = 2;
        let networking = Networking::new(max_peers, Duration::from_secs(5));
        
        // Manually add peers to test the limit
        for i in 0..max_peers {
            let mut peers = networking.peers.write().await;
            peers.push(Peer {
                address: format!("127.0.0.1:{}", 8000 + i),
                stream: Arc::new(Mutex::new(tokio_native_tls::TlsStream::from(TcpStream::connect("127.0.0.1:1234").await.unwrap()))),
            });
        }

        // Attempt to add one more peer
        let result = networking.connect_to_peer("127.0.0.1:9000").await;
        assert!(result.is_err());
        assert_eq!(networking.peer_count().await, max_peers);
    }

    #[tokio::test]
    async fn test_remove_peer() {
        let networking = Networking::new(10, Duration::from_secs(5));
        
        // Manually add a peer
        {
            let mut peers = networking.peers.write().await;
            peers.push(Peer {
                address: "127.0.0.1:8000".to_string(),
                stream: Arc::new(Mutex::new(tokio_native_tls::TlsStream::from(TcpStream::connect("127.0.0.1:1234").await.unwrap()))),
            });
        }

        assert_eq!(networking.peer_count().await, 1);

        // Remove the peer
        let result = networking.remove_peer("127.0.0.1:8000").await;
        assert!(result.is_ok());
        assert_eq!(networking.peer_count().await, 0);
    }
}