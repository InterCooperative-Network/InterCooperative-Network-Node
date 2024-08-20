// icn_networking/src/lib.rs

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use log::{info, error};
use native_tls::{TlsAcceptor, TlsConnector, Identity};
use tokio_native_tls::{TlsAcceptor as TokioTlsAcceptor, TlsStream};
use icn_shared::{IcnError, IcnResult};

pub struct Networking {
    peers: Arc<Mutex<Vec<TlsStream<TcpStream>>>>,
}

impl Networking {
    pub fn new() -> Self {
        Networking {
            peers: Arc::new(Mutex::new(vec![])),
        }
    }

    pub async fn start_server(&self, address: &str, identity: Identity) -> IcnResult<()> {
        let acceptor = TlsAcceptor::new(identity)
            .map_err(|e| IcnError::Network(format!("Failed to create TLS acceptor: {}", e)))?;
        let acceptor = TokioTlsAcceptor::from(acceptor);

        let listener = TcpListener::bind(address).await
            .map_err(|e| IcnError::Network(format!("Failed to bind to address: {}", e)))?;
        info!("Server started on {}", address);

        loop {
            let (stream, _) = listener.accept().await
                .map_err(|e| IcnError::Network(format!("Failed to accept connection: {}", e)))?;
            let acceptor = acceptor.clone();
            let peers = Arc::clone(&self.peers);

            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => handle_client(tls_stream, peers).await,
                    Err(e) => error!("Failed to accept TLS connection: {:?}", e),
                }
            });
        }
    }

    pub async fn connect_to_peer(&self, address: &str) -> IcnResult<()> {
        let connector = TlsConnector::new()
            .map_err(|e| IcnError::Network(format!("Failed to create TLS connector: {}", e)))?;
        let connector = tokio_native_tls::TlsConnector::from(connector);

        let stream = TcpStream::connect(address).await
            .map_err(|e| IcnError::Network(format!("Failed to connect to peer: {}", e)))?;
        let tls_stream = connector.connect(address, stream).await
            .map_err(|e| IcnError::Network(format!("Failed to establish TLS connection: {}", e)))?;

        self.peers.lock().await.push(tls_stream);
        info!("Connected to peer at {}", address);
        Ok(())
    }

    pub async fn broadcast_message(&self, message: &str) -> IcnResult<()> {
        let mut peers = self.peers.lock().await;
        for peer in peers.iter_mut() {
            peer.write_all(message.as_bytes()).await
                .map_err(|e| IcnError::Network(format!("Failed to send message: {}", e)))?;
        }
        Ok(())
    }

    pub async fn initialize(&self) -> IcnResult<()> {
        // Initialization logic here
        Ok(())
    }

    pub async fn stop(&self) -> IcnResult<()> {
        // Stop logic here
        Ok(())
    }
}

async fn handle_client(mut stream: TlsStream<TcpStream>, peers: Arc<Mutex<Vec<TlsStream<TcpStream>>>>) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]);
                info!("Received message: {}", message);
                // Process the message here
            }
            Err(e) => {
                error!("Error reading from stream: {:?}", e);
                break;
            }
        }
    }
    let mut peers = peers.lock().await;
    peers.retain(|p| !std::ptr::eq(p.get_ref(), stream.get_ref()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_networking_creation() {
        let networking = Networking::new();
        assert!(networking.peers.lock().await.is_empty());
    }

    #[tokio::test]
    async fn test_connect_to_peer() {
        let networking = Networking::new();
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        tokio::spawn(async move {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            tx.send(addr).unwrap();
            let (stream, _) = listener.accept().await.unwrap();
            let identity = Identity::from_pkcs12(include_bytes!("test_cert.p12"), "password").unwrap();
            let acceptor = TlsAcceptor::new(identity).unwrap();
            let _tls_stream = acceptor.accept(stream).await.unwrap();
        });

        let addr: SocketAddr = rx.await.unwrap();
        let result = networking.connect_to_peer(&addr.to_string()).await;
        assert!(result.is_ok());
        assert_eq!(networking.peers.lock().await.len(), 1);
    }
}