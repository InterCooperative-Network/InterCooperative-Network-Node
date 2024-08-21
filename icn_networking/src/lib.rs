use std::fs::File;
use std::io::Read;
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex; // Using Tokio's async Mutex
use tokio_native_tls::{TlsAcceptor, TlsConnector};
use native_tls::Identity;
use icn_shared::{IcnError, IcnResult};
use log::{info, error};

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
    /// * `password` - The password for the TLS key.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Arc<Identity>>` - The loaded TLS identity wrapped in an Arc, or an error if loading fails.
    pub fn load_tls_identity(cert_path: &str, key_path: &str, _password: &str) -> IcnResult<Arc<Identity>> {
        if !std::path::Path::new(cert_path).exists() {
            return Err(IcnError::Network(format!("Certificate file not found: {}", cert_path)));
        }

        if !std::path::Path::new(key_path).exists() {
            return Err(IcnError::Network(format!("Key file not found: {}", key_path)));
        }

        let mut cert_file = File::open(cert_path)
            .map_err(|e| IcnError::Network(format!("Failed to open certificate file: {}", e)))?;
        let mut cert = Vec::new();
        cert_file.read_to_end(&mut cert)
            .map_err(|e| IcnError::Network(format!("Failed to read certificate file: {}", e)))?;

        let mut key_file = File::open(key_path)
            .map_err(|e| IcnError::Network(format!("Failed to open key file: {}", e)))?;
        let mut key = Vec::new();
        key_file.read_to_end(&mut key)
            .map_err(|e| IcnError::Network(format!("Failed to read key file: {}", e)))?;

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
    /// * `IcnResult<()>` - An empty result indicating success or an error if starting the server fails.
    pub async fn start_server(&mut self, address: &str, identity: Arc<Identity>) -> IcnResult<()> {
        self.identity = Some(identity.clone());
        let acceptor = TlsAcceptor::from(
            native_tls::TlsAcceptor::new((*identity).clone())
                .map_err(|e| IcnError::Network(format!("Failed to create TLS acceptor: {}", e)))?
        );

        let listener = TcpListener::bind(address).await
            .map_err(|e| IcnError::Network(format!("Failed to bind to address: {}", e)))?;
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
    /// * `IcnResult<()>` - An empty result indicating success or an error if the connection fails.
    pub async fn connect_to_peer(&self, address: &str) -> IcnResult<()> {
        let connector = TlsConnector::from(
            native_tls::TlsConnector::new()
                .map_err(|e| IcnError::Network(format!("Failed to create TLS connector: {}", e)))?
        );

        let stream = TcpStream::connect(address).await
            .map_err(|e| IcnError::Network(format!("Failed to connect to peer: {}", e)))?;

        let tls_stream = connector.connect(address, stream).await
            .map_err(|e| IcnError::Network(format!("Failed to establish TLS connection: {}", e)))?;

        let tls_stream = Arc::new(Mutex::new(tls_stream));

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
    /// * `IcnResult<()>` - An empty result indicating success or an error if the message could not be sent.
    pub async fn broadcast_message(&self, message: &str) -> IcnResult<()> {
        let peers = self.peers.read().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
        for peer in peers.iter() {
            let mut locked_peer = peer.lock().await;
            locked_peer.write_all(message.as_bytes()).await
                .map_err(|e| IcnError::Network(format!("Failed to send message: {}", e)))?;
        }
        Ok(())
    }

    /// Stops the networking component and disconnects all peers.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - An empty result indicating success or an error if stopping fails.
    pub async fn stop(&self) -> IcnResult<()> {
        let mut peers = self.peers.write().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
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
/// * `IcnResult<()>` - An empty result indicating success or an error if the connection fails.
async fn handle_client_connection(
    stream: TcpStream,
    acceptor: TlsAcceptor,
    peers: Arc<RwLock<Vec<Arc<Mutex<tokio_native_tls::TlsStream<TcpStream>>>>>>,
    _custom_arg: Option<()>
) -> IcnResult<()> {
    let tls_stream = acceptor.accept(stream).await
        .map_err(|e| IcnError::Network(format!("Failed to accept TLS connection: {:?}", e)))?;

    let tls_stream = Arc::new(Mutex::new(tls_stream));

    {
        let mut peers_guard = peers.write().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
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
/// * `IcnResult<()>` - An empty result indicating success or an error if the connection fails.
async fn handle_client(
    stream: Arc<Mutex<tokio_native_tls::TlsStream<TcpStream>>>, 
    peers: Arc<RwLock<Vec<Arc<Mutex<tokio_native_tls::TlsStream<TcpStream>>>>>>,
) -> IcnResult<()> {
    let mut buffer = [0; 1024];

    loop {
        let mut locked_stream = stream.lock().await;
        match locked_stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]);
                info!("Received message: {}", message);
            }
            Err(e) => {
                error!("Error reading from stream: {:?}", e);
                break;
            }
        }
    }

    let mut peers = peers.write().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
    peers.retain(|p| !Arc::ptr_eq(p, &stream));
    Ok(())
}

