mod assembler;
pub mod commands;
mod disassembler;
mod handlers;
mod messages;
mod network_discovery;
mod packet_sender;
mod router;
pub(crate) mod stats;

use crate::node::stats::Stats;
use commands::HostCommand;
use crossbeam_channel::{select, Receiver, Sender};
use log::{error, info};
use rand::seq::IteratorRandom;
use rand::{rng, Rng};
use std::collections::{HashMap};
use std::time::Duration;
use wg_2024::controller::DroneEvent;
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType::{Client, Drone, Server};
use wg_2024::packet::{Fragment, NodeType, Packet};
use crate::commands::HostEvent;

pub struct Server {
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

impl Server {
    pub fn new(
        id: NodeId,
        node_type: NodeType,
        controller_send: Sender<HostEvent>,
        controller_recv: Receiver<HostCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        if let Drone = node_type {
            error!("Drone nodes are not supported by SimpleHost");
            panic!("Drone nodes are not supported by SimpleHost");
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

    pub fn echo_mode_on(&mut self) {
        self.echo_mode = true;
    }

    pub fn echo_mode_off(&mut self) {
        self.echo_mode = false;
    }

    pub fn auto_send_on(&mut self, interval: u64) {
        self.auto_send = true;
        self.auto_send_interval = interval;
    }

    pub fn auto_send_off(&mut self) {
        self.auto_send = false;
    }

    pub fn run(&mut self) {
        // Start network discovery
        info!("Host {} started network discovery", self.id);
        self.discover_network();

        // Random number generator
        let mut rng = rand::rng();

        let mut last_send_time = std::time::Instant::now();

        loop {
            if self.auto_send
                && last_send_time.elapsed() >= Duration::from_millis(self.auto_send_interval)
            {
                last_send_time = std::time::Instant::now();

                // Retain only servers if the server is a client and viceversa
                let mut reachable_hosts = self.known_nodes.clone();
                reachable_hosts.retain(|&_id, node_type| match self.node_type {
                    Client => *node_type == Server,
                    Server => *node_type == Client,
                    Drone => false,
                });

                // Choose a random server to send a message to
                if !reachable_hosts.is_empty() {
                    if let Some(&random_node_id) = reachable_hosts
                        .keys()
                        .filter(|&&id| id != self.id)
                        .choose(&mut rng)
                    {
                        // Send a message to the random server
                        println!(
                            "Node {}: Send a message to server {}",
                            self.id, random_node_id
                        );
                        info!("Node {}: Send a message to server {random_node_id}", self.id);
                        self.send_random_message(random_node_id);
                    }
                } else {
                    info!("Node {}: No reachable hosts", self.id);
                }
            }

            // Handle incoming packets
            select! {
                // Handle incoming packets
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        self.handle_packet(packet);
                    } else {
                        // Channel closed
                        break;
                    }
                },
                // Handle SC commands
                recv(self.controller_recv) -> command => {
                    if let Ok(cmd) = command {
                        self.handle_command(cmd);
                    } else {
                        // Channel closed
                        break;
                    }
                },
                default(Duration::from_millis(100)) => {
                  // No more packets
                }
            }
        }
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

