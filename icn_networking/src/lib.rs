// icn_networking/src/lib.rs

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, Error};
use std::sync::{Arc, Mutex};
use std::thread;
use log::{info, error};
use native_tls::{TlsAcceptor, TlsStream, Identity};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// The Networking struct manages peer-to-peer communication over TLS.
/// It maintains a list of connected peers and handles incoming and outgoing messages.
pub struct Networking {
    _peers: Arc<Mutex<Vec<TlsStream<TcpStream>>>>, // Shared list of connected peers
}

impl Networking {
    /// Creates a new Networking instance with an empty peer list.
    pub fn new() -> Self {
        Networking {
            _peers: Arc::<Mutex<Vec<TlsStream<TcpStream>>>>::new(Mutex::new(vec![])),
        }
    }

    /// Starts a TLS server at the specified address and listens for incoming connections.
    /// Returns an error if the server fails to start.
    pub fn start_server(&self, address: &str) -> Result<(), Error> {
        let identity = load_identity()?; // Load the server's TLS identity (certificate and private key)
        let acceptor = TlsAcceptor::new(identity)?;

        let listener = TcpListener::bind(address)?;
        info!("Server started on {}", address);

        // Handle incoming connections
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let peers = Arc::clone(&self._peers);
                    let acceptor = acceptor.clone();
                    thread::spawn(move || {
                        match acceptor.accept(stream) {
                            Ok(tls_stream) => handle_client(tls_stream, peers), // Handle client connection
                            Err(e) => error!("Failed to accept client: {:?}", e),
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept client: {:?}", e);
                }
            }
        }

        Ok(())
    }

    /// Connects to a peer at the specified address using TLS.
    /// Returns an error if the connection fails.
    pub fn connect_to_peer(&self, address: &str) -> Result<(), Error> {
        let connector = native_tls::TlsConnector::new()?;
        let stream = TcpStream::connect(address)?;
        let tls_stream = connector.connect(address, stream)?;
        self._peers.lock().unwrap().push(tls_stream); // Add the peer to the list
        info!("Connected to peer at {}", address);
        Ok(())
    }

    /// Broadcasts a message to all connected peers.
    /// Logs errors if sending the message fails.
    pub fn broadcast_message(&self, message: &str) -> Result<(), Error> {
        let mut peers = self._peers.lock().unwrap();
        for peer in peers.iter_mut() {
            if let Err(e) = peer.write_all(message.as_bytes()) {
                error!("Failed to send message to peer: {:?}", e);
            }
        }
        Ok(())
    }
}

/// Handles communication with a connected client.
/// Continuously reads messages from the client and logs them.
fn handle_client(mut stream: TlsStream<TcpStream>, _peers: Arc<Mutex<Vec<TlsStream<TcpStream>>>>) {
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(_) => {
                let message = String::from_utf8_lossy(&buffer[..]);
                info!("Received message: {}", message);
            }
            Err(e) => {
                error!("Error reading from stream: {:?}", e);
                break;
            }
        }
    }
}

/// Loads the TLS identity (certificate and private key) from a PKCS#12 file.
/// Returns an `Identity` object or an error if loading fails.
fn load_identity() -> Result<Identity, Error> {
    let cert_path = Path::new("path/to/cert.pfx"); // Path to the PKCS#12 file
    let cert_file = File::open(&cert_path)?;
    let mut cert_reader = BufReader::new(cert_file);
    let identity = Identity::from_pkcs12_der(&mut cert_reader, "password")?;
    Ok(identity)
}
