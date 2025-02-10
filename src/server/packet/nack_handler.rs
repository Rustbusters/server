use std::clone;

use crate::{RustBustersServer, StatsManager};
use common_utils::HostEvent;
use common_utils::{PacketHeader, PacketTypeHeader};
use log::{info, warn};
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::NackType;
use wg_2024::packet::NackType::Dropped;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};

impl RustBustersServer {
    /// Handles a Negative Acknowledgment (NACK) for a previously sent message fragment.
    ///
    /// ### Parameters
    /// - `session_id: u64` – The session identifier to which the NACK belongs.
    /// - `fragment_index: u64` – The index of the fragment that was negatively acknowledged.
    /// - `nack_type: NackType` – The type of NACK, indicating the reason for failure.
    ///
    /// ### Behavior
    /// - If the fragment is found in `pending_sent`:
    ///   - If `NackType::Dropped`, resends the fragment and updates statistics.
    ///   - If `NackType::ErrorInRouting`, removes the faulty node from `topology` and `known_nodes`,
    ///     recalculates the route, and attempts to resend the fragment.
    ///   - If `NackType::DestinationIsDrone` or `NackType::UnexpectedRecipient`, logs a warning.
    /// - If the fragment is unknown, logs a warning.
    pub(crate) fn handle_nack(
        &mut self,
        session_id: u64,
        fragment_index: u64,
        nack_type: NackType,
    ) {
        match self
            .pending_sent
            .get(&(session_id, fragment_index))
            .cloned()
        {
            Some(mut packet) => {
                match nack_type {
                    NackType::Dropped => {
                        info!("Server {}: Resending fragment {}", self.id, fragment_index);
                        // Resend the packet
                        if let Some(sender) = self.packet_send.get(&packet.routing_header.hops[1]) {
                            if let Err(err) = sender.send(packet.clone()) {
                                warn!(
                                    "Server {}: Unable to resend fragment {}: {}",
                                    self.id, fragment_index, err
                                );
                            } else {
                                StatsManager::inc_message_fragments_sent(self.id);

                                // Send MsgFragment to Simulation Controller
                                self.send_to_sc(HostEvent::PacketSent(PacketHeader {
                                    session_id,
                                    pack_type: PacketTypeHeader::MsgFragment,
                                    routing_header: packet.routing_header.clone(),
                                }));
                            }
                        }
                    }
                    NackType::ErrorInRouting(drone_id) => {
                        warn!(
                            "Server {}: Nack for fragment {} with type {:?}",
                            self.id, fragment_index, nack_type
                        );

                        // Updating topology
                        self.topology.remove(&drone_id);
                        self.topology.iter_mut().for_each(|(src_id, neighbors)| {
                            neighbors.retain(|&dest_id| dest_id != drone_id);
                        });
                        // Removing from known nodes
                        self.known_nodes.remove(&drone_id);

                        // Calculating new route and resending fragment
                        let dest_id = *packet.routing_header.hops.last().expect("No destination");
                        if let Some(route) = self.find_route(dest_id) {
                            packet.routing_header.hops = route.clone();
                            packet.routing_header.hop_index = 1;
                        } else {
                            warn!("Server {}: Error in finding route", self.id);
                            self.launch_network_discovery();
                            if let Some(route) = self.find_route(dest_id) {
                                packet.routing_header.hops = route.clone();
                                packet.routing_header.hop_index = 1;
                            }
                        }

                        // Resend the packet
                        if let Some(sender) = self.packet_send.get(&packet.routing_header.hops[1]) {
                            if let Err(err) = sender.send(packet.clone()) {
                                warn!(
                                    "Server {}: Unable to resend fragment {}: {}",
                                    self.id, fragment_index, err
                                );
                            } else {
                                StatsManager::inc_message_fragments_sent(self.id);

                                // Send MsgFragment to Simulation Controller
                                self.send_to_sc(HostEvent::PacketSent(PacketHeader {
                                    session_id,
                                    pack_type: PacketTypeHeader::MsgFragment,
                                    routing_header: packet.routing_header.clone(),
                                }));
                            }
                        }
                    }
                    NackType::DestinationIsDrone | NackType::UnexpectedRecipient(_) => {
                        warn!(
                            "Server {}: Nack for fragment {} with type {:?}",
                            self.id, fragment_index, nack_type
                        );
                    }
                    _ => {}
                }
            }
            None => {
                warn!("Server {}: Nack for unknown fragment", self.id);
            }
        }
    }
}
