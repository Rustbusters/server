use crate::http::HttpServer;
use crate::utils::traits::Runnable;
use crate::websocket::WebSocketServer;
use crate::RustBustersServer;

use common_utils::HostCommand;
use crossbeam_channel::Receiver;
use std::{
    option,
    thread::{self, JoinHandle},
};
use wg_2024::config::Server;

pub struct RustBustersServerController {
    // HTTP server address
    http_server_address: String,
    // Path for storing static content
    http_public_path: String,
    // WebSocket server address
    ws_server_address: String,
    // Thread handles
    thread_handles: Vec<JoinHandle<()>>,
    // Crossbeam channel for many to 1 communication between network servers and the controller
    receiver: Receiver<HostCommand>,
}

impl Runnable for RustBustersServerController {
    fn run(self) -> Option<JoinHandle<()>> {
        let handle = thread::spawn(move || {
            self.start();
        });
        Some(handle)
    }
}

impl RustBustersServerController {
    pub fn new(
        http_server_address: String,
        http_public_path: String,
        ws_server_address: String,
        receiver: Receiver<HostCommand>,
    ) -> Self {
        Self {
            http_server_address,
            http_public_path,
            ws_server_address,
            thread_handles: Vec::new(),
            receiver,
        }
    }

    fn start(mut self) {
        // Run http server with UI
        if let Some(http_handle) = Self::run_http(self.http_server_address, self.http_public_path) {
            self.thread_handles.push(http_handle);
        }
        // Run websocket server
        if let Some(ws_handle) = Self::run_ws_server(self.ws_server_address) {
            self.thread_handles.push(ws_handle);
        }

        // Blocks untils a command is received: reduce CPU usage
        while let Ok(command) = self.receiver.recv() {
            // When the Stop command is received, terminate the process and the children services
            if let HostCommand::Stop = command {
                break;
            }
        }

        for handle in self.thread_handles {
            let _ = handle.join(); // Safe join with error handling
        }
    }

    fn run_http(http_server_address: String, http_public_path: String) -> Option<JoinHandle<()>> {
        let http_server: HttpServer = HttpServer::new(http_server_address, http_public_path);
        let handle = http_server.run();
        handle
    }

    fn run_ws_server(ws_server_address: String) -> Option<JoinHandle<()>> {
        let ws_server = WebSocketServer::new(ws_server_address);
        let handle = ws_server.run();
        handle
    }
}
