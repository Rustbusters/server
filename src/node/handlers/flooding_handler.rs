use log::{info, warn};
use wg_2024::network::SourceRoutingHeader;
use crate::SimpleHost;
use wg_2024::packet::{FloodRequest, FloodResponse, Packet, PacketType};

impl SimpleHost {
    pub(crate) fn handle_flood_response(&mut self, flood_response: FloodResponse) {
        for window in flood_response.path_trace.windows(2) {
            if let [(from_id, from_type), (to_id, to_type)] = window {
                self.known_nodes.insert(*from_id, from_type.clone());
                self.known_nodes.insert(*to_id, to_type.clone());

                // Update topology
                let from_to = self.topology.entry(*from_id).or_default();
                if !from_to.contains(to_id) {
                    from_to.push(*to_id);
                }

                let to_from = self.topology.entry(*to_id).or_default();
                if !to_from.contains(from_id) {
                    to_from.push(*from_id);
                }
            }
        }

        info!("Node {}: Updated topology: {:?}", self.id, self.topology);
        info!("Node {}: Known nodes: {:?}", self.id, self.known_nodes);
    }

    pub(crate) fn handle_flood_request(&mut self, flood_request: FloodRequest, session_id: u64) {
        let mut new_path_trace = flood_request.path_trace.clone();
        new_path_trace.push((self.id, self.node_type.clone()));

        let flood_response = FloodResponse {
            flood_id: flood_request.flood_id,
            path_trace: new_path_trace.clone(),
        };

        // Create the packet
        let response_packet = Packet {
            pack_type: PacketType::FloodResponse(flood_response),
            routing_header: SourceRoutingHeader {
                hop_index: 1,
                hops: new_path_trace.iter().map(|(id, _)| *id).rev().collect(),
            },
            session_id,
        };

        // Send the FloodResponse back to the initiator
        if let Some(sender) = self
            .packet_send
            .get(&response_packet.routing_header.hops[1])
        {
            info!(
                "Node {}: Sending FloodResponse to initiator {}, next hop {}",
                self.id, flood_request.initiator_id, response_packet.routing_header.hops[1]
            );
            let _ = sender.send(response_packet);
        } else {
            warn!(
                "Node {}: Cannot send FloodResponse to initiator {}",
                self.id, flood_request.initiator_id
            );
        }
    }
}
