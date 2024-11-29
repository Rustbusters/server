use crate::SimpleHost;
use log::info;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Ack, Fragment, Packet, PacketType};

impl SimpleHost {
    pub(crate) fn handle_message_fragment(
        &mut self,
        session_id: u64,
        fragment: Fragment,
        source_routing_header: SourceRoutingHeader,
    ) {
        // Handle incoming message fragments (reassembly not implemented for simplicity)
        info!(
            "Node {}: Received fragment {} of session {}",
            self.id, fragment.fragment_index, session_id
        );

        // Increment the number of received fragments
        self.stats.inc_fragments_received();
        
        // TODO: count of full messages received
        // (può servire una variabile per mantenere eventuali pacchetti in pending all'arrivo di nacks)

        // Send an Acknowledgment
        let ack = Ack {
            fragment_index: fragment.fragment_index,
        };

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
            let _ = sender.send(ack_packet);
            
            // Increment the number of sent Acks
            self.stats.inc_acks_sent()
        }

        info!(
            "Node {}: Sent Ack for fragment {} to {}",
            self.id, fragment.fragment_index, next_hop
        );
    }
}
