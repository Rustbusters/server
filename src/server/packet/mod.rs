mod ack_handler;
mod client_handler;
mod flood_handler;
mod fragment_handler;
mod nack_handler;
mod sender;

use crate::RustBustersServer;
use crate::StatsManager;
use log::{debug, info, warn};

use wg_2024::packet::{Ack, FloodRequest, FloodResponse, Fragment, Packet, PacketType};

impl RustBustersServer {
    /// Handles an incoming packet and dispatches it to the appropriate handler based on its type.
    ///
    /// ### Parameters
    /// - `packet: Packet` – The received packet containing a specific `PacketType`
    ///   and associated metadata.
    ///
    /// ### Behavior
    /// - Identifies the type of the received packet and processes it accordingly:
    ///   - `FloodRequest`: Logs the event, updates statistics, and calls `handle_flood_request()`.
    ///   - `FloodResponse`: Logs the event, updates statistics, and calls `handle_flood_response()`.
    ///   - `MsgFragment`: Logs the event, updates statistics, and calls `handle_fragment()`
    ///     to process message reassembly.
    ///   - `Ack`: Logs the acknowledgment, updates statistics, and calls `handle_ack()`.
    ///   - `Nack`: Logs the negative acknowledgment, updates statistics, and calls `handle_nack()`.
    pub(crate) fn handle_packet(&mut self, packet: Packet) {
        match packet.pack_type {
            PacketType::FloodRequest(flood_request) => {
                info!(
                    "Server {}: Received FloodRequest with flood_id {}",
                    self.id, flood_request.flood_id
                );
                StatsManager::inc_flood_requests_received(self.id);
                self.handle_flood_request(flood_request, packet.session_id);
            }
            PacketType::FloodResponse(flood_response) => {
                info!(
                    "Server {}: Received FloodResponse with flood_id {}",
                    self.id, flood_response.flood_id
                );
                StatsManager::inc_flood_responses_received(self.id);
                self.handle_flood_response(flood_response);
            }
            PacketType::MsgFragment(fragment) => {
                // Handle incoming message fragments
                info!(
                    "Server {}: Received fragment {} of session {}",
                    self.id, fragment.fragment_index, packet.session_id
                );
                StatsManager::inc_message_fragments_received(self.id);
                self.handle_fragment(fragment, packet.session_id, packet.routing_header);
            }
            PacketType::Ack(ack) => {
                // Handle Acknowledgments
                info!(
                    "Server {}: Received Ack for fragment {}",
                    self.id, ack.fragment_index
                );
                StatsManager::inc_acks_received(self.id);
                self.handle_ack(packet.session_id, ack.fragment_index);
            }
            PacketType::Nack(nack) => {
                // Handle Negative Acknowledgments
                info!("Server {}: Received Nack {nack:?}", self.id);
                StatsManager::inc_nacks_received(self.id);
                self.handle_nack(packet.session_id, nack.fragment_index, nack.nack_type);
            }
        }
    }
}
