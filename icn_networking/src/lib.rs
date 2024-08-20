use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, Error};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Networking {
    peers: Arc<Mutex<Vec<TcpStream>>>,
}

impl Networking {
    pub fn new() -> Self {
        Networking {
            peers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start_server(&self, address: &str) -> Result<(), Error> {
        let listener = TcpListener::bind(address)?;
        println!("Server started on {}", address);

        for stream in listener.incoming() {
            let stream = stream?;
            let peers = Arc::clone(&self.peers);

            // Add the new connection to the list of peers
            peers.lock().unwrap().push(stream.try_clone().unwrap());

            // Handle the new connection in a separate thread
            thread::spawn(move || {
                if let Err(e) = handle_client(stream, peers) {
                    eprintln!("Error handling client: {:?}", e);
                }
            });
        }

        Ok(())
    }

    pub fn connect_to_peer(&self, address: &str) -> Result<(), Error> {
        match TcpStream::connect(address) {
            Ok(stream) => {
                let mut peers = self.peers.lock().unwrap();
                peers.push(stream.try_clone().unwrap());
                println!("Connected to peer at {}", address);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to connect to peer at {}: {:?}", address, e);
                Err(e)
            }
        }
    }

    pub fn broadcast_message(&self, message: &str) -> Result<(), Error> {
        let peers = self.peers.lock().unwrap();
        for peer in peers.iter() {
            let mut stream = peer.try_clone().unwrap();
            stream.write_all(message.as_bytes())?;
        }
        Ok(())
    }

    pub fn discover_peers(&self, start_port: u16, end_port: u16) {
        for port in start_port..=end_port {
            let address = format!("127.0.0.1:{}", port);
            if let Err(e) = self.connect_to_peer(&address) {
                eprintln!("Error discovering peer on port {}: {:?}", port, e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, peers: Arc<Mutex<Vec<TcpStream>>>) -> Result<(), Error> {
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                // Connection was closed
                break;
            }
            Ok(_) => {
                let message = String::from_utf8_lossy(&buffer[..]);
                println!("Received message: {}", message);

                // Broadcast the message to all peers
                let peers = peers.lock().unwrap();
                for peer in peers.iter() {
                    if peer.peer_addr().unwrap() != stream.peer_addr().unwrap() {
                        let mut peer_stream = peer.try_clone().unwrap();
                        peer_stream.write_all(message.as_bytes())?;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from stream: {:?}", e);
                break;
            }
        }
    }
    Ok(())
}
