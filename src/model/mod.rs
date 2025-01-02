mod handlers;
mod network;
mod ad;
pub mod commands;
pub mod stats;
pub mod websocket;

use crate::model::stats::Stats;
use crate::RustBustersServerController;
use common_utils::{HostCommand, HostEvent};
use crossbeam_channel::{Receiver, Sender};
use log::{error, info};
use tokio::task;
use std::collections::{HashMap};
use wg_2024::network::NodeId;
use wg_2024::packet::{Fragment, NodeType, Packet};
use rand::*;
use std::thread;
use tokio_tungstenite::tungstenite::handshake::server;
use wg_2024::config::Server;
use tokio::task::{JoinHandle};
use std::sync::{Arc, Mutex};
use futures::{SinkExt, StreamExt};
use tokio::runtime::Runtime;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, connect_async};
use tokio_tungstenite::tungstenite::Message;

use websocket::WebSocketClient;

pub struct RustBustersServer {
    id: NodeId,
    controller_send: Sender<HostEvent>,
    controller_recv: Receiver<HostCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    known_nodes: HashMap<NodeId, NodeType>,
    topology: HashMap<NodeId, Vec<NodeId>>,
    flood_id_counter: u64,
    session_id_counter: u64,
    // (session_id, fragment_index) -> packet
    pending_sent: HashMap<(u64, u64), Packet>,
    // session_id -> (fragments, num_fragments) (u8 is the number of fragments received) (for reassembly)
    pending_received: HashMap<u64, (Vec<Option<Fragment>>, u64)>,
    stats: Stats,
    websocket_client: Option<WebSocketClient>,
    websocket_server_address: String
}

impl RustBustersServer {
    pub fn new(
        id: NodeId,
        controller_send: Sender<HostEvent>,
        controller_recv: Receiver<HostCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        websocket_server_address: String
    ) -> Self {
        info!("Server {} spawned succesfully", id);
        Self {
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            known_nodes: HashMap::new(),
            topology: HashMap::new(),
            flood_id_counter: rng().random_range(1000..=2000),
            session_id_counter: rng().random_range(100..=200),
            pending_sent: HashMap::new(),
            pending_received: HashMap::new(),
            stats: Stats::default(),
            websocket_client: None,
            websocket_server_address,
        }
    }

    pub fn run(&self) {
        // Start network discovery
        info!("RustBustersServer {} initiated network discovery", self.id);
        // self.discover_network();
        
        self.connect_to_websocket_server();
        
        // Listen for incoming messages
        loop {

        }
    }

    pub fn connect_to_websocket_server(&self) {
        let websocket_client = WebSocketClient::new(self.websocket_server_address.clone());
        let client_id = self.id;
        thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create Tokio runtime");

            rt.block_on(async move {
                // Launch WebSocket server
                let server_task = task::spawn(async move {
                    if let Err(e) = websocket_client.connect_to_websocket_server(client_id).await {
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
    
    pub(crate) fn send_to_sc(&mut self, event: HostEvent) {
        if self.controller_send.send(event).is_ok() {
            info!(
                "Server {} - Sent NodeEvent to SC",
                self.id
            );
        } else {
            error!(
                "Server {} - Error in sending event to SC",
                self.id
            );
        }
    }
}

