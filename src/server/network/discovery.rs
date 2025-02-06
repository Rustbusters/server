use common_utils::HostEvent;
use common_utils::{PacketHeader, PacketTypeHeader};
use log::{info, warn};
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::NodeType::Server;
use wg_2024::packet::{FloodRequest, Packet, PacketType};

use crate::{RustBustersServer, StatsManager};

impl RustBustersServer {
    pub fn launch_network_discovery(&mut self) {
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

        for (neighbor_id, neighbor_sender) in self.packet_send.clone() {
            info!(
                "Server {}: Sending FloodRequest to {} with flood_id {}",
                self.id, neighbor_id, flood_id
            );
            if let Err(err) = neighbor_sender.send(packet.clone()) {
                warn!(
                    "Server {}: Unable to send FloodRequest to {}: {}",
                    self.id, neighbor_id, err
                );
            } else {
                // Update stats
                StatsManager::inc_flood_requests_sent(self.id);

                // Send FloodRequest packet to Simulation Controller
                self.send_to_sc(HostEvent::PacketSent(PacketHeader {
                    session_id: 0,
                    pack_type: PacketTypeHeader::FloodRequest,
                    routing_header: packet.routing_header.clone(),
                }));
            }
        }
    }
}
