use crate::db::{self, DbManager};
use crate::message::WebSocketMessage;
use crate::{ConnectionsWrapper, RustBustersServerController, StatsWrapper};
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
    pub(crate) session_ids: HashMap<NodeId, u64>,
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
        StatsWrapper::get_or_create_stats(id);
        // Init crossbeam channels websocket server -> network listener
        ConnectionsWrapper::add_connection(id);
        // Init crossbeam channels network listener -> websocket server
        let ws_receiver = ConnectionsWrapper::add_ws_channel(id);

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
            session_ids: HashMap::new(),
            flood_id_counter: random_number,
            session_id_counter: 0,
            pending_sent: HashMap::new(),
            pending_received: HashMap::new(),
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

        // Start network listener
        self.launch_network_listener();
    }

    pub fn launch_network_listener(&mut self) {
        // Listen for incoming messages
        loop {
            if (self.last_discovery.elapsed() >= self.discovery_interval) {
                println!("Server {} - Discovering network", self.id);
                info!("Server {} - Discovering network", self.id);
                self.discover_network();
                self.last_discovery = Instant::now();
            }

            // let nack = Nack {
            //     fragment_index: 0, // If the packet is not a fragment, it's considered as a whole, so fragment_index will be 0.
            //     nack_type: NackType::Dropped,
            // };
            // if self.id == 7 {
            //     let packet = Packet {
            //         pack_type: PacketType::Nack(nack),
            //         routing_header: SourceRoutingHeader {
            //             hop_index: 0,
            //             hops: vec![0],
            //         },
            //         session_id: 0,
            //     };
            //     self.packet_send.get(&2).unwrap().send(packet);
            //     println!("Sending from server {}", self.id);
            // }

            select_biased! {
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
                },

                // Handle network packets
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(mut packet) = packet_res {
                        self.handle_packet(packet);
                        let stats = StatsWrapper::get_stats(self.id);
                        println!("Server {} is sending stats to WS", self.id);
                        ConnectionsWrapper::send_stats(self.id, stats);
                    } else {
                        error!("Server {} - Error in receiving packet", self.id);
                    }
                },

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

    pub(crate) fn handle_ws_message(&self, message: WebSocketMessage) {
        match message {
            WebSocketMessage::GetServerMessages(server_id) => {
                if let Ok(db_manager) = &self.db_manager {
                    // Retrieve messages from server's database
                    let db_messages = db_manager.get_all();
                    println!("[DB] {db_messages:?}");

                    if let Ok(messages) = db_messages {
                        // Send through the ConnectionsWrapper
                        ConnectionsWrapper::send_server_messages(server_id, messages);
                    }
                }
            }
            _ => {}
        }
    }
}
