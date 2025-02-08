use crate::{RustBustersServer, StatsManager};
use common_utils::HostEvent;
use common_utils::{PacketHeader, PacketTypeHeader};
use log::{info, warn};
use wg_2024::packet::NackType;
use wg_2024::packet::NackType::Dropped;

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
                        self.topology.remove(&drone_id);
                        self.launch_network_discovery();
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
