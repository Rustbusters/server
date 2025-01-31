use crate::http::HttpServer;
use crate::websocket::WebSocketServer;
use crate::RustBustersServer;

use futures::{SinkExt, StreamExt};
use std::thread;
use wg_2024::config::Server;

pub struct RustBustersServerController {
    // HTTP server address
    pub(crate) http_server_address: String,
    // Path for storing static content
    pub(crate) http_public_path: String,
    // WebSocket server address
    pub(crate) ws_server_address: String,
}

impl RustBustersServerController {
    pub fn new(
        http_server_address: String,
        http_public_path: String,
        ws_server_address: String,
    ) -> Self {
        Self {
            http_server_address,
            http_public_path,
            ws_server_address,
        }
    }

    pub fn launch(&self) {
        self.run_ui();
        self.run_websocket_server();
    }

    fn run_ui(&self) {
        let http_server = HttpServer::new(
            self.http_server_address.clone(),
            self.http_public_path.clone(),
        );
        http_server.run();
    }

    fn run_websocket_server(&self) {
        let websocket_server = WebSocketServer::new(self.ws_server_address.clone());
        websocket_server.run();
    }
}
