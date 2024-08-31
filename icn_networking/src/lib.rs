// Filename: icn_networking/src/lib.rs

use std::fs::File;
use std::io::Read;
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_native_tls::{TlsAcceptor, TlsConnector};
use native_tls::Identity;
use futures::lock::Mutex as FuturesMutex;
use thiserror::Error;
use log::{info, error, warn};

/// Custom error type for the networking module.
#[derive(Error, Debug)]
pub enum NetworkingError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("TLS error: {0}")]
    Tls(#[from] native_tls::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Lock error")]
    Lock,
}

/// Type alias for results returned by networking functions.
pub type NetworkingResult<T> = Result<T, NetworkingError>;

/// The `Networking` struct is responsible for managing peer-to-peer network connections
/// in a secure manner using TLS (Transport Layer Security). It allows for starting a server,
/// connecting to peers, and broadcasting messages to all connected peers.
#[derive(Clone)]
pub struct Networking {
    peers: Arc<RwLock<Vec<Arc<FuturesMutex<tokio_native_tls::TlsStream<TcpStream>>>>>>,
    identity: Option<Arc<Identity>>,
}

impl Networking {
    /// Creates a new instance of the `Networking` struct.
    ///
    /// # Returns
    ///
    /// * `Networking` - An instance of the `Networking` struct with an empty list of peers.
    pub fn new() -> Self {
        Networking {
            peers: Arc::new(RwLock::new(vec![])),
            identity: None,
        }
    }

    /// Loads a TLS identity from a certificate and key file.
    ///
    /// # Arguments
    ///
    /// * `cert_path` - The path to the TLS certificate file.
    /// * `key_path` - The path to the TLS key file.
    ///
    /// # Returns
    ///
    /// * `NetworkingResult<Arc<Identity>>` - The loaded TLS identity wrapped in an Arc, or an error if loading fails.
    pub fn load_tls_identity(cert_path: &str, key_path: &str) -> NetworkingResult<Arc<Identity>> {
        if !std::path::Path::new(cert_path).exists() {
            return Err(NetworkingError::Network(format!("Certificate file not found: {}", cert_path)));
        }

        if !std::path::Path::new(key_path).exists() {
            return Err(NetworkingError::Network(format!("Key file not found: {}", key_path)));
        }

        let mut cert_file = File::open(cert_path)?;
        let mut cert = Vec::new();
        cert_file.read_to_end(&mut cert)?;

        let mut key_file = File::open(key_path)?;
        let mut key = Vec::new();
        key_file.read_to_end(&mut key)?;

        let identity = Identity::from_pkcs8(&cert, &key)?;

        Ok(Arc::new(identity))
    }

