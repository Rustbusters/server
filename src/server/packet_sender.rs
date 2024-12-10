use crate::node::messages::Message;
use crate::node::SimpleHost;
use log::{debug, info};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Packet, PacketType};
use crate::commands::HostEvent;

impl SimpleHost {
    pub(crate) fn send_random_message(&mut self, destination_id: NodeId) {
        // Compute the route to the destination
        if let Some(route) = self.compute_route(destination_id) {
            // Increment session_id_counter
            self.session_id_counter += 1;
            let session_id = self.session_id_counter;

            // // Serialize and fragment the message
            let message = Message::Custom(
                format!("Hello from {} with session {}. This is a random message. Bla Bla Bla Things\
                 to make the message longer for testing purposes and see if and how fragmentation \
                 works. I hope it works. QuackableQuackableQuackableQuackableQuackableQuackableQuackable\
                 QuackableQuackableQuackableQuackableQuackable", self.id, session_id
                )
            );
            let fragments = self.disassemble_message(message.clone());

            // Send the fragments along the route
            for fragment in fragments {
                debug!(
                    "Node {}: Sending fragment {:?} of session {} to {}",
                    self.id, fragment, session_id, destination_id
                );
                let fragment_index = fragment.fragment_index;
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
                    // TODO: in indiv. contr., better handling of send errors
                    let _ = sender.send(packet.clone());
                    self.pending_sent
                        .entry((session_id, fragment_index))
                        .or_insert(packet);
                    self.stats.inc_fragments_sent();
                }
            }
            self.stats.inc_messages_sent();
            let _ = self.controller_send.send(HostEvent::MessageSent(message));

            info!(
                "Node {}: Sent message to {} via route {:?}",
                self.id, destination_id, route
            );
        } else {
            info!("Node {}: No route to {}", self.id, destination_id);
        }
    }
}
