//! # ICN Networking Module
//!
//! This module provides networking functionality for the InterCooperative Network (ICN) project.
//! It handles peer-to-peer connections, TLS encryption, and message broadcasting.
//!
//! The main struct, `Networking`, manages connections, peer information, and network operations.
//! It uses asynchronous I/O with tokio and provides a secure communication channel using TLS.

use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_native_tls::{TlsAcceptor, TlsConnector, TlsStream};
use native_tls::{Identity, TlsConnector as NativeTlsConnector};
use thiserror::Error;
use tokio::sync::{RwLock, Mutex};
use log::{info, error, warn, debug};
use std::time::Duration;

/// Custom error type for the networking module.
#[derive(Error, Debug)]
pub enum NetworkingError {
    /// Represents general network-related errors.
    #[error("Network error: {0}")]
    Network(String),
    
    /// Represents TLS-specific errors.
    #[error("TLS error: {0}")]
    Tls(#[from] native_tls::Error),
    
    /// Represents I/O errors.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Represents errors related to locking mechanisms.
    #[error("Lock error: {0}")]
    Lock(String),
    
    /// Represents timeout errors.
    #[error("Timeout error: {0}")]
    Timeout(String),
}

/// Type alias for results returned by networking functions.
pub type NetworkingResult<T> = Result<T, NetworkingError>;

/// Represents a peer in the network.
#[derive(Debug, Clone)]
struct Peer {
    /// The network address of the peer.
    address: String,
    /// The TLS-encrypted stream connected to the peer.
    stream: Arc<Mutex<TlsStream<TcpStream>>>,
}

/// The `Networking` struct is responsible for managing peer-to-peer network connections
/// in a secure manner using TLS (Transport Layer Security).
#[derive(Clone)]
pub struct Networking {
    /// List of connected peers.
    peers: Arc<RwLock<Vec<Peer>>>,
    /// TLS identity for the node.
    identity: Option<Arc<Identity>>,
    /// Maximum number of allowed peer connections.
    max_peers: usize,
    /// Timeout duration for connection attempts.
    connection_timeout: Duration,
}

impl Networking {
    /// Creates a new instance of the `Networking` struct.
    ///
    /// # Arguments
    ///
    /// * `max_peers` - The maximum number of peer connections allowed.
    /// * `connection_timeout` - The timeout duration for connection attempts.
    ///
    /// # Returns
    ///
    /// A new `Networking` instance.
    pub fn new(max_peers: usize, connection_timeout: Duration) -> Self {
        Networking {
            peers: Arc::new(RwLock::new(vec![])),
            identity: None,
            max_peers,
            connection_timeout,
        }
    }

    /// Loads a TLS identity from a certificate and key file.
    ///
    /// # Arguments
    ///
    /// * `cert_path` - Path to the certificate file.
    /// * `key_path` - Path to the private key file.
    ///
    /// # Returns
    ///
    /// A `NetworkingResult` containing the loaded `Identity` if successful.
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
    ///
    /// # Arguments
    ///
    /// * `address` - The address to bind the server to.
    /// * `identity` - The TLS identity to use for the server.
    ///
    /// # Returns
    ///
    /// A `NetworkingResult` indicating success or failure.
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
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the peer to connect to.
    ///
    /// # Returns
    ///
    /// A `NetworkingResult` indicating success or failure.
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
    ///
    /// # Arguments
    ///
    /// * `message` - The message to broadcast.
    ///
    /// # Returns
    ///
    /// A `NetworkingResult` indicating success or failure.
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
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the peer to remove.
    ///
    /// # Returns
    ///
    /// A `NetworkingResult` indicating success or failure.
    pub async fn remove_peer(&self, address: &str) -> NetworkingResult<()> {
        let mut peers = self.peers.write().await;
        peers.retain(|p| p.address != address);
        warn!("Removed disconnected peer: {}", address);
        Ok(())
    }

