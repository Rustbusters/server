use crate::{RustBustersServerController, StatsManager};
use common_utils::{HostCommand, HostEvent};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use wg_2024::network::NodeId;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};

use common_utils::{HostMessage, ServerToClientMessage, Stats, User};

use std::collections::HashSet;

use crate::RustBustersServer;

impl RustBustersServer {
    pub(crate) fn send_message(&mut self, destination_id: NodeId, message: HostMessage) {
        // Find route to destination
        if let Some(route) = self.find_route(destination_id) {
            // Disassemble the message
            let fragments = self.disassemble_message(&message);

            self.session_id_counter += 1;
            let session_id = self.session_id_counter;

            // Send the fragments along the route
            for fragment in fragments {
                debug!(
                    "Server {}: Sending fragment {} of session {} to Client {}",
                    self.id, fragment.fragment_index, session_id, destination_id
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
                    if let Err(e) = sender.send(packet.clone()) {
                        warn!(
                            "Server {}: Failed to send packet to {}: {:?}",
                            self.id, next_hop, e
                        );
                        let error_msg = ServerToClientMessage::SendingError {
                            error: "Failed to send message! Retry in a few seconds".to_string(),
                            message: match message.clone() {
                                HostMessage::FromClient(client_msg) => client_msg,
                                _ => return,
                            },
                        };
                    }
                    self.pending_sent
                        .entry((session_id, fragment_index))
                        .or_insert(packet);
                    StatsManager::inc_fragments_sent(self.id);
                }
            }
            StatsManager::inc_messages_sent(self.id);
            let _ = self
                .controller_send
                .send(HostEvent::HostMessageSent(message));

            info!(
                "Server {}: Sent message to {} via route {:?}",
                self.id, destination_id, route
            );
        } else {
            error!(
                "Server {}: Unable to find route to {}",
                self.id, destination_id
            );
        }
    }
}
