use crate::server::db::{self, DbManager};
use crate::state::Stats;
use crate::utils::message::WebSocketRequest;
use crate::utils::traits::{Runnable, Service};
use crate::{
    InternalChannelsManager, RustBustersServerController, StatsManager, WSChannelsManager,
};
use common_utils::{HostCommand, HostEvent};
use crossbeam::select;
use crossbeam_channel::{select_biased, unbounded, Receiver, RecvTimeoutError, Sender};
use log::{debug, error, info, warn};
use rand::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, LazyLock, Mutex};
use std::thread::{self, sleep, JoinHandle};
use std::time::{Duration, Instant};
use wg_2024::config::Server;
use wg_2024::network::NodeId;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};
use wg_2024::packet::{Nack, NackType};

use common_utils::{HostMessage, ServerToClientMessage, User};

use std::collections::HashSet;

use super::db::DbMessage;

pub struct RustBustersServer {
    // Basic configuration
    pub(crate) id: NodeId,
    pub(crate) controller_send: Sender<HostEvent>,
    pub(crate) controller_recv: Receiver<HostCommand>,
    pub(crate) packet_send: HashMap<NodeId, Sender<Packet>>,
    pub(crate) packet_recv: Receiver<Packet>,
    pub(crate) ws_receiver: Receiver<WebSocketRequest>, // receiver for the websocket server
    pub(crate) server_controller_sender: Sender<HostCommand>,

    pub(crate) known_node_types: HashMap<NodeId, NodeType>, // node_id -> node_type (Drone/Client/Server)
    pub(crate) topology: HashMap<NodeId, Vec<NodeId>>,

    pub(crate) flood_id_counter: u64,
    pub(crate) session_id_counter: u64,

    pub(crate) pending_sent: HashMap<(u64, u64), Packet>, // (session_id, fragment_index) -> packet
    pub(crate) pending_received: HashMap<u64, (Vec<Option<Fragment>>, u64)>, // session_id -> (fragments, num_fragments) (u8 is the number of fragments received) (for reassembly)
    pub(crate) sessions_info: HashMap<u64, (NodeId, Instant, HostMessage)>, // session_id -> (destination, instant, message)

    // Map for storing the active user sessions
    pub(crate) active_users: HashMap<NodeId, String>,

    // Network discovery
    last_discovery: Instant,      // last time the network discovery was made
    discovery_interval: Duration, // the interval at which to perform the discovery

    // Database manager
    pub(crate) db_manager: Result<DbManager, rusqlite::Error>, // manages the internal server's database

    // Termination condition
    pub(crate) has_stopped: bool,
}

impl Runnable for RustBustersServer {
    fn run(mut self) -> Option<JoinHandle<()>> {
        let handle = thread::spawn(move || {
            self.start();
        });
        Some(handle)
    }
}

impl Service for RustBustersServer {
    fn start(mut self) {
        // Start network discovery
        info!("Server {} started network discovery", self.id);
        self.launch_network_discovery();

        // Start network listener
        info!("Server {} launched the network listener", self.id);
        self.launch_network_listener();
    }
}

impl RustBustersServer {
    pub fn new(
        id: NodeId,
        controller_send: Sender<HostEvent>,
        controller_recv: Receiver<HostCommand>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        packet_recv: Receiver<Packet>,
        server_controller_sender: Sender<HostCommand>,
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
            packet_send,
            packet_recv,
            ws_receiver,
            server_controller_sender,
            known_node_types: HashMap::new(),
            topology: HashMap::new(),
            flood_id_counter: random_number,
            session_id_counter: 0,
            pending_sent: HashMap::new(),
            pending_received: HashMap::new(),
            sessions_info: HashMap::new(),
            active_users: HashMap::new(),
            last_discovery: Instant::now(),
            discovery_interval,
            db_manager: DbManager::new(id, db_name),
            has_stopped: false,
        }
    }

    /// Launches the handling of packets, simulation controller commands and UI requests.
    fn launch_network_listener(&mut self) {
        // Listen for incoming messages
        loop {
            if (self.last_discovery.elapsed() >= self.discovery_interval) {
                info!("Server {} - Discovering network", self.id);
                self.launch_network_discovery();
                self.last_discovery = Instant::now();
            }

            select_biased! {
                // Handle Simulation Controller commands
                recv(self.controller_recv) -> command => {
                    if let Ok(cmd) = command {
                        self.handle_command(cmd);
                        if (self.has_stopped) { // If stop command was received previously
                            break;
                        }
                    } else {
                        error!("Server {} - Error in receiving command", self.id);
                    }
                }

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
                        self.handle_ws_request(message);
                    } else {
                        error!("Server {} - Error in websocket message receipt", self.id);
                    }
                }

                // No more packets
                default(Duration::from_millis(1000)) => {
                    thread::yield_now(); // Give other threads CPU time
                }
            }
        }

        info!("[SERVER-{}] Terminating RustBustersServer thread", self.id);
    }

    fn handle_ws_request(&self, message: WebSocketRequest) {
        match message {
            WebSocketRequest::GetStats => self.send_stats(),
            WebSocketRequest::GetMessages => self.send_db_messages(),
            WebSocketRequest::GetActiveUsers => self.send_active_users(),
            _ => {}
        }
    }

    pub(crate) fn send_stats(&self) {
        let stats = StatsManager::get_stats(self.id);
        InternalChannelsManager::send_stats(self.id, stats);
    }

    pub(crate) fn send_db_message(&self, db_message: DbMessage) {
        if let Ok(db_manager) = &self.db_manager {
            info!("[DB-{}] {db_message:?}", self.id);
            // Send through the internal network server -> websocket server messages
            InternalChannelsManager::send_message(self.id, db_message);
        }
    }

    pub(crate) fn send_db_messages(&self) {
        if let Ok(db_manager) = &self.db_manager {
            // Retrieve messages from server's database
            if let Ok(db_messages) = db_manager.get_all() {
                info!("[DB-{}] {db_messages:?}", self.id);
                // Send through the internal network server -> websocket server messages
                InternalChannelsManager::send_messages(self.id, db_messages);
            }
        }
    }

    pub(crate) fn send_active_users(&self) {
        let active_users = self.get_active_users();
        InternalChannelsManager::send_active_users(self.id, active_users);
    }

    pub(crate) fn get_active_users(&self) -> Vec<User> {
        self.active_users
            .iter()
            .map(|(id, name)| User::new(id.clone(), name.to_string()))
            .collect()
    }
}
