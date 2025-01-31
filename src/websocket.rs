use crate::{ConnectionsWrapper, StatsWrapper};

use rusqlite::Connection;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use url::Url;
use wg_2024::config::Server;
use wg_2024::network::NodeId;

use common_utils::Stats;
use std::collections::HashMap;
use uuid::Uuid;

use tungstenite::{Message, WebSocket};

pub struct WebSocketServer {
    pub(crate) address: String,
}

impl WebSocketServer {
    pub fn new(address: String) -> Self {
        Self { address }
    }

    pub fn run(self) {
        // Listens for incoming websocket connections
        thread::spawn(move || {
            self.listen(self.address.clone());
        });
    }

    pub fn listen(&self, address: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(listener) = TcpListener::bind(&address) {
            println!("[SERVER-WS] Server running at ws://{}", address);
            listener
                .set_nonblocking(true)
                .expect("Cannot set non-blocking");

            loop {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if let Ok(ws_stream) = tungstenite::accept(stream) {
                            thread::spawn(move || Self::handle_connection(ws_stream));
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No incoming connections, sleep briefly to avoid busy-waiting
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                        break;
                    }
                }

                if ConnectionsWrapper::is_empty() {
                    break;
                }
            }

            Ok(())
        } else {
            Err("Failed to bind WebSocket server")?
        }
    }

    fn handle_connection(mut ws_stream: WebSocket<TcpStream>) {
        println!("[SERVER-WS] Connection established");
        // Handle incoming messages from the client
        loop {
            ConnectionsWrapper::receive_and_forward_message(&mut ws_stream);
            thread::sleep(Duration::from_millis(100));
        }
    }
}
