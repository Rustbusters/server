use crate::RustBustersServer;
use log::info;
use wg_2024::packet::{Packet, PacketType};

impl RustBustersServer {
    pub(crate) fn handle_packet(&mut self, packet: Packet) {
        match packet.pack_type {
            PacketType::FloodRequest(flood_request) => {
                info!(
                    "Server {}: Received FloodRequest with flood_id {}",
                    self.id, flood_request.flood_id
                );
                self.handle_flood_request(flood_request, packet.session_id);
            }
            PacketType::FloodResponse(flood_response) => {
                info!(
                    "Server {}: Received FloodResponse with flood_id {}",
                    self.id, flood_response.flood_id
                );
                self.handle_flood_response(flood_response);
            }
            PacketType::MsgFragment(fragment) => {
                // Handle incoming message fragments
                info!(
                    "Server {}: Received fragment {} of session {}",
                    self.id, fragment.fragment_index, packet.session_id
                );
                self.handle_message_fragment(fragment, packet.session_id, packet.routing_header);
            }
            PacketType::Ack(ack) => {
                // Handle Acknowledgments
                info!(
                    "Server {}: Received Ack for fragment {}",
                    self.id, ack.fragment_index
                );
                self.handle_ack(packet.session_id, ack.fragment_index);
            }
            PacketType::Nack(nack) => {
                // Handle Negative Acknowledgments
                info!("Server {}: Received Nack {nack:?}", self.id);
                self.handle_nack(packet.session_id, nack.fragment_index, nack.nack_type);
            }
        }
    }
}
