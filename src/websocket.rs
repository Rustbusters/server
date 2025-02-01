use crate::utils::traits::{Runnable, Service};
use crate::{InternalChannelsManager, StatsManager};

use log::{info, warn};
use rusqlite::Connection;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use url::Url;
use wg_2024::config::Server;
use wg_2024::network::NodeId;

use common_utils::Stats;
use std::collections::HashMap;
use uuid::Uuid;

use tungstenite::{Error, Message, WebSocket};

pub struct WebSocketServer {
    pub(crate) address: String,
}

impl Runnable for WebSocketServer {
    fn run(self) -> Option<JoinHandle<()>> {
        // Listens for incoming websocket connections
        thread::spawn(move || {
            self.start();
        });
        None
    }
}

impl Service for WebSocketServer {
    fn start(self) {
        if let Ok(listener) = TcpListener::bind(&self.address) {
            info!("[SERVER-WSS] Listening at ws://{}", &self.address);
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

                if InternalChannelsManager::is_empty() {
                    break;
                }
            }
        } else {
            eprintln!("Failed to bind WebSocket server");
        }
    }
}

impl WebSocketServer {
    pub fn new(address: String) -> Self {
        Self { address }
    }

    fn handle_connection(mut ws_stream: WebSocket<TcpStream>) {
        info!("[SERVER-WSS] Connection established");
        ws_stream.get_ref().set_nonblocking(true).unwrap();

        // Handle incoming messages from the client
        loop {
            InternalChannelsManager::receive_and_forward_message(&mut ws_stream);
            thread::sleep(Duration::from_millis(100));

            if let Err(Error::ConnectionClosed | Error::AlreadyClosed) = ws_stream.read() {
                break;
            }
        }
    }
}
