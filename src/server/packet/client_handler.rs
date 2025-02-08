use crate::{InternalChannelsManager, RustBustersServer};
use common_utils::{
    ClientToServerMessage, HostMessage, MessageBody, MessageContent, ServerToClientMessage, User,
};
use uuid::Uuid;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Ack, Fragment, Packet, PacketType};

use crate::server::db::DbMessage;

impl RustBustersServer {
    /// Handles user registration
    ///
    /// ### Behavior
    /// 1. Inserts the `src_id` in the `active_users` map and sends the corresponding message back.
    /// 2. Notifies the other active users connected to the server.
    pub(crate) fn handle_register_user(&mut self, src_id: NodeId, name: &str) {
        // Verify the user presence in the hashset
        if !self.active_users.contains_key(&src_id) {
            // Name already taken
            if self.active_users.values().any(|n| n == name) {
                self.send_network_message(
                    src_id,
                    HostMessage::FromServer(ServerToClientMessage::RegistrationFailure {
                        reason: "Name already taken".to_string(),
                    }),
                );
                return;
            }

            // Newly inserted
            self.active_users.insert(src_id, name.to_string());
            // Send Registration Success message
            self.send_network_message(
                src_id,
                HostMessage::FromServer(ServerToClientMessage::RegistrationSuccess),
            );

            // Send active users to Internal Channels for retransmission to WebSoket Server and finally to the UI
            self.send_active_users();

            // Cloning because of borrow checker issues
            let other_users = self.active_users.clone();
            other_users.iter().filter(|(&id, _)| id != src_id).for_each(
                |(&other_id, other_name)| {
                    self.send_network_message(
                        other_id,
                        HostMessage::FromServer(ServerToClientMessage::NewUserRegistered {
                            id: src_id,
                            name: name.to_string(),
                        }),
                    );
                },
            );
        } else {
            // Already exists
            // Send Registration Failure message
            self.send_network_message(
                src_id,
                HostMessage::FromServer(ServerToClientMessage::RegistrationFailure {
                    reason: "You are already registered".to_string(),
                }),
            );
        }
    }

    /// Handles the user unregistration
    ///
    /// ### Behavior
    /// 1. Removes the `src_id` from the `active_users` map and sends the corresponding message back.
    /// 2. Notifies the other active users connected to the server.
    pub(crate) fn handle_unregister_user(&mut self, src_id: NodeId) {
        // Verify the user presence in the hashset
        if self.active_users.contains_key(&src_id) {
            self.active_users.remove(&src_id);

            // Send Unregistration Success message
            self.send_network_message(
                src_id,
                HostMessage::FromServer(ServerToClientMessage::UnregisterSuccess),
            );

            // Send active users to Internal Channels for retransmission to WebSoket client and finally to the UI
            self.send_active_users();

            // Cloning because of borrow checker issues
            let other_users = self.active_users.clone();
            other_users.iter().for_each(|(&other_id, other_name)| {
                self.send_network_message(
                    other_id,
                    HostMessage::FromServer(ServerToClientMessage::UserUnregistered { id: src_id }),
                );
            });
        } else {
            // Already unregistered
            // Send Unregsiteration Failure message
            self.send_network_message(
                src_id,
                HostMessage::FromServer(ServerToClientMessage::UnregisterFailure {
                    reason: "You are already unregistered".to_string(),
                }),
            );
        }
    }

    /// Handles request for the list of active users
    ///
    /// ### Behavior
    /// 1. Sends the list of active users to the requester.
    /// 2. Sends the list of active users through the Internal Channels to the UI
    pub(crate) fn handle_request_active_users(&mut self, src_id: NodeId) {
        // Send the list of active users
        self.send_network_message(
            src_id,
            HostMessage::FromServer(ServerToClientMessage::ActiveUsersList {
                users: self.get_active_users(),
            }),
        );

        // Send active users to Internal Channels for retransmission to WebSoket Server
        self.send_active_users();
    }

    /// Handles private message receipt
    ///
    /// ### Behavior
    /// 1. Verifies if the `src_id` and `dest_id` are registered.
    /// 2. Sends the message to the recipient.
    /// 3. Save the received message to the local database via the `db_manager`.
    pub(crate) fn handle_send_private_message(
        &mut self,
        src_id: NodeId,
        dest_id: NodeId,
        message: &MessageBody,
    ) {
        // Check if the sender is an active user
        if !self.active_users.contains_key(&src_id) {
            self.send_network_message(
                src_id,
                HostMessage::FromServer(ServerToClientMessage::UserNotFound { user_id: src_id }),
            );
            return;
        }

        // Check if the recipient is an active user
        if !self.active_users.contains_key(&dest_id) {
            self.send_network_message(
                src_id,
                HostMessage::FromServer(ServerToClientMessage::UserNotFound { user_id: dest_id }),
            );
            return;
        }
        // Send the message to the recipient
        self.send_network_message(
            dest_id,
            HostMessage::FromServer(ServerToClientMessage::PrivateMessage {
                sender_id: src_id,
                message: message.clone(),
            }),
        );
        // Save message to local database
        if let Ok(db_manager) = &self.db_manager {
            let message_str = match message.content.clone() {
                MessageContent::Text(text) => text,
                MessageContent::Image(image) => image,
                _ => "".to_string(),
            };
            if let Ok(new_db_message) = db_manager.insert(src_id, dest_id, message_str) {
                self.send_db_message(new_db_message);
            }
        }
    }
}
