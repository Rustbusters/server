use crossbeam_channel::{select, Receiver, Sender};
use rand::seq::IteratorRandom;
use rand::{random, rng, Rng};
use log::{info, error, debug, warn};
use std::collections::{HashMap, HashSet, VecDeque};
use std::thread;
use std::time::Duration;
use wg_2024::controller::{DroneCommand, NodeEvent};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::NodeType::{Client, Drone, Server};
use wg_2024::packet::{
    FloodRequest, FloodResponse, Fragment, Message, MessageContent, MessageData, NodeType, Packet,
    PacketType,
};

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
    pending_floods: HashSet<u64>,
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
        if let NodeType::Drone = node_type {
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
            pending_floods: HashSet::new(),
        }
    }

    pub fn run(&mut self) {
        // Start network discovery
        info!("Host {} started network discovery", self.id);
        self.network_discovery();

        // Random number generator
        let mut rng = rand::rng();

        loop {
            // Random sleep between 1 and 5 seconds
            let sleep_duration = rng.random_range(1..=10);
            thread::sleep(Duration::from_secs(sleep_duration));

            let mut hosts = self.known_nodes.clone();
            hosts.retain(|&id, node_type| match node_type {
                NodeType::Client => true,
                NodeType::Server => true,
                _ => false,
            });

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

    fn network_discovery(&mut self) {
        // Generate a unique flood_id
        self.flood_id_counter += 1;
        let flood_id = self.flood_id_counter;

        // Initialize the FloodRequest
        let flood_request = FloodRequest {
            flood_id,
            initiator_id: self.id,
            path_trace: vec![(self.id, self.node_type.clone())],
        };

        // Create the packet without routing header (it's ignored for FloodRequest)
        let packet = Packet {
            pack_type: PacketType::FloodRequest(flood_request),
            routing_header: SourceRoutingHeader {
                hop_index: 0,
                hops: vec![self.id],
            },
            session_id: 0,
        };

        // Send the FloodRequest to all immediate neighbors
        for (&neighbor_id, neighbor_sender) in &self.packet_send {
            let _ = neighbor_sender.send(packet.clone());
        }

        // Mark this flood_id as pending
        self.pending_floods.insert(flood_id);
    }

    fn handle_packet(&mut self, packet: Packet) {
        match packet.pack_type {
            PacketType::FloodResponse(flood_response) => {
                self.handle_flood_response(flood_response);
            }
            PacketType::MsgFragment(fragment) => {
                // Handle incoming message fragments
                self.handle_message_fragment(packet.session_id, fragment);
            }
            PacketType::Ack(ack) => {
                // Handle Acknowledgments
                info!("Received Ack for fragment {}", ack.fragment_index);
            }
            PacketType::Nack(nack) => {
                // Handle Negative Acknowledgments
                info!("Received Nack {nack:?}");
            }
            _ => {
                // Other packet types can be handled here
            }
        }
    }

    fn handle_flood_response(&mut self, flood_response: FloodResponse) {
        if self.pending_floods.contains(&flood_response.flood_id) {
            // Process the flood response
            for window in flood_response.path_trace.windows(2) {
                if let [(from_id, from_type), (to_id, to_type)] = window {
                    // Update known nodes
                    self.known_nodes.insert(*from_id, from_type.clone());
                    self.known_nodes.insert(*to_id, to_type.clone());

                    // Update topology
                    self.topology
                        .entry(*from_id)
                        .or_insert_with(Vec::new)
                        .push(*to_id);
                    self.topology
                        .entry(*to_id)
                        .or_insert_with(Vec::new)
                        .push(*from_id);
                }
            }

            // Remove the flood_id from pending
            self.pending_floods.remove(&flood_response.flood_id);

            info!("Updated topology: {:?}", self.topology);
            info!("Known nodes: {:?}", self.known_nodes);
        }
    }

    fn send_random_message(&mut self, destination_id: NodeId) {
        // Compute the route to the destination
        if let Some(route) = self.compute_route(destination_id) {
            // // Create a random message content
            // let message_content = MessageContent::ReqServerType;
            //
            // Increment session_id_counter
            self.session_id_counter += 1;
            let session_id = self.session_id_counter;

            // // Create the message
            // let message_data = Message {
            //     message_data: MessageData {
            //         source_id: 0,
            //         session_id,
            //         content: message_content,
            //     },
            //     routing_header: SourceRoutingHeader {
            //         hop_index: 0,
            //         hops: route.clone(),
            //     },
            // };

            // Serialize and fragment the message
            let fragments = self.fragment_message(/*&message_data*/);

            // Send the fragments along the route
            for fragment in fragments {
                let packet = Packet {
                    pack_type: PacketType::MsgFragment(fragment),
                    routing_header: SourceRoutingHeader {
                        hop_index: 1, // Set to 1 initially by the sender
                        hops: route.clone(),
                    },
                    session_id,
                };

                // Send the packet to the first hop
                let next_hop = packet.routing_header.hops[1];
                if let Some(sender) = self.packet_send.get(&next_hop) {
                    let _ = sender.send(packet);
                }
            }

            info!("Sent message to {} via route {:?}", destination_id, route);
        } else {
            info!("No route to {}", destination_id);
        }
    }
    fn compute_route(&self, destination_id: NodeId) -> Option<Vec<NodeId>> {
        // Simple BFS to find the shortest path
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut predecessors = HashMap::new();

        visited.insert(self.id);
        queue.push_back(self.id);

        while let Some(current) = queue.remove(0) {
            if current == destination_id {
                // Build the path from self.id to destination_id
                let mut path = vec![destination_id];
                let mut node = destination_id;
                while node != self.id {
                    if let Some(&pred) = predecessors.get(&node) {
                        path.push(pred);
                        node = pred;
                    } else {
                        break;
                    }
                }
                path.reverse();
                return Some(path);
            }

            if let Some(neighbors) = self.topology.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                        predecessors.insert(neighbor, current);
                    }
                }
            }
        }

        None // No route found
    }

    fn fragment_message(&self /*message: &Message*/) -> Vec<Fragment> {
        // Serialize the MessageData
        // let serialized_data = bincode::serialize(&message.message_data).unwrap();
        // TODO: Implement serialization (not derived in the lib)... for now, use random data
        let serialized_data: Vec<u8> = (0..rng().random_range(100..200))
            .map(|_| random())
            .collect();

        // Fragment the data into chunks of 80 bytes
        let chunk_size = 80;
        let total_size = serialized_data.len();
        let total_n_fragments = ((total_size + chunk_size - 1) / chunk_size) as u64;

        let mut fragments = Vec::new();

        for (i, chunk) in serialized_data.chunks(chunk_size).enumerate() {
            let mut data_array = [0u8; 80];
            let length = chunk.len();
            data_array[..length].copy_from_slice(chunk);

            let fragment = Fragment {
                fragment_index: i as u64,
                total_n_fragments,
                length: length as u8,
                data: data_array,
            };

            fragments.push(fragment);
        }

        fragments
    }

    fn handle_message_fragment(&mut self, session_id: u64, fragment: Fragment) {
        // Handle incoming message fragments (reassembly not implemented for simplicity)
        info!(
            "Received fragment {} of session {}",
            fragment.fragment_index, session_id
        );
    }
}
