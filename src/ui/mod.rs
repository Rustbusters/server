mod websocket;
mod http;

use wg_2024::config::Server;

use websocket::{WebSocketClient, WebSocketController, WebSocketServer};
use http::HttpServer;

pub struct RustBustersServerUI {
    ip_addr: String,
    port: u16,
    network_servers: Vec<Server>,
    websocket_server_address: String,
    http_server_address: String,
}

impl RustBustersServerUI {
    pub fn new(ip_addr: &str, port: u16, network_servers: Vec<Server>) -> Self {
        let ip_addr = ip_addr.to_string();
        let websocket_server_address = String::from(format!("{}:{}", ip_addr.clone(), port));
        let http_server_address = String::from(format!("{}:{}", ip_addr.clone(), port + 1));

        Self { ip_addr, port, network_servers, websocket_server_address, http_server_address }
    }
    
    pub fn run_ui(self) {
        // 1. Create WebSocketController that will spawn the server + clients
        let websocket_controller = WebSocketController::new(self.websocket_server_address, self.network_servers.clone());
        websocket_controller.run();

        // 2. Create and launch HTTP Server for hosting UI
        let http_server = HttpServer::new(self.http_server_address.clone(), "static/server/emeliyanov");
        http_server.run();
    }
}
