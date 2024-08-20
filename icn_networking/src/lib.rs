use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, Error};
use std::sync::{Arc, Mutex};
use std::thread;
use log::{info, error};

pub struct Networking {
    peers: Arc<Mutex<Vec<TcpStream>>>,
}

impl Networking {
    pub fn new() -> Self {
        Networking {
            peers: Arc<Mutex<Vec<TcpStream>>>::new(Mutex::new(vec![])),
        }
    }

    pub fn start_server(&self, address: &str) -> Result<(), Error> {
        let listener = TcpListener::bind(address)?;
        info!("Server started on {}", address);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let peers = Arc::clone(&self.peers);
                    thread::spawn(move || {
                        handle_client(stream, peers);
                    });
                }
                Err(e) => {
                    error!("Failed to accept client: {:?}", e);
                }
            }
        }

        Ok(())
    }

    pub fn connect_to_peer(&self, address: &str) -> Result<(), Error> {
        match TcpStream::connect(address) {
            Ok(stream) => {
                self.peers.lock().unwrap().push(stream);
                info!("Connected to peer at {}", address);
            }
            Err(e) => {
                error!("Failed to connect to peer: {:?}", e);
            }
        }
        Ok(())
    }

    pub fn broadcast_message(&self, message: &str) -> Result<(), Error> {
        let mut peers = self.peers.lock().unwrap();
        for peer in peers.iter_mut() {
            if let Err(e) = peer.write_all(message.as_bytes()) {
                error!("Failed to send message to peer: {:?}", e);
            }
        }
        Ok(())
    }
}

fn handle_client(mut stream: TcpStream, peers: Arc<Mutex<Vec<TcpStream>>>) {
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
