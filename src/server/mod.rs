pub mod commands;
mod handlers;
mod network;
mod packet_sender;
pub(crate) mod stats;
mod ad;

use crate::server::stats::Stats;
use commands::HostCommand;
use crossbeam_channel::{Receiver, Sender};
use log::{error, info};
use rand::{rng, Rng};
use std::collections::{HashMap};
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType::{Drone};
use wg_2024::packet::{Fragment, NodeType, Packet};
use crate::commands::HostEvent;

pub struct RustBustersServer {
    id: NodeId,
    node_type: NodeType,
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
    echo_mode: bool,
    auto_send: bool,
    auto_send_interval: u64, // in ms
}

impl RustBustersServer {
    pub fn new(
        id: NodeId,
        node_type: NodeType,
        controller_send: Sender<HostEvent>,
        controller_recv: Receiver<HostCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        if let Drone = node_type {
            error!("Drone nodes are not supported by RustBustersServer");
            panic!("Drone nodes are not supported by RustBustersServer");
        }
        info!("Host {} spawned succesfully", id);
        Self {
            id,
            node_type,
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
            echo_mode: false,
            auto_send: false,
            auto_send_interval: 0,
        }
    }

    pub fn auto_send_on(&mut self, interval: u64) {
        self.auto_send = true;
        self.auto_send_interval = interval;
    }

    pub fn auto_send_off(&mut self) {
        self.auto_send = false;
    }

    pub fn run(&self) {
        // Start network discovery
        info!("RustBustersServer {} initiated network discovery", self.id);
        self.discover_network();

        // Loop over and listen for incoming messages
    }
    
    pub(crate) fn send_to_sc(&mut self, event: HostEvent) {
        if self.controller_send.send(event).is_ok() {
            info!(
                "Node {} - Sent NodeEvent to SC",
                self.id
            );
        } else {
            error!(
                "Node {} - Error in sending event to SC",
                self.id
            );
        }
    }
}

