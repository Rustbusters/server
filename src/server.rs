use crate::db::{self, DbManager};
use crate::message::WebSocketMessage;
use crate::{
    InternalChannelsManager, RustBustersServerController, StatsManager, WSChannelsManager,
};
use common_utils::{HostCommand, HostEvent};
use crossbeam::select;
use crossbeam_channel::{select_biased, unbounded, Receiver, RecvTimeoutError, Sender};
use futures::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use rand::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, LazyLock, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};
use wg_2024::config::Server;
use wg_2024::network::NodeId;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};
use wg_2024::packet::{Nack, NackType};

use common_utils::{HostMessage, ServerToClientMessage, Stats, User};

use std::collections::HashSet;

pub struct RustBustersServer {
    pub(crate) id: NodeId,
    pub(crate) controller_send: Sender<HostEvent>,
    pub(crate) controller_recv: Receiver<HostCommand>,
    pub(crate) packet_recv: Receiver<Packet>,
    pub(crate) packet_send: HashMap<NodeId, Sender<Packet>>,
    pub(crate) ws_receiver: Receiver<WebSocketMessage>,
    pub(crate) known_nodes: HashMap<NodeId, NodeType>,
    pub(crate) topology: HashMap<NodeId, Vec<NodeId>>,
    pub(crate) flood_id_counter: u64,
    pub(crate) session_id_counter: u64,
    // (session_id, fragment_index) -> packet
    pub(crate) pending_sent: HashMap<(u64, u64), Packet>,
    // session_id -> (fragments, num_fragments) (u8 is the number of fragments received) (for reassembly)
    pub(crate) pending_received: HashMap<u64, (Vec<Option<Fragment>>, u64)>,
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

        // Init stats for server
        StatsManager::get_or_create_stats(id);
        // Init crossbeam channels websocket server -> network listener
        InternalChannelsManager::add_channel(id);
        // Init crossbeam channels network listener -> websocket server
        let ws_receiver = WSChannelsManager::add_channel(id);

        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(1000..=2000); // Generates a number between 1 and 1000

        info!("Server {} spawned succesfully", id);
        Self {
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            ws_receiver,
            known_nodes: HashMap::new(),
            topology: HashMap::new(),
            flood_id_counter: random_number,
            session_id_counter: 0,
            pending_sent: HashMap::new(),
            pending_received: HashMap::new(),
            websocket_server_address,
            active_users: HashMap::new(),
            last_discovery: Instant::now(),
            discovery_interval,
            db_manager: DbManager::new(id, db_name),
        }
    }

    pub fn launch(&mut self) {
        // Start network discovery
        info!("Server {} started network discovery", self.id);
        self.discover_network();

        // Start network listener
        info!("Server {} launched the network listener", self.id);
        self.launch_network_listener();
    }

    pub fn launch_network_listener(&mut self) {
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
                        self.send_stats();
                    } else {
                        error!("Server {} - Error in receiving packet", self.id);
                    }
                }

                // Handle UI requests
                recv(self.ws_receiver) -> ws_message => {
                    if let Ok(message) = ws_message {
                        self.handle_ws_message(message);
                    } else {
                        error!("Server {} - Error in websocket message receipt", self.id);
                    }
                }

                // Handle Simulation Controller commands
                recv(self.controller_recv) -> command => {
                    if let Ok(cmd) = command {
                        self.handle_command(cmd);
                    } else {
                        error!("Server {} - Error in receiving command", self.id);
                    }
                }

                // No more packets
                default(Duration::from_millis(1000)) => {
                    thread::yield_now(); // Give other threads CPU time
                }
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

    fn send_stats(&self) {
        let stats = StatsManager::get_stats(self.id);
        InternalChannelsManager::send_stats(self.id, stats);
    }

    fn handle_ws_message(&self, message: WebSocketMessage) {
        match message {
            WebSocketMessage::GetServerMessages(server_id) => {
                if let Ok(db_manager) = &self.db_manager {
                    // Retrieve messages from server's database
                    let db_messages = db_manager.get_all();
                    info!("[DB-{}] {db_messages:?}", self.id);

                    if let Ok(messages) = db_messages {
                        // Send through the internal network server -> websocket server messages
                        InternalChannelsManager::send_server_messages(server_id, messages);
                    }
                }
            }
            _ => {}
        }
    }
}
