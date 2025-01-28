use log::{info, warn};
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::NodeType::Server;
use wg_2024::packet::{FloodRequest, Packet, PacketType};

use crate::RustBustersServer;

impl RustBustersServer {
    pub fn discover_network(&mut self) {
        // Generate a unique flood_id
        self.flood_id_counter += 1;
        let flood_id = self.flood_id_counter;

        // Initialize the FloodRequest
        let flood_request = FloodRequest {
            flood_id,
            initiator_id: self.id,
            path_trace: vec![(self.id, Server)],
        };

        // Create the packet without routing header (it's ignored for FloodRequest)
        let packet = Packet {
            pack_type: PacketType::FloodRequest(flood_request),
            routing_header: SourceRoutingHeader {
                hop_index: 0,
                hops: vec![],
            },
            session_id: 0,
        };

        for (&neighbor_id, neighbor_sender) in &self.packet_send {
            info!(
                "Server {}: Sending FloodRequest to {} with flood_id {}",
                self.id, neighbor_id, flood_id
            );
            if let Err(err) = neighbor_sender.send(packet.clone()) {
                warn!(
                    "Server {}: Unable to send FloodRequest to {}: {}",
                    self.id, neighbor_id, err
                );
            }
        }
    }
}
