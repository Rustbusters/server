use crate::db::{self, DbManager};
use crate::RustBustersServerController;
use common_utils::{HostCommand, HostEvent};
use crossbeam_channel::{select_biased, unbounded, Receiver, RecvTimeoutError, Sender};
use futures::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use rand::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};
use wg_2024::config::Server;
use wg_2024::network::NodeId;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};

use crate::websocket::message::{InternalMessage, WebSocketMessage};
use common_utils::{HostMessage, ServerToClientMessage, Stats, User};

use std::collections::HashSet;

pub struct RustBustersServer {
    pub(crate) id: NodeId,
    pub(crate) controller_send: Sender<HostEvent>,
    pub(crate) controller_recv: Receiver<HostCommand>,
    pub(crate) packet_recv: Receiver<Packet>,
    pub(crate) packet_send: HashMap<NodeId, Sender<Packet>>,
    pub(crate) known_nodes: HashMap<NodeId, NodeType>,
    pub(crate) topology: HashMap<NodeId, Vec<NodeId>>,
    pub(crate) session_ids: HashMap<NodeId, u64>,
    pub(crate) flood_id_counter: u64,
    pub(crate) session_id_counter: u64,
    // (session_id, fragment_index) -> packet
    pub(crate) pending_sent: HashMap<(u64, u64), Packet>,
    // session_id -> (fragments, num_fragments) (u8 is the number of fragments received) (for reassembly)
    pub(crate) pending_received: HashMap<u64, (Vec<Option<Fragment>>, u64)>,
    pub(crate) stats: Stats,
    websocket_server_address: String,

    // Map for storing the active user sessions
    pub(crate) active_users: HashMap<NodeId, String>,

    // Network discovery
    last_discovery: Instant,
    discovery_interval: Duration,

    // Database manager
    pub(crate) db_manager: Result<DbManager, rusqlite::Error>,
}

impl RustBustersServer {
    pub fn new(
        id: NodeId,
        controller_send: Sender<HostEvent>,
        controller_recv: Receiver<HostCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        websocket_server_address: String,
        discovery_interval: Option<Duration>,
    ) -> Self {
        let discovery_interval = discovery_interval.unwrap_or(Duration::from_secs(30));
        let db_name = format!("server_{}.db", id);

        info!("Server {} spawned succesfully", id);
        Self {
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            known_nodes: HashMap::new(),
            topology: HashMap::new(),
            session_ids: HashMap::new(),
            flood_id_counter: 0,
            session_id_counter: 0,
            pending_sent: HashMap::new(),
            pending_received: HashMap::new(),
            stats: Stats::default(),
            websocket_server_address,
            active_users: HashMap::new(),
            last_discovery: Instant::now(),
            discovery_interval,
            db_manager: DbManager::new(db_name),
        }
    }

    pub fn launch(&mut self) {
        // Start network discovery
        info!("RustBustersServer {} initiated network discovery", self.id);

        // Start network discovery
        info!("Server {} started network discovery", self.id);
        self.discover_network();

        // Crossbeam channels for communication between the server's network listener and websocket server
        let (tx, rx) = unbounded::<InternalMessage>();

        // Start network listener
        self.launch_network_listener(tx, rx);
    }

    pub fn launch_network_listener(
        &mut self,
        tx: Sender<InternalMessage>,
        rx: Receiver<InternalMessage>,
    ) {
        // Listen for incoming messages
        loop {
            if (self.last_discovery.elapsed() >= self.discovery_interval) {
                info!("Server {} - Discovering network", self.id);
                self.discover_network();
                self.last_discovery = Instant::now();
            }

            select_biased! {
                // Handle network packets
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(mut packet) = packet_res {
                        self.handle_packet(packet);
                        // TODO: Send stats to websocket server
                        // 1. Use tx to send info to websocket server
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
                default(Duration::from_secs(2)) => {}
            }
        }
    }

    pub(crate) fn send_to_sc(&mut self, event: HostEvent) {
        if self.controller_send.send(event).is_ok() {
            info!("Server {} - Sent NodeEvent to SC", self.id);
        } else {
            error!("Server {} - Error in sending event to SC", self.id);
        }
    }
}
