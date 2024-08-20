// File: icn_networking/src/lib.rs

use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_native_tls::TlsAcceptor;
use native_tls::{Identity, TlsAcceptor as NativeTlsAcceptor};
use icn_shared::{IcnError, IcnResult};
use log::{info, error};

/// Represents the networking component of the ICN node
pub struct Networking {
    /// A list of connected peers, protected by a Mutex for thread-safety
    peers: Arc<Mutex<Vec<tokio_native_tls::TlsStream<TcpStream>>>>,
}

impl Networking {
    /// Creates a new Networking instance
    pub fn new() -> Self {
        Networking {
            peers: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Starts a TLS server to accept incoming connections from peers
    ///
    /// # Arguments
    ///
    /// * `address` - The address to bind the server to (e.g., "127.0.0.1:8080")
    /// * `identity` - The TLS identity for the server, containing the certificate and private key
    pub async fn start_server(&self, address: &str, identity: Identity) -> IcnResult<()> {
        let acceptor = TlsAcceptor::from(
            NativeTlsAcceptor::new(identity)
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
                        match acceptor.accept(stream).await {
                            Ok(tls_stream) => {
                                if let Err(e) = handle_client(tls_stream, peers).await {
                                    error!("Error handling client: {:?}", e);
                                }
                            }
                            Err(e) => error!("Failed to accept TLS connection: {:?}", e),
                        }
                    });
                }
                Err(e) => error!("Failed to accept TCP connection: {:?}", e),
            }
        }
    }

    /// Establishes a TLS connection to a peer at the specified address
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the peer to connect to (e.g., "127.0.0.1:8080")
    pub async fn connect_to_peer(&self, address: &str) -> IcnResult<()> {
        let connector = tokio_native_tls::TlsConnector::from(
            native_tls::TlsConnector::new()
                .map_err(|e| IcnError::Network(format!("Failed to create TLS connector: {}", e)))?
        );

        let stream = TcpStream::connect(address).await
            .map_err(|e| IcnError::Network(format!("Failed to connect to peer: {}", e)))?;

        let tls_stream = connector.connect(address, stream).await
            .map_err(|e| IcnError::Network(format!("Failed to establish TLS connection: {}", e)))?;

        // Add the connected peer to the list
        self.peers.lock().unwrap().push(tls_stream);
        info!("Connected to peer at {}", address);
        Ok(())
    }

    /// Broadcasts a message to all connected peers
    ///
    /// # Arguments
    ///
    /// * `message` - The message to broadcast
    pub async fn broadcast_message(&self, message: &str) -> IcnResult<()> {
        let mut peers = self.peers.lock().unwrap();
        for peer in peers.iter_mut() {
            peer.write_all(message.as_bytes()).await
                .map_err(|e| IcnError::Network(format!("Failed to send message: {}", e)))?;
        }
        Ok(())
    }

    /// Initializes the networking component
    pub async fn initialize(&self) -> IcnResult<()> {
        // Initialization logic here (e.g., loading peer addresses from configuration)
        Ok(())
    }

    /// Stops the networking component, closing all connections
    pub async fn stop(&self) -> IcnResult<()> {
        // Stop logic here (e.g., gracefully closing connections to peers)
        Ok(())
    }
}

/// Handles incoming client connections, processes received messages, and manages peer disconnections
///
/// # Arguments
///
/// * `stream` - The TLS stream representing the connection to the client
/// * `peers` - A shared list of connected peers
async fn handle_client(mut stream: tokio_native_tls::TlsStream<TcpStream>, peers: Arc<Mutex<Vec<tokio_native_tls::TlsStream<TcpStream>>>>) -> IcnResult<()> {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => {
                // Connection closed gracefully
                break;
            }
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]);
                info!("Received message: {}", message);

                // TODO: Process the message here
                // This could involve:
                // - Parsing the message based on a defined protocol
                // - Routing the message to the appropriate module (e.g., consensus, blockchain)
                // - Taking corresponding actions based on the message content
            }
            Err(e) => {
                error!("Error reading from stream: {:?}", e);
                break;
            }
        }
    }

    // Remove the disconnected peer from the list
    let mut peers = peers.lock().unwrap();
    peers.retain(|p| !std::ptr::eq(p.get_ref(), stream.get_ref()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_networking_creation() {
        let networking = Networking::new();
        assert_eq!(networking.peers.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_connect_to_peer() {
        let runtime = Runtime::new().unwrap();
        let networking = Networking::new();

        // This test requires a running TLS server to connect to
        // For a real test, you'd need to set up a mock TLS server
        // Here, we'll just test that the method doesn't panic
        runtime.block_on(async {
            let result = networking.connect_to_peer("localhost:8080").await;
            assert!(result.is_err()); // Expect an error since we're not actually connecting
        });
    }

    // Add more tests for other methods
}