    /// Stops the networking component and disconnects all peers.
    ///
    /// # Returns
    ///
    /// A `NetworkingResult` indicating success or failure.
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
    ///
    /// # Arguments
    ///
    /// * `stream` - The incoming TCP stream.
    /// * `peer_addr` - The address of the connecting peer.
    /// * `acceptor` - The TLS acceptor for securing the connection.
    ///
    /// # Returns
    ///
    /// A `NetworkingResult` indicating success or failure.
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
    ///
    /// # Arguments
    ///
    /// * `stream` - The TLS stream for communication with the peer.
    /// * `peer_address` - The address of the peer.
    ///
    /// # Returns
    ///
    /// A `NetworkingResult` indicating success or failure.
    async fn handle_peer_communication(
        &self,
        stream: Arc<Mutex<TlsStream<TcpStream>>>,
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
    ///
    /// # Arguments
    ///
    /// * `sender` - The address of the sender.
    /// * `message` - The received message.
    ///
    /// # Returns
    ///
    /// A `NetworkingResult` indicating success or failure.
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
    ///
    /// # Returns
    ///
    /// The number of connected peers.
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Returns a list of connected peer addresses.
    ///
    /// # Returns
    ///
    /// A vector of peer addresses.
    pub async fn get_peer_addresses(&self) -> Vec<String> {
        self.peers.read().await.iter().map(|p| p.address.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_native_tls::native_tls;

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
            let dummy_stream = TcpStream::connect("127.0.0.1:1234").await.unwrap();
            let dummy_tls_stream = TlsStream::new(dummy_stream);
            peers.push(Peer {
                address: format!("127.0.0.1:{}", 8000 + i),
                stream: Arc::new(Mutex::new(dummy_tls_stream)),
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
            let dummy_stream = TcpStream::connect("127.0.0.1:1234").await.unwrap();
            let dummy_tls_stream = TlsStream::new(dummy_stream);
            peers.push(Peer {
                address: "127.0.0.1:8000".to_string(),
                stream: Arc::new(Mutex::new(dummy_tls_stream)),
            });
        }

        assert_eq!(networking.peer_count().await, 1);

        // Remove the peer
        let result = networking.remove_peer("127.0.0.1:8000").await;
        assert!(result.is_ok());
        assert_eq!(networking.peer_count().await, 0);
    }

    #[tokio::test]
    async fn test_get_peer_addresses() {
        let networking = Networking::new(10, Duration::from_secs(5));
        
        // Manually add some peers
        {
            let mut peers = networking.peers.write().await;
            for i in 0..3 {
                let dummy_stream = TcpStream::connect("127.0.0.1:1234").await.unwrap();
                let dummy_tls_stream = TlsStream::new(dummy_stream);
                peers.push(Peer {
                    address: format!("127.0.0.1:{}", 8000 + i),
                    stream: Arc::new(Mutex::new(dummy_tls_stream)),
                });
            }
        }

        let addresses = networking.get_peer_addresses().await;
        assert_eq!(addresses.len(), 3);
        assert!(addresses.contains(&"127.0.0.1:8000".to_string()));
        assert!(addresses.contains(&"127.0.0.1:8001".to_string()));
        assert!(addresses.contains(&"127.0.0.1:8002".to_string()));
    }

    #[tokio::test]
    async fn test_handle_client_connection() {
        let networking = Networking::new(10, Duration::from_secs(5));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Create a dummy TLS acceptor for testing
        let cert = native_tls::Certificate::from_pem(b"-----BEGIN CERTIFICATE-----\nMIIDazCCAlOgAwIBAgIUBY1hlCGvdj4NhBXkZ/uLUZNILAwwDQYJKoZIhvcNAQEL\nBQAwRTELMAkGA1UEBhMCQVUxEzARBgNVBAgMClNvbWUtU3RhdGUxITAfBgNVBAoM\nGEludGVybmV0IFdpZGdpdHMgUHR5IEx0ZDAeFw0xOTAzMjEwMDQzNTJaFw0yMDAz\nMjAwMDQzNTJaMEUxCzAJBgNVBAYTAkFVMRMwEQYDVQQIDApTb21lLVN0YXRlMSEw\nHwYDVQQKDBhJbnRlcm5ldCBXaWRnaXRzIFB0eSBMdGQwggEiMA0GCSqGSIb3DQEB\nAQUAA4IBDwAwggEKAoIBAQCy+ZrIvwwiZv4bPmvKx/637ltZLwfgh0+kZxoZxp/h\nMq6E09EQJrcpBsNd2udZGEGvudV7B6umbGbUZVWYUbc2oQCwITVCb+XLUyB3znps\nqk7LEzCpReJMuW2Q2P5NRtIJLnxTi3tLmjzrsaOFs5EUgBMM8PGxh+AVMiOVH+EB\nlW4vRNhY2LCdjvQpWIFwS4TnQN4H0722ESS/\x08Jc2vaQZI/HXzacXhb8lQOTpT\nF1f7KYNBogmpMHduyfBaMj7hzZL5TvXUKuRSu3XtHSKfYIGq6JSwIIXB3Sdd3tnX\n6JHgBTvIbHdhfR7o2mUpTQRomlY6/YEEApzoIu4prhWNAgMBAAGjUzBRMB0GA1Ud\nDgQWBBRwuBCzFB2qaIVrFEOCppqA0XkW6jAfBgNVHSMEGDAWgBRwuBCzFB2qaIVr\nFEOCppqA0XkW6jAPBgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUAA4IBAQB9\nLYMfYZlVMRMZIxvwFEY+DjGwuz+/s2fmL1QfNucv37YmDDOVc2yZVPqHOIQohLPA\nZCYAHpGK70LJyu3YLmDPJNwO/bXEeGG7MOgW9Z+7f0nZo8oUc5tNIKdnNm15Iaxu\ncxk6xtxP0J9lZRX/yMoTOXhX8jD4nYbwkM5k0fcLRnGJG98oPMcU4h80MmQYGd5+\nwPHJZ8K/YVZhZl8OOpCQxkZaFYHoGv0AgQzIB2Eip3jkqwvp2vqLR3K/W4O4B0dM\n4xC1iy3uWJnYP0jrePH/ZIQTL6kaTxoEQ0fLkz9+y9zzbCdoKs6G2bZtnMlXtqzO\nqUAFe9z7N5ruZ5WzZCZS\n-----END CERTIFICATE-----\n").unwrap();
        let key = native_tls::Identity::from_pkcs8(b"-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQCy+ZrIvwwiZv4b\nPmvKx/637ltZLwfgh0+kZxoZxp/hMq6E09EQJrcpBsNd2udZGEGvudV7B6umbGbU\nZVWYUbc2oQCwITVCb+XLUyB3znpsqk7LEzCpReJMuW2Q2P5NRtIJLnxTi3tLmjzr\nsaOFs5EUgBMM8PGxh+AVMiOVH+EBlW4vRNhY2LCdjvQpWIFwS4TnQN4H0722ESS/\n\x08Jc2vaQZI/HXzacXhb8lQOTpTF1f7KYNBogmpMHduyfBaMj7hzZL5TvXUKuRS\nu3XtHSKfYIGq6JSwIIXB3Sdd3tnX6JHgBTvIbHdhfR7o2mUpTQRomlY6/YEEApzo\nIu4prhWNAgMBAAECggEBAKA7P3027R/KP0eSCzcqHIZIGpoRHg1QkSK5hzZh2Dm6\nwMvBOZWzDKK6mNc1+hK78oqxp6N+yWewx/JEyUkF5n/5m0kk8aRZM/vbWtaGHQV2\nBOjm2RagpNhz7pZNSLpjOb/Wv1HxGGiYo0o9oZ/+7iW8S+FNJ/+vSNV2bK2xWx+d\nIDPUgOoQzTrSN3Aa5OgG0GJaJ5U0r6C+rCSlrxPQRlTr7Sq3cVgC7BUIe19LIw2d\nNWHtwcV0Q3AVpb4A+LgIBsKQmMB0B3/CIjYPuR3HNfrkT9YDl/qERGlPVkzP8l+R\nHGG2aGAHpHD8XbmkSgSh1pM8UNjFoFHyPgq1RvQLdqECgYEA4MzOPVuqTGdU3g6D\nvQW9r/4yKN1AVB6d4nwzOSWC22PRK7vuwZ/WxT8bjgkw7mfWtDtPm/Utn+8fsPHk\nL4mRpWzWOKTTWY86+iYL48u/GokLz0rW1lNTc3kXiNbONXwhRMDhQ2/5QUKFEHLe\nqczHhfkVIyOXwJN60LXcmg4a62sCgYEAzJJTBFDwZtSAXQPnBhVTGXmwEbJSAcFb\nHMT0RI2A+9SYnp6fdhJg4eyQEUNqmg+j0p+y0cNY1tTTsZaCn2QBB0fzaJd+Z/UZ\nGQrwZwF/uq+Zy1ItQQEsRp5rVAmzRRyPshdXgrcj4rKrbvfvhN0DMCzMa1nqRGhR\nN+X5yNSNbm8CgYAVLGM7oMx5G8DX5TlbfjHb95S5bCYpBnFduwppLN+0vt7H6Gqw\n5v3mXdFnXlE4MYVqrfKy2qF8/tg+vIz5PKCiC9lfvBN3e2B3zcYrqg2xA9pV3n4W\nxBWJbN72dK9N2LOIxAkM3PUmPKFPbX1Xm7I4AJ5RStxKJmAZ1m1q2zGTXQKBgCGR\n6kTvNbjpwq6KoiJSw9uMZBOz8JgY1M+APD+wHtRn/fPV/uQmjF6YkhDvD/1/K1Tx\n4q/1BQaFCPOYsj4B09nCBZNNKlVNFOQGmH39pAwOFgDxsHVkuPGmvuhXwCGnFiK2\nAe41FgH5Cv2bKLg7RcE7vx3OVwqXZBGKkNWlNNexAoGBALCW44xCbPa1Ks5MApS1\nAaQRFuFmRCrLVDO9S+w6RcJFdHDMOIw+XwX+8YHHX0gHqYr/ZhhdJ4G9QU/RgKM6\nk/6LQf1g0MUArqOiyfcv+nBxeDAv39IHGS7/YTHpuXSaQkjRUwBHnrTRYgAZeqA0\nqrfuRpXHKKChOwcVJQ3OZVBw\n-----END PRIVATE KEY-----\n", b"").unwrap();
        let acceptor = TlsAcceptor::from(native_tls::TlsAcceptor::new(key).unwrap());

        // Handle the client connection in a separate task
        let networking_clone = networking.clone();
        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                networking_clone.handle_client_connection(stream, addr, acceptor).await.unwrap();
            }
        });

        // Give some time for the connection to be established
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(networking.peer_count().await, 1);
        let addresses = networking.get_peer_addresses().await;
        assert!(addresses.contains(&addr.to_string()));
    }
}