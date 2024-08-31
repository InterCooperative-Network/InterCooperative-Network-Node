// Filename: icn_networking/src/lib.rs

use std::fs::File;
use std::io::Read;
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio_native_tls::{TlsAcceptor, TlsConnector};
use native_tls::Identity;
use icn_shared::{IcnError, IcnResult};
use log::{info, error, warn};

/// The `Networking` struct is responsible for managing peer-to-peer network connections
/// in a secure manner using TLS (Transport Layer Security). It allows for starting a server,
/// connecting to peers, and broadcasting messages to all connected peers.
#[derive(Clone)]
pub struct Networking {
    peers: Arc<RwLock<Vec<Arc<Mutex<tokio_native_tls::TlsStream<TcpStream>>>>>>,
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
    /// * `IcnResult<Arc<Identity>>` - The loaded TLS identity wrapped in an Arc, or an error if loading fails.
    ///
    /// # Errors
    ///
    /// * `IcnError::Network` - If the certificate or key file is not found, or if there's an error reading or parsing the files.
    pub fn load_tls_identity(cert_path: &str, key_path: &str) -> IcnResult<Arc<Identity>> {
        // Check if the certificate file exists
        if !std::path::Path::new(cert_path).exists() {
            return Err(IcnError::Network(format!("Certificate file not found: {}", cert_path)));
        }

        // Check if the key file exists
        if !std::path::Path::new(key_path).exists() {
            return Err(IcnError::Network(format!("Key file not found: {}", key_path)));
        }

        // Open and read the certificate file
        let mut cert_file = File::open(cert_path)
            .map_err(|e| IcnError::Network(format!("Failed to open certificate file: {}", e)))?;
        let mut cert = Vec::new();
        cert_file.read_to_end(&mut cert)
            .map_err(|e| IcnError::Network(format!("Failed to read certificate file: {}", e)))?;

        // Open and read the key file
        let mut key_file = File::open(key_path)
            .map_err(|e| IcnError::Network(format!("Failed to open key file: {}", e)))?;
        let mut key = Vec::new();
        key_file.read_to_end(&mut key)
            .map_err(|e| IcnError::Network(format!("Failed to read key file: {}", e)))?;

        // Load the identity from the certificate and key
        let identity = Identity::from_pkcs8(&cert, &key)
            .map_err(|e| IcnError::Network(format!("Failed to load TLS identity: {}", e)))?;

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
    /// * `IcnResult<()>` - An empty result indicating success, or an error if starting the server fails.
    ///
    /// # Errors
    ///
    /// * `IcnError::Network` - If there's an error creating the TLS acceptor, binding to the address, or accepting a TCP connection.
    pub async fn start_server(&mut self, address: &str, identity: Arc<Identity>) -> IcnResult<()> {
        // Store the identity for future use
        self.identity = Some(identity.clone());

        // Create a TLS acceptor from the identity
        let acceptor = TlsAcceptor::from(
            native_tls::TlsAcceptor::new(identity.as_ref().clone())
                .map_err(|e| IcnError::Network(format!("Failed to create TLS acceptor: {}", e)))?
        );

        // Bind a TCP listener to the specified address
        let listener = TcpListener::bind(address).await
            .map_err(|e| IcnError::Network(format!("Failed to bind to address: {}", e)))?;
        info!("Server started on {}", address);

        // Accept incoming connections in a loop
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    // Clone necessary data for the spawned task
                    let acceptor = acceptor.clone();
                    let peers = Arc::clone(&self.peers);

                    // Spawn a task to handle the client connection
                    tokio::spawn(async move {
                        // Handle the client connection and log any errors
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
    /// * `IcnResult<()>` - An empty result indicating success, or an error if the connection fails.
    ///
    /// # Errors
    ///
    /// * `IcnError::Network` - If there's an error creating the TLS connector, connecting to the peer, establishing the TLS connection, or acquiring the peers lock.
    pub async fn connect_to_peer(&self, address: &str) -> IcnResult<()> {
        // Create a TLS connector
        let connector = TlsConnector::from(
            native_tls::TlsConnector::new()
                .map_err(|e| IcnError::Network(format!("Failed to create TLS connector: {}", e)))?
        );

        // Connect a TCP stream to the peer address
        let stream = TcpStream::connect(address).await
            .map_err(|e| IcnError::Network(format!("Failed to connect to peer: {}", e)))?;

        // Establish a TLS connection over the TCP stream
        let tls_stream = connector.connect(address, stream).await
            .map_err(|e| IcnError::Network(format!("Failed to establish TLS connection: {}", e)))?;

        // Wrap the TLS stream in an Arc<Mutex> for shared, asynchronous access
        let tls_stream = Arc::new(Mutex::new(tls_stream));

        // Add the new peer to the list of connected peers
        {
            let mut peers_guard = self.peers.write().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
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
    /// * `IcnResult<()>` - An empty result indicating success, or an error if broadcasting fails.
    ///
    /// # Errors
    ///
    /// * `IcnError::Network` - If there's an error acquiring the peers lock or sending the message to a peer.
    pub async fn broadcast_message(&self, message: &str) -> IcnResult<()> {
        // Collect all peers in a temporary vector to avoid holding the lock while sending messages
        let peers_snapshot = {
            let peers = self.peers.read().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
            peers.clone()
        };

        // Iterate over the snapshot of peers and send the message to each one
        for peer in peers_snapshot.iter() {
            let mut locked_peer = peer.lock().await;
            // Attempt to write the message to the peer, handling potential errors
            if let Err(e) = locked_peer.write_all(message.as_bytes()).await {
                // Log the error and remove the peer from the list if the write fails
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
    /// * `IcnResult<()>` - An empty result indicating success, or an error if the removal fails.
    ///
    /// # Errors
    ///
    /// * `IcnError::Network` - If there's an error acquiring the peers lock.
    async fn remove_peer(&self, peer: &Arc<Mutex<tokio_native_tls::TlsStream<TcpStream>>>) -> IcnResult<()> {
        let mut peers = self.peers.write().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
        peers.retain(|p| !Arc::ptr_eq(p, peer));
        warn!("Removed disconnected peer");
        Ok(())
    }

    /// Stops the networking component and disconnects all peers.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - An empty result indicating success or an error if stopping fails.
    pub async fn stop(&self) -> IcnResult<()> {
        // Acquire a write lock on the list of peers to modify it
        let mut peers = self.peers.write().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;

        // Iterate over the peers and attempt to shut down each connection
        for peer in peers.iter() {
            let mut locked_peer = peer.lock().await;
            if let Err(e) = locked_peer.shutdown().await {
                // Log an error if shutting down a peer connection fails
                error!("Failed to close peer connection: {:?}", e);
            }
        }

        // Clear the list of peers after disconnecting them
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
/// * `IcnResult<()>` - An empty result indicating success or an error if the connection fails.
async fn handle_client_connection(
    stream: TcpStream,
    acceptor: TlsAcceptor,
    peers: Arc<RwLock<Vec<Arc<Mutex<tokio_native_tls::TlsStream<TcpStream>>>>>>,
    _custom_arg: Option<()>
) -> IcnResult<()> {
    // Accept the TLS connection from the incoming TCP stream
    let tls_stream = acceptor.accept(stream).await
        .map_err(|e| IcnError::Network(format!("Failed to accept TLS connection: {:?}", e)))?;

    // Wrap the TLS stream in an Arc<Mutex> for shared, asynchronous access
    let tls_stream = Arc::new(Mutex::new(tls_stream));

    // Add the new peer to the list of connected peers
    {
        let mut peers_guard = peers.write().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
        peers_guard.push(tls_stream.clone());
    }

    // Handle communication with the connected client
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
/// * `IcnResult<()>` - An empty result indicating success or an error if the connection fails.
async fn handle_client(
    stream: Arc<Mutex<tokio_native_tls::TlsStream<TcpStream>>>,
    peers: Arc<RwLock<Vec<Arc<Mutex<tokio_native_tls::TlsStream<TcpStream>>>>>>,
) -> IcnResult<()> {
    let mut buffer = [0; 1024];

    loop {
        // Acquire a lock on the TLS stream to read from it
        let mut locked_stream = stream.lock().await;

        // Attempt to read data from the stream into the buffer
        match locked_stream.read(&mut buffer).await {
            // Connection closed gracefully by the peer
            Ok(0) => {
                info!("Peer disconnected gracefully");
                break;
            }
            // Data successfully read from the stream
            Ok(n) => {
                // Convert the read bytes into a UTF-8 string (lossy conversion)
                let message = String::from_utf8_lossy(&buffer[..n]);
                info!("Received message: {}", message);

                // TODO: Process the received message here
            }
            // An error occurred while reading from the stream
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // This is not a real error, just indicates no data is currently available
                continue; 
            }
            // Other errors
            Err(e) => {
                // Log the error and break the loop to disconnect the peer
                error!("Error reading from stream: {:?}", e);
                break;
            }
        }
    }

    // Remove the disconnected peer from the list of peers
    {
        let mut peers = peers.write().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
        peers.retain(|p| !Arc::ptr_eq(p, &stream));
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
