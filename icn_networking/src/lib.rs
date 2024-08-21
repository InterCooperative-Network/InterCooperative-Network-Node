use std::sync::{Arc, RwLock};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_native_tls::{TlsAcceptor, TlsConnector};
use native_tls::Identity;
use icn_shared::{IcnError, IcnResult};
use log::{info, error};

/// The `Networking` struct is responsible for managing peer-to-peer network connections
/// in a secure manner using TLS (Transport Layer Security). It allows for starting a server,
/// connecting to peers, and broadcasting messages to all connected peers.
#[derive(Clone)]
pub struct Networking {
    peers: Arc<RwLock<Vec<tokio_native_tls::TlsStream<TcpStream>>>>,
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
    /// * `IcnResult<Identity>` - The loaded TLS identity or an error if loading fails.
    pub fn load_tls_identity(cert_path: &str, key_path: &str, _password: &str) -> IcnResult<Identity> {
        if !Path::new(cert_path).exists() {
            return Err(IcnError::Network(format!("Certificate file not found: {}", cert_path)));
        }

        if !Path::new(key_path).exists() {
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

        Identity::from_pkcs8(&cert, &key)
            .map_err(|e| IcnError::Network(format!("Failed to load TLS identity: {}", e)))
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
    pub async fn start_server(&self, address: &str, identity: Identity) -> IcnResult<()> {
        let acceptor = TlsAcceptor::from(
            native_tls::TlsAcceptor::new(identity)
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
                        if let Err(e) = handle_client_connection(stream, acceptor, peers).await {
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

        self.peers.write().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?.push(tls_stream);
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
        let mut peers = self.peers.write().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
        for peer in peers.iter_mut() {
            peer.write_all(message.as_bytes()).await
                .map_err(|e| IcnError::Network(format!("Failed to send message: {}", e)))?;
        }
        Ok(())
    }

    /// Initializes the networking component. Currently, this function is a placeholder.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - An empty result indicating success or an error.
    pub async fn initialize(&self) -> IcnResult<()> {
        Ok(())
    }

    /// Stops the networking component and disconnects all peers.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - An empty result indicating success or an error if stopping fails.
    pub async fn stop(&self) -> IcnResult<()> {
        let mut peers = self.peers.write().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
        for peer in peers.iter_mut() {
            if let Err(e) = peer.shutdown().await {
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
///
/// # Returns
///
/// * `IcnResult<()>` - An empty result indicating success or an error if the connection fails.
async fn handle_client_connection(
    stream: TcpStream,
    acceptor: TlsAcceptor,
    peers: Arc<RwLock<Vec<tokio_native_tls::TlsStream<TcpStream>>>>
) -> IcnResult<()> {
    let tls_stream = acceptor.accept(stream).await
        .map_err(|e| IcnError::Network(format!("Failed to accept TLS connection: {:?}", e)))?;

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
    mut stream: tokio_native_tls::TlsStream<TcpStream>, 
    peers: Arc<RwLock<Vec<tokio_native_tls::TlsStream<TcpStream>>>>
) -> IcnResult<()> {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer).await {
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
    peers.retain(|p| !std::ptr::eq(p.get_ref(), stream.get_ref()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use tokio::net::TcpListener;

    #[test]
    /// Tests the creation of a new Networking instance.
    fn test_networking_creation() {
        let networking = Networking::new();
        assert_eq!(networking.peers.read().unwrap().len(), 0);
    }

    #[test]
    /// Tests the ability to connect to a peer.
    fn test_connect_to_peer() {
        let runtime = Runtime::new().unwrap();
        let networking = Networking::new();

        let listener = runtime.block_on(async {
            TcpListener::bind("127.0.0.1:0").await.unwrap()
        });
        let addr = listener.local_addr().unwrap();

        runtime.block_on(async {
            let networking_clone = networking.clone();
            tokio::spawn(async move {
                let (client, _) = listener.accept().await.unwrap();
                let tls_connector = tokio_native_tls::TlsConnector::from(
                    native_tls::TlsConnector::new().unwrap()
                );

                let tls_stream = tls_connector.connect("localhost", client).await.unwrap();
                networking_clone.peers.write().unwrap().push(tls_stream);
            });

            let result = networking.connect_to_peer(&addr.to_string()).await;
            assert!(result.is_ok());
        });
    }

    #[test]
    /// Tests the loading of a TLS identity.
    fn test_load_tls_identity() {
        let cert_file = tempfile::NamedTempFile::new().unwrap();
        let key_file = tempfile::NamedTempFile::new().unwrap();

        std::fs::write(cert_file.path(), b"dummy cert data").unwrap();
        std::fs::write(key_file.path(), b"dummy key data").unwrap();

        let result = Networking::load_tls_identity(
            cert_file.path().to_str().unwrap(),
            key_file.path().to_str().unwrap(),
            "test_password"
        );
        assert!(result.is_err());
    }

    #[test]
    /// Tests broadcasting a message to peers.
    fn test_broadcast_message() {
        let runtime = Runtime::new().unwrap();
        let networking = Networking::new();

        runtime.block_on(async {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            let networking_clone = networking.clone();
            tokio::spawn(async move {
                let (client, _) = listener.accept().await.unwrap();
                let tls_connector = tokio_native_tls::TlsConnector::from(
                    native_tls::TlsConnector::new().unwrap()
                );

                let tls_stream = tls_connector.connect("localhost", client).await.unwrap();
                networking_clone.peers.write().unwrap().push(tls_stream);
            });

            let result = networking.connect_to_peer(&addr.to_string()).await;
            assert!(result.is_ok());

            let result = networking.broadcast_message("Test message").await;
            assert!(result.is_ok());
        });
    }
}
