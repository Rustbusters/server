use crate::SimpleHost;
use log::info;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Packet, PacketType};

impl SimpleHost {
    pub(crate) fn send_random_message(&mut self, destination_id: NodeId) {
        // Compute the route to the destination
        if let Some(route) = self.compute_route(destination_id) {
            // Increment session_id_counter
            self.session_id_counter += 1;
            let session_id = self.session_id_counter;

            // Serialize and fragment the message
            let fragments = self.generate_random_fragments(/*&message_data*/);

            // Send the fragments along the route
            for fragment in fragments {
                let packet = Packet {
                    pack_type: PacketType::MsgFragment(fragment),
                    routing_header: SourceRoutingHeader {
                        hop_index: 1,
                        hops: route.clone(),
                    },
                    session_id,
                };

                // Send the packet to the first hop
                let next_hop = packet.routing_header.hops[1];
                if let Some(sender) = self.packet_send.get(&next_hop) {
                    let _ = sender.send(packet);
                    self.stats.inc_fragments_sent();
                }
            }
            self.stats.inc_messages_sent();

            info!(
                "Node {}: Sent message to {} via route {:?}",
                self.id, destination_id, route
            );
        } else {
            info!("Node {}: No route to {}", self.id, destination_id);
        }
    }
}
