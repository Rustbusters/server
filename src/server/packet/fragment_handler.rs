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
    /// Handles an incoming message fragment, reassembles the message if all fragments are received,
    /// and sends an acknowledgment (ACK) back to the sender.
    ///
    /// ### Parameters
    /// - `fragment: Fragment` – The received fragment containing part of a larger message.
    /// - `session_id: u64` – The session identifier to which this fragment belongs.
    /// - `source_routing_header: SourceRoutingHeader` – The routing information for the fragment.
    ///
    /// ### Behavior
    /// 1. Stores the received fragment in `pending_received`.
    /// 2. If all fragments for `session_id` are received:
    ///    - Reassembles the message.
    ///    - If it's a `FromClient` message, handles it based on the message type:
    ///      - Registers/unregisters users.
    ///      - Retrieves the active user list.
    ///      - Sends private messages.
    ///    - Logs the successful message reception.
    ///    - Updates message reception statistics.
    /// 3. If reassembly fails, logs an error.
    /// 4. Sends an acknowledgment (ACK) for the received fragment:
    ///    - Constructs an ACK packet with a reversed routing path.
    ///    - Attempts to send the ACK to the sender.
    ///    - If sending fails, notifies the simulation controller.
    ///    - Updates ACK sending statistics.
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
        // Construct the Ack packet
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

        // Send the Ack back to the sender
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

    /// Inserts the received fragment into `pending_received`.
    ///
    /// ### Parameters
    /// - `session_id: u64` – The session identifier associated with the fragment.
    /// - `fragment: Fragment` – The received fragment to be stored.
    ///
    /// ### Returns
    /// - `true` if all fragments for the session have been received, allowing reassembly.
    /// - `false` otherwise.
    ///
    /// ### Behavior
    /// - Stores the fragment at its corresponding index in `pending_received`.
    /// - Increments the count of received fragments.
    /// - If all fragments are received, signals readiness for reassembly.
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
