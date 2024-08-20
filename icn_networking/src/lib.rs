// File: icn_networking/src/lib.rs

use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_native_tls::{TlsAcceptor, TlsConnector};
use native_tls::Identity;
use icn_shared::{IcnError, IcnResult};
use log::{info, error};

#[derive(Clone)]
pub struct Networking {
    peers: Arc<Mutex<Vec<tokio_native_tls::TlsStream<TcpStream>>>>,
}

impl Networking {
    pub fn new() -> Self {
        Networking {
            peers: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn load_tls_identity(cert_path: &str, key_path: &str, password: &str) -> IcnResult<Identity> {
        let cert = std::fs::read(cert_path)
            .map_err(|e| IcnError::Network(format!("Failed to read certificate file: {}", e)))?;
        let _key = std::fs::read(key_path)
            .map_err(|e| IcnError::Network(format!("Failed to read key file: {}", e)))?;
        Identity::from_pkcs12(&cert, password)
            .map_err(|e| IcnError::Network(format!("Failed to load TLS identity: {}", e)))
    }

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

    pub async fn connect_to_peer(&self, address: &str) -> IcnResult<()> {
        let connector = TlsConnector::from(
            native_tls::TlsConnector::new()
                .map_err(|e| IcnError::Network(format!("Failed to create TLS connector: {}", e)))?
        );

        let stream = TcpStream::connect(address).await
            .map_err(|e| IcnError::Network(format!("Failed to connect to peer: {}", e)))?;

        let tls_stream = connector.connect(address, stream).await
            .map_err(|e| IcnError::Network(format!("Failed to establish TLS connection: {}", e)))?;

        self.peers.lock().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?.push(tls_stream);
        info!("Connected to peer at {}", address);
        Ok(())
    }

    pub async fn broadcast_message(&self, message: &str) -> IcnResult<()> {
        let mut peers = self.peers.lock().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
        for peer in peers.iter_mut() {
            peer.write_all(message.as_bytes()).await
                .map_err(|e| IcnError::Network(format!("Failed to send message: {}", e)))?;
        }
        Ok(())
    }

    pub async fn initialize(&self) -> IcnResult<()> {
        Ok(())
    }

    pub async fn stop(&self) -> IcnResult<()> {
        let mut peers = self.peers.lock().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
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

async fn handle_client_connection(
    stream: TcpStream,
    acceptor: TlsAcceptor,
    peers: Arc<Mutex<Vec<tokio_native_tls::TlsStream<TcpStream>>>>
) -> IcnResult<()> {
    let tls_stream = acceptor.accept(stream).await
        .map_err(|e| IcnError::Network(format!("Failed to accept TLS connection: {:?}", e)))?;

    handle_client(tls_stream, peers).await
}

async fn handle_client(mut stream: tokio_native_tls::TlsStream<TcpStream>, peers: Arc<Mutex<Vec<tokio_native_tls::TlsStream<TcpStream>>>>) -> IcnResult<()> {
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

    let mut peers = peers.lock().map_err(|_| IcnError::Network("Failed to acquire peers lock".to_string()))?;
    peers.retain(|p| !std::ptr::eq(p.get_ref(), stream.get_ref()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use tokio::net::TcpListener;

    #[test]
    fn test_networking_creation() {
        let networking = Networking::new();
        assert_eq!(networking.peers.lock().unwrap().len(), 0);
    }

    #[test]
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
                networking_clone.peers.lock().unwrap().push(tls_stream);
            });

            let result = networking.connect_to_peer(&addr.to_string()).await;
            assert!(result.is_ok());
        });
    }

    #[test]
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
                networking_clone.peers.lock().unwrap().push(tls_stream);
            });

            let result = networking.connect_to_peer(&addr.to_string()).await;
            assert!(result.is_ok());

            let result = networking.broadcast_message("Test message").await;
            assert!(result.is_ok());
        });
    }
}