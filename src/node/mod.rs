mod networ_discovery;
mod packet_sender;
mod fragmenter;
mod router;
mod handlers;

use crossbeam_channel::{select, Receiver, Sender};
use log::{error, info};
use rand::seq::IteratorRandom;
use rand::{rng, Rng};
use std::collections::{HashMap};
use std::thread;
use std::time::Duration;
use wg_2024::controller::{DroneCommand, NodeEvent};
use wg_2024::network::{NodeId};
use wg_2024::packet::{NodeType, Packet};
use wg_2024::packet::NodeType::{Client, Drone, Server};

pub struct SimpleHost {
    id: NodeId,
    node_type: NodeType,
    controller_send: Sender<NodeEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    known_nodes: HashMap<NodeId, NodeType>,
    topology: HashMap<NodeId, Vec<NodeId>>,
    flood_id_counter: u64,
    session_id_counter: u64,
}

impl SimpleHost {
    pub fn new(
        id: NodeId,
        node_type: NodeType,
        controller_send: Sender<NodeEvent>,
        controller_recv: Receiver<DroneCommand>,
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
        }
    }

    pub fn run(&mut self) {
        // Start network discovery
        info!("Host {} started network discovery", self.id);
        self.discover_network();

        // Random number generator
        let mut rng = rand::rng();

        loop {
            // Random sleep between 1 and 5 seconds
            let sleep_duration = rng.random_range(1..=10);
            thread::sleep(Duration::from_secs(sleep_duration));

            let mut hosts = self.known_nodes.clone();
            hosts.retain(|&_id, node_type| matches!(node_type, Client | Server));

            // Choose a random node to send a message to
            if !hosts.is_empty() {
                if let Some(&random_node_id) =
                    hosts.keys().filter(|&&id| id != self.id).choose(&mut rng)
                {
                    // Send a message to the random node
                    info!("Send a message to node {random_node_id}");
                    self.send_random_message(random_node_id);
                }
            }

            // Handle incoming packets
            loop {
                select! {
                    recv(self.packet_recv) -> packet_res => {
                        if let Ok(packet) = packet_res {
                            self.handle_packet(packet);
                        } else {
                            // Channel closed
                            break;
                        }
                    },
                    default(Duration::from_millis(100)) => {
                      // No more packets
                      break;
                    }
                }
            }
        }
    }
}
