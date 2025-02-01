use crate::http::HttpServer;
use crate::utils::traits::Runnable;
use crate::websocket::WebSocketServer;
use crate::RustBustersServer;

use std::thread::{self, JoinHandle};
use wg_2024::config::Server;

pub struct RustBustersServerController {
    // HTTP server address
    http_server_address: String,
    // Path for storing static content
    http_public_path: String,
    // WebSocket server address
    ws_server_address: String,
}

impl Runnable for RustBustersServerController {
    fn run(self) -> Option<JoinHandle<()>> {
        self.start();
        None
    }
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

    fn start(self) {
        // Run the HTTP server UI
        Self::run_ui(self.http_server_address, self.http_public_path);
        // Run the WS server
        Self::run_websocket_server(self.ws_server_address);
    }

    fn run_ui(http_server_address: String, http_public_path: String) {
        let http_server = HttpServer::new(http_server_address, http_public_path);
        http_server.run();
    }

    fn run_websocket_server(ws_server_address: String) {
        let websocket_server = WebSocketServer::new(ws_server_address);
        websocket_server.run();
    }
}
