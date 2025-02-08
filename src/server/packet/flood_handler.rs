use crate::{RustBustersServer, StatsManager};
use common_utils::HostEvent;
use common_utils::{PacketHeader, PacketTypeHeader};
use log::info;
use log::warn;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{FloodRequest, FloodResponse, Packet, PacketType};

use wg_2024::packet::NodeType::Server;

impl RustBustersServer {
    /// Handles a flood response and updates the network topology.
    ///
    /// ### Parameters
    /// - `flood_response: FloodResponse` – The response containing a `path_trace` that represents
    ///   the sequence of nodes involved in the flood process.
    ///
    /// ### Behavior
    /// 1. Iterates through `path_trace` in pairs of nodes (windows of size 2).
    /// 2. Extracts `from_id`, `to_id` (node IDs) and their respective types.
    /// 3. Adds both nodes to the `known_nodes` map to track discovered nodes and their types.
    /// 4. Updates `topology` by ensuring bidirectional connectivity between `from_id` and `to_id`.
    pub(crate) fn handle_flood_response(&mut self, flood_response: FloodResponse) {
        for window in flood_response.path_trace.windows(2) {
            if let [(from_id, from_type), (to_id, to_type)] = window {
                self.known_nodes.insert(*from_id, *from_type);
                self.known_nodes.insert(*to_id, *to_type);

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

        info!("Server {}: Updated topology: {:?}", self.id, self.topology);
        info!("Server {}: Known nodes: {:?}", self.id, self.known_nodes);
    }

    /// Handles an incoming flood request and responds accordingly.
    ///
    /// ### Parameters
    /// - `flood_request: FloodRequest` – The received flood request containing `path_trace`
    ///   and other metadata.
    /// - `session_id: u64` – The session identifier associated with this request.
    ///
    /// ### Behavior
    /// 1. Clones `path_trace` from the request and appends the server's own ID and type.
    /// 2. Creates a `FloodResponse` containing the updated path trace.
    /// 3. If the request was initiated by this server:
    ///    - Logs the event.
    ///    - Directly calls `handle_flood_response` to update its topology without sending a response.
    /// 4. Otherwise:
    ///    - Constructs a `Packet` with a `FloodResponse`.
    ///    - Sets up source routing by reversing the `path_trace`.
    ///    - Attempts to send the packet to the next hop in the route.
    /// 5. If sending fails:
    ///    - Logs a warning and notifies the simulation controller via `ControllerShortcut`.
    /// 6. Updates flood response statistics.
    /// 7. Notifies the simulation controller about the sent `FloodResponse` packet.
    pub(crate) fn handle_flood_request(&mut self, flood_request: FloodRequest, session_id: u64) {
        let mut new_path_trace = flood_request.path_trace.clone();
        new_path_trace.push((self.id, Server));

        let flood_response = FloodResponse {
            flood_id: flood_request.flood_id,
            path_trace: new_path_trace.clone(),
        };

        // If the packet was sent by this server, learn the topology without sending a response
        if flood_request.initiator_id == self.id {
            info!(
                "Server {}: Received own FloodRequest with flood_id {}. Learning topology...",
                self.id, flood_request.flood_id
            );
            self.handle_flood_response(flood_response);
            return;
        }

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
                "Server {}: Sending FloodResponse to initiator {}, next hop {}",
                self.id, flood_request.initiator_id, response_packet.routing_header.hops[1]
            );

            if let Err(err) = sender.send(response_packet.clone()) {
                warn!(
                    "Server {}: Error sending FloodResponse to initiator {}: {}",
                    self.id, flood_request.initiator_id, err
                );
                self.send_to_sc(HostEvent::ControllerShortcut(response_packet.clone()))
            }

            // Update stats
            StatsManager::inc_flood_responses_sent(self.id);
        } else {
            warn!(
                "Server {}: Cannot send FloodResponse to initiator {}",
                self.id, flood_request.initiator_id
            );
            self.send_to_sc(HostEvent::ControllerShortcut(response_packet.clone()))
        }

        // Send FloodResponse packet to Simulation Controller
        self.send_to_sc(HostEvent::PacketSent(PacketHeader {
            session_id,
            pack_type: PacketTypeHeader::FloodResponse,
            routing_header: response_packet.routing_header.clone(),
        }));
    }
}
