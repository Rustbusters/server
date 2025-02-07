use crate::http::HttpServer;
use crate::utils::traits::Runnable;
use crate::websocket::WebSocketServer;
use crate::RustBustersServer;

use common_utils::HostCommand;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;
use std::{
    option,
    thread::{self, JoinHandle},
};
use wg_2024::config::Server;

// List of commands exchanged between the RustbustersServerController and the different services
pub(crate) enum InternalCommand {
    Stop,
}

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
    // HTTP Server Channels
    http_sender: Sender<InternalCommand>,
    http_receiver: Receiver<InternalCommand>,
    // WS Server Channels
    ws_sender: Sender<InternalCommand>,
    ws_receiver: Receiver<InternalCommand>,
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
        let (http_sender, http_receiver) = unbounded::<InternalCommand>();
        let (ws_sender, ws_receiver) = unbounded::<InternalCommand>();
        Self {
            http_server_address,
            http_public_path,
            ws_server_address,
            thread_handles: Vec::new(),
            receiver,
            http_sender,
            http_receiver,
            ws_sender,
            ws_receiver,
        }
    }

    fn start(mut self) {
        // Run http server with UI
        if let Some(http_handle) = Self::run_http(
            self.http_server_address,
            self.http_public_path,
            self.http_receiver,
        ) {
            self.thread_handles.push(http_handle);
        }
        // Run websocket server
        if let Some(ws_handle) = Self::run_ws_server(self.ws_server_address, self.ws_receiver) {
            self.thread_handles.push(ws_handle);
        }

        // Blocks untils a command is received: reduce CPU usage
        while let Ok(command) = self.receiver.recv() {
            // When the Stop command is received, terminate the process and the children services
            if let HostCommand::Stop = command {
                info!("[RB-CONTROLLER] Stop command received");
                self.http_sender.send(InternalCommand::Stop);
                self.ws_sender.send(InternalCommand::Stop);
                break;
            }
        }

        info!("[RB-CONTROLLER] Terminating services");
        for handle in self.thread_handles {
            let _ = handle.join(); // Safe join with error handling
        }
    }

    fn run_http(
        http_server_address: String,
        http_public_path: String,
        http_receiver: Receiver<InternalCommand>,
    ) -> Option<JoinHandle<()>> {
        let http_server: HttpServer =
            HttpServer::new(http_server_address, http_public_path, http_receiver);
        let handle = http_server.run();
        handle
    }

    fn run_ws_server(
        ws_server_address: String,
        ws_receiver: Receiver<InternalCommand>,
    ) -> Option<JoinHandle<()>> {
        let ws_server = WebSocketServer::new(ws_server_address, ws_receiver);
        let handle = ws_server.run();
        handle
    }
}