    /// Starts a server that listens on the specified address using the provided TLS identity.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to bind the server to.
    /// * `identity` - The TLS identity to use for secure connections.
    ///
    /// # Returns
    ///
    /// * `NetworkingResult<()>` - An empty result indicating success, or an error if starting the server fails.
    pub async fn start_server(&mut self, address: &str, identity: Arc<Identity>) -> NetworkingResult<()> {
        self.identity = Some(identity.clone());

        let acceptor = TlsAcceptor::from(
            native_tls::TlsAcceptor::new(identity.as_ref().clone())?
        );

        let listener = TcpListener::bind(address).await?;
        info!("Server started on {}", address);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let acceptor = acceptor.clone();
                    let peers = Arc::clone(&self.peers);

                    tokio::spawn(async move {
                        if let Err(e) = handle_client_connection(stream, acceptor, peers, None).await {
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
    /// * `NetworkingResult<()>` - An empty result indicating success, or an error if the connection fails.
    pub async fn connect_to_peer(&self, address: &str) -> NetworkingResult<()> {
        let connector = TlsConnector::from(
            native_tls::TlsConnector::new()?
        );

        let stream = TcpStream::connect(address).await?;

        let tls_stream = connector.connect(address, stream).await?;

        let tls_stream = Arc::new(FuturesMutex::new(tls_stream));

        {
            let mut peers_guard = self.peers.write().map_err(|_| NetworkingError::Lock)?;
            peers_guard.push(tls_stream.clone());
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
    /// * `NetworkingResult<()>` - An empty result indicating success, or an error if broadcasting fails.
    pub async fn broadcast_message(&self, message: &str) -> NetworkingResult<()> {
        let peers_snapshot = {
            let peers = self.peers.read().map_err(|_| NetworkingError::Lock)?;
            peers.clone()
        };

        for peer in peers_snapshot.iter() {
            let mut locked_peer = peer.lock().await;
            if let Err(e) = locked_peer.write_all(message.as_bytes()).await {
                error!("Failed to send message to peer: {}", e);
                if let Err(e) = self.remove_peer(peer).await {
                    error!("Failed to remove peer: {:?}", e);
                }
            }
        }
        Ok(())
    }

    /// Removes a disconnected or faulty peer from the list.
    ///
    /// # Arguments
    ///
    /// * `peer` - The peer to be removed.
    ///
    /// # Returns
    ///
    /// * `NetworkingResult<()>` - An empty result indicating success, or an error if the removal fails.
    async fn remove_peer(&self, peer: &Arc<FuturesMutex<tokio_native_tls::TlsStream<TcpStream>>>) -> NetworkingResult<()> {
        let mut peers = self.peers.write().map_err(|_| NetworkingError::Lock)?;
        peers.retain(|p| !Arc::ptr_eq(p, peer));
        warn!("Removed disconnected peer");
        Ok(())
    }

    /// Stops the networking component and disconnects all peers.
    ///
    /// # Returns
    ///
    /// * `NetworkingResult<()>` - An empty result indicating success or an error if stopping fails.
    pub async fn stop(&self) -> NetworkingResult<()> {
        let mut peers = self.peers.write().map_err(|_| NetworkingError::Lock)?;

        for peer in peers.iter() {
            let mut locked_peer = peer.lock().await;
            if let Err(e) = locked_peer.shutdown().await {
                error!("Failed to close peer connection: {:?}", e);
            }
        }

        peers.clear();
        info!("Networking component stopped.");
        Ok(())
    }
}

/// Handles an incoming client connection, establishing a secure TLS stream
/// and managing communication with the peer.
///
/// # Arguments
///
/// * `stream` - The TCP stream representing the client connection.
/// * `acceptor` - The TLS acceptor used to secure the connection.
/// * `peers` - The list of currently connected peers.
/// * `_custom_arg` - A placeholder for future use.
///
/// # Returns
///
/// * `NetworkingResult<()>` - An empty result indicating success or an error if the connection fails.
async fn handle_client_connection(
    stream: TcpStream,
    acceptor: TlsAcceptor,
    peers: Arc<RwLock<Vec<Arc<FuturesMutex<tokio_native_tls::TlsStream<TcpStream>>>>>>,
    _custom_arg: Option<()>
) -> NetworkingResult<()> {
    let tls_stream = acceptor.accept(stream).await?;

    let tls_stream = Arc::new(FuturesMutex::new(tls_stream));

    {
        let mut peers_guard = peers.write().map_err(|_| NetworkingError::Lock)?;
        peers_guard.push(tls_stream.clone());
    }

    handle_client(tls_stream, peers).await
}

/// Manages communication with a connected peer, reading and processing messages.
///
/// # Arguments
///
/// * `stream` - The secure TLS stream for the peer connection.
/// * `peers` - The list of currently connected peers.
///
/// # Returns
///
/// * `NetworkingResult<()>` - An empty result indicating success or an error if the connection fails.
async fn handle_client(
    stream: Arc<FuturesMutex<tokio_native_tls::TlsStream<TcpStream>>>,
    peers: Arc<RwLock<Vec<Arc<FuturesMutex<tokio_native_tls::TlsStream<TcpStream>>>>>>,
) -> NetworkingResult<()> {
    let mut buffer = [0; 1024];

    loop {
        let mut locked_stream = stream.lock().await;

        match locked_stream.read(&mut buffer).await {
            Ok(0) => {
                info!("Peer disconnected gracefully");
                break;
            }
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]);
                info!("Received message: {}", message);
                // TODO: Process the received message here
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                error!("Error reading from stream: {:?}", e);
                break;
            }
        }
    }

    {
        let mut peers_guard = peers.write().map_err(|_| NetworkingError::Lock)?;
        peers_guard.retain(|p| !Arc::ptr_eq(p, &stream));
        warn!("Removed disconnected peer");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_tls_identity() {
        let cert_path = "path/to/cert.pem";
        let key_path = "path/to/key.pem";

        let result = Networking::load_tls_identity(cert_path, key_path);

        assert!(result.is_err()); // Expect an error because the files don't exist
    }

    #[test]
    fn test_create_networking_instance() {
        let networking = Networking::new();
        assert!(networking.peers.read().unwrap().is_empty());
        assert!(networking.identity.is_none());
    }

    #[tokio::test]
    async fn test_start_server() {
        let cert_path = "path/to/cert.pem";
        let key_path = "path/to/key.pem";

        let identity = Networking::load_tls_identity(cert_path, key_path).unwrap();
        let mut networking = Networking::new();

        let result = networking.start_server("127.0.0.1:0", identity).await;

        assert!(result.is_err()); // Expect an error because the files don't exist
    }

    #[tokio::test]
    async fn test_connect_to_peer() {
        let networking = Networking::new();
        let result = networking.connect_to_peer("127.0.0.1:0").await;

        assert!(result.is_err()); // Expect an error because the peer is not available
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let networking = Networking::new();
        let result = networking.broadcast_message("Test message").await;

        assert!(result.is_ok()); // Expect success even if no peers are connected
    }

    #[tokio::test]
    async fn test_stop_networking() {
        let networking = Networking::new();
        let result = networking.stop().await;

        assert!(result.is_ok()); // Expect success even if no peers are connected
    }
}
