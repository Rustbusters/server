use crate::stats::Stats;
use crate::RustBustersServerController;
use common_utils::{HostCommand, HostEvent};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use log::{error, info, debug, warn};
use tokio::task;
use std::collections::{HashMap};
use std::hash::Hash;
use wg_2024::network::NodeId;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};
use wg_2024::network::SourceRoutingHeader;
use rand::*;
use std::thread::{self, sleep};
use tokio_tungstenite::tungstenite::handshake::server;
use wg_2024::config::Server;
use tokio::task::{JoinHandle};
use std::sync::{Arc, Mutex};
use futures::{SinkExt, StreamExt};
use tokio::runtime::Runtime;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, connect_async};
use tokio_tungstenite::tungstenite::Message;
use crossbeam_channel::{unbounded, select};

use crate::websocket::client::WebSocketClient;
use crate::websocket::message::{InternalMessage, WebSocketMessage};
use common_utils::{User, HostMessage, ServerToClientMessage};

use tokio::time::Duration;
use std::collections::HashSet;

pub struct RustBustersServer {
    pub(crate) id: NodeId,
    pub(crate) controller_send: Sender<HostEvent>,
    pub(crate) controller_recv: Receiver<HostCommand>,
    pub(crate) packet_recv: Receiver<Packet>,
    pub(crate) packet_send: HashMap<NodeId, Sender<Packet>>,
    pub(crate) known_nodes: HashMap<NodeId, NodeType>,
    pub(crate) topology: HashMap<NodeId, Vec<NodeId>>,
    pub(crate) flood_ids: HashMap<NodeId, u64>,
    pub(crate) session_ids: HashMap<NodeId, u64>,
    pub(crate) flood_id_counter: u64,
    pub(crate) session_id_counter: u64,
    // (session_id, fragment_index) -> packet
    pub(crate) pending_sent: HashMap<(u64, u64), Packet>,
    // session_id -> (fragments, num_fragments) (u8 is the number of fragments received) (for reassembly)
    pub(crate) pending_received: HashMap<u64, (Vec<Option<Fragment>>, u64)>,
    pub(crate) stats: Stats,
    pub(crate) websocket_client: Option<WebSocketClient>,
    websocket_server_address: String,
    
    // Map for storing the active user sessions
    pub(crate) active_users: HashMap<NodeId, String>,
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
            flood_ids: HashMap::new(),
            session_ids: HashMap::new(),
            flood_id_counter: 0,
            session_id_counter: 0,
            pending_sent: HashMap::new(),
            pending_received: HashMap::new(),
            stats: Stats::default(),
            websocket_client: None,
            websocket_server_address,
            active_users: HashMap::new(),
        }
    }

    pub fn launch(&mut self) {
        // Start network discovery
        info!("RustBustersServer {} initiated network discovery", self.id);
        // self.discover_network();

        // Create crossbeam channels for packet listener and websocket full-duplex communication
        let (tx, rx) = unbounded::<InternalMessage>();
        
        // Start websocket client
        Self::launch_websocket_client(self.id, self.websocket_server_address.clone(), tx.clone(), rx.clone());
        
        // Start network listener
        self.launch_network_listener(tx, rx);
    }

    pub fn launch_websocket_client(client_id: NodeId, ws_url: String, tx: Sender<InternalMessage>, rx: Receiver<InternalMessage>) {
        thread::spawn(move || {
            let client_id = client_id;
            let rt = Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async {
                if let Err(e) = WebSocketClient::run(ws_url, tx, rx).await {
                    eprintln!("WebSocket client error: {}", e);
                }
            });
        });
    }

    pub fn launch_network_listener(&mut self, tx: Sender<InternalMessage>, rx: Receiver<InternalMessage>) {
        // Listen for incoming messages
        select! {
            // Handle network packets
            recv(self.packet_recv) -> packet_res => {
                if let Ok(mut packet) = packet_res {
                    self.handle_packet(packet);
                    // tx.send(InternalMessage::FragmentReceived(0)).unwrap();
                    // TODO: Send stats
                } else {
                    error!("Client {} - Error in receiving packet", self.id);
                }
            },

            // Handle Simulation Controller commands
            recv(self.controller_recv) -> command => {
                if let Ok(cmd) = command {
                    self.handle_command(cmd);
                } else {
                    error!("Client {} - Error in receiving command", self.id);
                }
            },

            // No more packets
            default(Duration::from_millis(2000)) => {}
        }
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

