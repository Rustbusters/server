pub mod commands;
mod handlers;
mod network;
mod packet_sender;
pub(crate) mod stats;
mod ad;

use crate::server::stats::Stats;
use commands::HostCommand;
use crossbeam_channel::{Receiver, Sender};
use petgraph::prelude::GraphMap;
use petgraph::Undirected;
use log::{error, info};
use std::collections::{HashMap};
use wg_2024::network::NodeId;
use wg_2024::packet::{Fragment, NodeType, Packet};
use crate::commands::HostEvent;
use rand::*;

pub struct RustBustersServer {
    id: NodeId,
    controller_send: Sender<HostEvent>,
    controller_recv: Receiver<HostCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    known_nodes: HashMap<NodeId, NodeType>,
    topology: GraphMap<NodeId, f32, Undirected>,
    flood_id_counter: u64,
    session_id_counter: u64,
    // (session_id, fragment_index) -> packet
    pending_sent: HashMap<(u64, u64), Packet>,
    // session_id -> (fragments, num_fragments) (u8 is the number of fragments received) (for reassembly)
    pending_received: HashMap<u64, (Vec<Option<Fragment>>, u64)>,
    stats: Stats,
}

impl RustBustersServer {
    pub fn new(
        id: NodeId,
        controller_send: Sender<HostEvent>,
        controller_recv: Receiver<HostCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        info!("Server {} spawned succesfully", id);
        Self {
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            known_nodes: HashMap::new(),
            topology: GraphMap::new(),
            flood_id_counter: rng().random_range(1000..=2000),
            session_id_counter: rng().random_range(100..=200),
            pending_sent: HashMap::new(),
            pending_received: HashMap::new(),
            stats: Stats::default(),
        }
    }

    pub fn run(&self) {
        // Start network discovery
        info!("RustBustersServer {} initiated network discovery", self.id);
        self.discover_network();

        // Loop over and listen for incoming messages
        todo!();
        loop {

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

