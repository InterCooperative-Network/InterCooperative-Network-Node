// icn_networking/src/lib.rs

use std::sync::{Arc, RwLock};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use log::{info, error};
use native_tls::{TlsAcceptor, TlsConnector, Identity};
use icn_shared::{IcnError, IcnResult};

/// Represents the networking component of the ICN node
pub struct Networking {
    /// A list of connected peers, protected by a RwLock for thread-safety
    peers: Arc<RwLock<Vec<native_tls::TlsStream<TcpStream>>>>,
}

impl Networking {
    /// Creates a new Networking instance
    pub fn new() -> Self {
        Networking {
            peers: Arc::new(RwLock::new(vec![])),
        }
    }

    /// Starts a TLS server to accept incoming connections from peers
    ///
    /// # Arguments
    ///
    /// * `address` - The address to bind the server to (e.g., "127.0.0.1:8080")
    /// * `identity` - The TLS identity for the server, containing the certificate and private key
    pub fn start_server(&self, address: &str, identity: Identity) -> IcnResult<()> {
        let acceptor: Arc<TlsAcceptor> = Arc::new(
            TlsAcceptor::new(identity)
                .map_err(|e| IcnError::Network(format!("Failed to create TLS acceptor: {}", e)))?,
        );

        let listener = TcpListener::bind(address)
            .map_err(|e| IcnError::Network(format!("Failed to bind to address: {}", e)))?;
        info!("Server started on {}", address);

        // Accept incoming connections in a loop
        for stream in listener.incoming() {
            let acceptor = Arc::clone(&acceptor);
            let peers = Arc::clone(&self.peers);

            thread::spawn(move || {
                match stream {
                    Ok(stream) => {
                        // Accept the TLS connection and handle the client in a separate thread
                        match acceptor.accept(stream) {
                            Ok(tls_stream) => handle_client(tls_stream, peers),
                            Err(e) => error!("Failed to accept TLS connection: {:?}", e),
                        }
                    }
                    Err(e) => error!("Failed to accept TCP connection: {:?}", e),
                }
            });
        }

        Ok(())
    }

    /// Establishes a TLS connection to a peer at the specified address
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the peer to connect to (e.g., "127.0.0.1:8080")
    pub fn connect_to_peer(&self, address: &str) -> IcnResult<()> {
        let connector = TlsConnector::new()
            .map_err(|e| IcnError::Network(format!("Failed to create TLS connector: {}", e)))?;

        let stream = TcpStream::connect(address)
            .map_err(|e| IcnError::Network(format!("Failed to connect to peer: {}", e)))?;

        let tls_stream = connector.connect(address, stream)
            .map_err(|e| IcnError::Network(format!("Failed to establish TLS connection: {}", e)))?;

        // Add the connected peer to the list
        self.peers.write().unwrap().push(tls_stream);
        info!("Connected to peer at {}", address);
        Ok(())
    }

    /// Broadcasts a message to all connected peers
    ///
    /// # Arguments
    ///
    /// * `message` - The message to broadcast
    pub fn broadcast_message(&self, message: &str) -> IcnResult<()> {
        let mut peers = self.peers.write().unwrap();
        for peer in peers.iter_mut() {
            peer.write_all(message.as_bytes())
                .map_err(|e| IcnError::Network(format!("Failed to send message: {}", e)))?;
        }
        Ok(())
    }

    /// Initializes the networking component
    pub fn initialize(&self) -> IcnResult<()> {
        // Initialization logic here (e.g., loading peer addresses from configuration)
        Ok(())
    }

    /// Stops the networking component, closing all connections
    pub fn stop(&self) -> IcnResult<()> {
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
fn handle_client(mut stream: native_tls::TlsStream<TcpStream>, peers: Arc<RwLock<Vec<native_tls::TlsStream<TcpStream>>>>) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
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
    let mut peers = peers.write().unwrap();
    peers.retain(|p| !std::ptr::eq(p.get_ref(), stream.get_ref()));
}
