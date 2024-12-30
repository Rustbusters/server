use crate::server::RustBustersServer;
use log::{info, warn};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Ack, Fragment, Packet, PacketType};
use common_utils::HostEvent::{ControllerShortcut, HostMessageReceived};

impl RustBustersServer {
    pub(crate) fn handle_message_fragment(
        &mut self,
        fragment: Fragment,
        session_id: u64,
        source_routing_header: SourceRoutingHeader,
    ) {
        // Update stats
        self.stats.inc_fragments_received();

        // If after insert all fragments of the session are received, reassemble the message
        if self.set_pending(session_id, fragment.clone()) {
            match self.reassemble_fragments(session_id) {
                Ok(msg) => {
                    info!(
                        "Node {}: Received full message {:?} of session {}",
                        self.id, msg, session_id
                    );
                    self.stats.inc_messages_received();
                    if let Err(err) = self.controller_send.send(HostMessageReceived(msg)) {
                        warn!("Node {}: Unable to send MessageReceived(...) to controller: {}", self.id, err);
                    }
                }
                Err(err) => {
                    warn!("Node {}: {}", self.id, err);
                }
            }
        }

        // Echo mode
        let fragment_index = fragment.fragment_index;
        // Send an Acknowledgment
        let ack = Ack { fragment_index };

        let ack_packet = Packet {
            pack_type: PacketType::Ack(ack),
            routing_header: SourceRoutingHeader {
                hop_index: 1,
                hops: source_routing_header
                    .hops
                    .iter()
                    .rev()
                    .cloned()
                    .collect::<Vec<NodeId>>(),
            },
            session_id,
        };

        // Send the Acknowledgment back to the sender
        let next_hop = ack_packet.routing_header.hops[1];

        if let Some(sender) = self.packet_send.get(&next_hop) {
            if let Err(err) = sender.send(ack_packet.clone()){
                warn!("Node {}: Error sending Ack for fragment {} to {}: {}", self.id, fragment_index, next_hop, err);
                self.send_to_sc(ControllerShortcut(ack_packet));
                info!("Node {}: Sending ack through SC", self.id);
            } else {
                // Increment the number of sent Acks
                self.stats.inc_acks_sent();
                info!(
                    "Node {}: Sent Ack for fragment {} to {}",
                    self.id, fragment_index, next_hop
                );
            }
        } else {
            warn!(
                "Node {}: Cannot send Ack for fragment {} to {}",
                self.id, fragment_index, next_hop
            );
            self.send_to_sc(ControllerShortcut(ack_packet))
        }
    }

    /// Insert the fragment in the pending_received map at index fragment_index.
    /// 
    /// Return true if all fragments have been received
    fn set_pending(&mut self, session_id: u64, fragment: Fragment) -> bool {
        let fragment_index = fragment.fragment_index;
        let total_n_fragments = fragment.total_n_fragments;
        // insert the fragment in the pending_received map at index fragment_index
        let fragments = vec![None; fragment.total_n_fragments as usize];
        let entry = self.pending_received.entry(session_id).or_insert((fragments, 0));
        entry.0[fragment_index as usize] = Some(fragment);
        entry.1 += 1;
        
        entry.1 == total_n_fragments
    }
}
