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
            if let Err(e) = peer.write_all(message.as_bytes()).await {
                error!("Failed to send message: {:?}", e);
            }
        }
        Ok(())
    }

    pub fn stop_server(&self) -> IcnResult<()> {
        // Add your server stopping logic here
        Ok(())
    }
}

async fn handle_client(mut tls_stream: TlsStream<TcpStream>, peers: Arc<Mutex<Vec<TlsStream<TcpStream>>>>) {
    let mut buf = vec![0; 1024];
    loop {
        let n = match tls_stream.read(&mut buf).await {
            Ok(n) if n == 0 => break, // Connection closed
            Ok(n) => n,
            Err(e) => {
                error!("Failed to read from client: {:?}", e);
                break;
            }
        };

        let message = String::from_utf8_lossy(&buf[..n]);
        info!("Received message: {}", message);

        // Broadcast message to all peers
        let mut peers = peers.lock().await;
        for peer in peers.iter_mut() {
            if let Err(e) = peer.write_all(message.as_bytes()).await {
                error!("Failed to send message to peer: {:?}", e);
            }
        }
    }
}
