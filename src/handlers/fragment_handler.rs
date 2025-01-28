use crate::RustBustersServer;
use log::{info, warn};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Ack, Fragment, Packet, PacketType};
use common_utils::HostEvent::{ControllerShortcut, HostMessageReceived, };
use common_utils::{ClientToServerMessage, ServerToClientMessage, HostMessage, MessageBody, User};

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
                    if let HostMessage::FromClient(client_to_server_msg) = &msg {
                        let user_id = source_routing_header.current_hop().expect("No current hop in source routing header");
                        // Handle the various types of Client to Server messages
                        match client_to_server_msg {
                            ClientToServerMessage::RegisterUser {name} => self.handle_register_user(user_id, name),
                            ClientToServerMessage::UnregisterUser => self.handle_unregister_user(user_id),
                            ClientToServerMessage::RequestActiveUsers => self.handle_request_active_users(user_id),
                            ClientToServerMessage::SendPrivateMessage{ recipient_id, message } => self.handle_send_private_message(user_id, *recipient_id, message),
                            _ => {}
                        }
                    }
                    info!(
                        "Server {}: Received full message {:?} of session {}",
                        self.id, msg, session_id
                    );
                    self.stats.inc_messages_received();
                    if let Err(err) = self.controller_send.send(HostMessageReceived(msg)) {
                        warn!("Server {}: Unable to send MessageReceived(...) to controller: {}", self.id, err);
                    }
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

    /// Handle the user registration
    /// 
    /// Insert the user id in the active_users map and send the corresponding message back
    fn handle_register_user(&mut self, src_id: NodeId, name: &str) {
        // Verify the user presence in the hashset
        if !self.active_users.contains_key(&src_id) { // Newly inserted
            self.active_users.insert(src_id, name.to_string());
            // Send Registration Success message
            self.send_packet(src_id, HostMessage::FromServer(ServerToClientMessage::RegistrationSuccess));
            
            // Cloning because of borrow checker issues
            let other_users = self.active_users.clone();
            other_users.iter().filter(|(id, _)| *id != &src_id).for_each(|(id, name)| {
                self.send_packet(src_id, HostMessage::FromServer(ServerToClientMessage::NewUserRegistered { id: id.clone(), name: name.clone() }));
            });
        } else { // Already exists
            // Send Registration Failure message
            self.send_packet(src_id, HostMessage::FromServer(ServerToClientMessage::RegistrationFailure));
        }
    }

    /// Handle the user unregistration
    /// 
    /// Remove the user id from the active_users map and send the corresponding message back
    fn handle_unregister_user(&mut self, src_id: NodeId) {
        // Verify the user presence in the hashset
        if self.active_users.contains_key(&src_id) { // newly inserted
            self.active_users.remove(&src_id);
            // Send Unregistration Success message
            self.send_packet(src_id, HostMessage::FromServer(ServerToClientMessage::UnregisterSuccess));

            // Cloning because of borrow checker issues
            let other_users = self.active_users.clone();
            other_users.iter().filter(|(id, _)| *id != &src_id).for_each(|(id, name)| {
                self.send_packet(src_id, HostMessage::FromServer(ServerToClientMessage::UserUnregistered { id: id.clone() }));
            });
        } else { // already exist
            // Send Unregsiteration Failure message
            self.send_packet(src_id, HostMessage::FromServer(ServerToClientMessage::UnregisterFailure));
        }
    }

    /// Handle the request for the list of active users
    /// 
    /// Send the list of active users to the requester
    fn handle_request_active_users(&mut self, src_id: NodeId) {
        // Send the list of active users
        let active_users: Vec<User> = self.active_users.iter().map(|(id, name)| User::new(id.clone(), name.to_string()) ).collect();
        self.send_packet(src_id, HostMessage::FromServer(ServerToClientMessage::ActiveUsersList { users: active_users }));
    }

    /// Handle the private message
    /// 
    /// Send the message to the recipient
    fn handle_send_private_message(&mut self, src_id: NodeId, dest_id: NodeId, message: &MessageBody) {
        // Check if the recipient is an active user
        if !self.active_users.contains_key(&dest_id) {
            self.send_packet(dest_id, HostMessage::FromServer(ServerToClientMessage::UserNotFound { user_id: dest_id }));
            return;
        }
        // Send the message to the recipient
        self.send_packet(dest_id, HostMessage::FromServer(ServerToClientMessage::PrivateMessage { sender_id: src_id, message: message.clone() }));
    }

}
