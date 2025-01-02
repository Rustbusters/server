
use crate::view::http::HttpServer;
use crate::model::websocket::{self, WebSocketServer};
use crate::model::RustBustersServer;

use tokio_tungstenite::tungstenite::handshake::server;
use wg_2024::config::Server;
use std::thread;
use tokio::task::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use futures::{SinkExt, StreamExt};
use tokio::runtime::Runtime;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, connect_async};
use tokio_tungstenite::tungstenite::Message;

pub struct RustBustersServerController {
    // Network config
    ip: [u8; 4],
    port: u16,
    // HTTP
    pub(crate) http_server_address: String,
    // WebSocket (port = http_port + 1)
    pub(crate) websocket_server_address: String,
    // Path for storing static content
    pub(crate) public_path: String
}

impl RustBustersServerController {
    pub fn new(ip: [u8; 4], port: u16, public_path: &str) -> Self {
        let ip_str: String = ip.iter().map(|n| n.to_string()).collect::<Vec<String>>().join(".");
        let http_server_address = format!("{}:{}", ip_str, port);
        let websocket_server_address = format!("{}:{}", ip_str, port+1);

        Self { ip, port, http_server_address, websocket_server_address, public_path: public_path.to_string() }
    }

    pub fn launch(&self) {
        self.run_ui();
        self.run_websocket_server();
    }

    fn run_ui(&self) {
        let http_server = HttpServer::new(self.http_server_address.clone(), self.public_path.clone());
        http_server.run();
    }

    fn run_websocket_server(&self) {
        let websocket_server = WebSocketServer::new(self.websocket_server_address.clone());
        websocket_server.run(self.websocket_server_address.clone());
        thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create Tokio runtime");

            rt.block_on(async move {
                // Launch WebSocket server
                let server_task = task::spawn(async move {
                    if let Err(e) = websocket_server.run(websocket_server.address.clone()).await {
                        eprintln!("Error in WebSocket Server task: {}", e);
                    }
                });

                // Await server task
                if let Err(e) = server_task.await {
                    eprintln!("WebSocket server error: {}", e);
                }
            });
        });
    }
}