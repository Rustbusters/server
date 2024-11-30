use crate::SimpleHost;
use log::info;
use wg_2024::packet::{Packet, PacketType};

impl SimpleHost {
    pub(crate) fn handle_packet(&mut self, packet: Packet) {
        match packet.pack_type {
            PacketType::FloodRequest(flood_request) => {
                info!(
                    "Node {}: Received FloodRequest with flood_id {}",
                    self.id, flood_request.flood_id
                );
                self.handle_flood_request(flood_request, packet.session_id);
            }
            PacketType::FloodResponse(flood_response) => {
                info!(
                    "Node {}: Received FloodResponse with flood_id {}",
                    self.id, flood_response.flood_id
                );
                self.handle_flood_response(flood_response);
            }
            PacketType::MsgFragment(fragment) => {
                // Handle incoming message fragments
                info!(
                    "Node {}: Received fragment {} of session {}",
                    self.id, fragment.fragment_index, packet.session_id
                );
                self.handle_message_fragment(fragment, packet.session_id, packet.routing_header);
            }
            PacketType::Ack(ack) => {
                // Handle Acknowledgments
                info!(
                    "Node {}: Received Ack for fragment {}",
                    self.id, ack.fragment_index
                );
                self.stats.inc_acks_received();
            }
            PacketType::Nack(nack) => {
                // Handle Negative Acknowledgments
                info!("Node {}: Received Nack {nack:?}", self.id);
                self.stats.inc_nacks_received();
                // TODO: handle Nacks -> keep track of pending packets
            }
        }
    }
}
