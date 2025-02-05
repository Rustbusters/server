use crate::RustBustersServer;
use crate::StatsManager;
use common_utils::HostEvent;
use common_utils::{
    ClientToServerMessage, HostMessage, MessageBody, PacketHeader, PacketTypeHeader,
    ServerToClientMessage, User,
};
use log::{info, warn};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Ack, Fragment, Packet, PacketType};

impl RustBustersServer {
    pub(crate) fn handle_fragment(
        &mut self,
        fragment: Fragment,
        session_id: u64,
        source_routing_header: SourceRoutingHeader,
    ) {
        // If after insert all fragments of the session are received, reassemble the message
        if self.set_pending(session_id, fragment.clone()) {
            match self.reassemble_fragments(session_id) {
                Ok(msg) => {
                    if let HostMessage::FromClient(client_to_server_msg) = &msg {
                        let src_id = source_routing_header
                            .source()
                            .expect("No current hop in source routing header");
                        // Handle the various types of Client to Server messages
                        match client_to_server_msg {
                            ClientToServerMessage::RegisterUser { name } => {
                                self.handle_register_user(src_id, name)
                            }
                            ClientToServerMessage::UnregisterUser => {
                                self.handle_unregister_user(src_id)
                            }
                            ClientToServerMessage::RequestActiveUsers => {
                                self.handle_request_active_users(src_id)
                            }
                            ClientToServerMessage::SendPrivateMessage {
                                recipient_id,
                                message,
                            } => self.handle_send_private_message(src_id, *recipient_id, message),
                            _ => {}
                        }
                    }
                    info!(
                        "Server {}: Received full message {:?} of session {}",
                        self.id, msg, session_id
                    );
                    StatsManager::inc_messages_received(self.id);
                }
                Err(err) => {
                    warn!("Server {} failed to reassemble fragments: {}", self.id, err);
                }
            }
        }

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
            if let Err(err) = sender.send(ack_packet.clone()) {
                warn!(
                    "Server {}: Error sending Ack for fragment {} to {}: {}",
                    self.id, fragment_index, next_hop, err
                );
                self.send_to_sc(HostEvent::ControllerShortcut(ack_packet.clone()));
                info!("Server {}: Sending ack through SC", self.id);
            } else {
                // Update stats: increment the number of sent Acks
                StatsManager::inc_acks_sent(self.id);
                info!(
                    "Server {}: Sent Ack for fragment {} to {}",
                    self.id, fragment_index, next_hop
                );
            }
        } else {
            warn!(
                "Server {}: Cannot send Ack for fragment {} to {}",
                self.id, fragment_index, next_hop
            );
            self.send_to_sc(HostEvent::ControllerShortcut(ack_packet.clone()))
        }

        // Send Ack to Simulation Controller
        self.send_to_sc(HostEvent::PacketSent(PacketHeader {
            session_id,
            pack_type: PacketTypeHeader::Ack,
            routing_header: ack_packet.routing_header,
        }));
    }

    /// Insert the fragment in the pending_received map at index fragment_index.
    ///
    /// Return true if all fragments have been received
    fn set_pending(&mut self, session_id: u64, fragment: Fragment) -> bool {
        let fragment_index = fragment.fragment_index;
        let total_n_fragments = fragment.total_n_fragments;
        // insert the fragment in the pending_received map at index fragment_index
        let fragments = vec![None; fragment.total_n_fragments as usize];
        let entry = self
            .pending_received
            .entry(session_id)
            .or_insert((fragments, 0));
        entry.0[fragment_index as usize] = Some(fragment);
        entry.1 += 1;

        entry.1 == total_n_fragments
    }
}
