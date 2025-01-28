use crate::RustBustersServer;
use log::{info, warn};
use wg_2024::packet::NackType;
use wg_2024::packet::NackType::Dropped;

impl RustBustersServer {
    pub(crate) fn handle_nack(
        &mut self,
        session_id: u64,
        fragment_index: u64,
        nack_type: NackType
    ) {
        // Update stats
        self.stats.inc_nacks_received();
        
        match self.pending_sent.get(&(session_id, fragment_index)){
            None => {
                warn!("Node {}: Nack for unknown fragment", self.id);
            }
            Some(packet) => {
                if let Dropped = nack_type {
                    info!("Node {}: Resending fragment {}", self.id, fragment_index);
                    // TODO: decide if the fragment and message counters should be incremented on resend, only on ack or always
                    if let Some(sender) = self.packet_send.get(&packet.routing_header.hops[1]) {
                        if let Err(err) = sender.send(packet.clone()){
                            warn!("Node {}: Unable to resend fragment {}: {}", self.id, fragment_index, err);
                        } else {
                            self.stats.inc_fragments_sent();
                        }
                    }
                }
                else {
                    match nack_type {
                        NackType::ErrorInRouting(_) => {
                            // TODO: implement this
                            unimplemented!("Node {}: Nack for fragment {} with type {:?}", self.id, fragment_index, nack_type);
                        }
                        NackType::DestinationIsDrone | NackType::UnexpectedRecipient(_) => {
                            warn!("Node {}: Nack for fragment {} with type {:?}", self.id, fragment_index, nack_type);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
