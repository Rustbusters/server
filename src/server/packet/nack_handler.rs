use crate::{RustBustersServer, StatsManager};
use common_utils::HostEvent;
use common_utils::{PacketHeader, PacketTypeHeader};
use log::{info, warn};
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::NackType;
use wg_2024::packet::NackType::Dropped;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};

impl RustBustersServer {
    pub(crate) fn handle_nack(
        &mut self,
        session_id: u64,
        fragment_index: u64,
        nack_type: NackType,
    ) {
        match self.pending_sent.get(&(session_id, fragment_index)) {
            Some(packet) => {
                match nack_type {
                    NackType::Dropped => {
                        info!("Server {}: Resending fragment {}", self.id, fragment_index);
                        // Resend the fragment
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
                        // Removing from known nodes
                        self.known_nodes.remove(&drone_id);

                        // Calculating new route and resending fragment
                        let dest_id = *packet.routing_header.hops.last().expect("No destination");
                        if let Some(route) = self.find_route(dest_id) {
                            let mut new_packet = packet.clone();
                            new_packet.routing_header.hops = route.clone();
                            new_packet.routing_header.hop_index = 1;

                            // Send the packet
                            if let Some(sender) =
                                self.packet_send.get(&new_packet.routing_header.hops[1])
                            {
                                if let Err(err) = sender.send(new_packet.clone()) {
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
                                        routing_header: new_packet.routing_header.clone(),
                                    }));
                                }
                            }
                        } else {
                            warn!("Server {}: Error in finding route", self.id);
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
