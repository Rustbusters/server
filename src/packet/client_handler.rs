use crate::RustBustersServer;
use common_utils::HostEvent::{ControllerShortcut, HostMessageReceived};
use common_utils::{
    ClientToServerMessage, HostMessage, MessageBody, MessageContent, ServerToClientMessage, User,
};
use uuid::Uuid;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Ack, Fragment, Packet, PacketType};

use crate::db::DbMessage;

impl RustBustersServer {
    /// Handle the user registration
    ///
    /// Insert the user id in the active_users map and send the corresponding message back
    pub(crate) fn handle_register_user(&mut self, src_id: NodeId, name: &str) {
        println!("Server {} - Received RegisterUser", self.id);
        // Verify the user presence in the hashset
        if !self.active_users.contains_key(&src_id) {
            // Newly inserted
            self.active_users.insert(src_id, name.to_string());
            // Send Registration Success message
            self.send_message(
                src_id,
                HostMessage::FromServer(ServerToClientMessage::RegistrationSuccess),
            );

            // TODO: remove later, for test purposes only
            // DB OPERATION
            // Save message to local database
            // if let Ok(db_manager) = &self.db_manager {
            //     println!("Inserting in db");
            //     db_manager.insert(DbMessage::new(src_id, 4, "Hello".to_string()));
            // }

            // Cloning because of borrow checker issues
            let other_users = self.active_users.clone();
            other_users
                .iter()
                .filter(|(id, _)| *id != &src_id)
                .for_each(|(id, name)| {
                    self.send_message(
                        src_id,
                        HostMessage::FromServer(ServerToClientMessage::NewUserRegistered {
                            id: id.clone(),
                            name: name.clone(),
                        }),
                    );
                });
        } else {
            // Already exists
            // Send Registration Failure message
            self.send_message(
                src_id,
                HostMessage::FromServer(ServerToClientMessage::RegistrationFailure),
            );
        }
    }

    /// Handle the user unregistration
    ///
    /// Remove the user id from the active_users map and send the corresponding message back
    pub(crate) fn handle_unregister_user(&mut self, src_id: NodeId) {
        // Verify the user presence in the hashset
        if self.active_users.contains_key(&src_id) {
            // newly inserted
            self.active_users.remove(&src_id);
            // Send Unregistration Success message
            self.send_message(
                src_id,
                HostMessage::FromServer(ServerToClientMessage::UnregisterSuccess),
            );

            // Cloning because of borrow checker issues
            let other_users = self.active_users.clone();
            other_users
                .iter()
                .filter(|(id, _)| *id != &src_id)
                .for_each(|(id, name)| {
                    self.send_message(
                        src_id,
                        HostMessage::FromServer(ServerToClientMessage::UserUnregistered {
                            id: id.clone(),
                        }),
                    );
                });
        } else {
            // already exist
            // Send Unregsiteration Failure message
            self.send_message(
                src_id,
                HostMessage::FromServer(ServerToClientMessage::UnregisterFailure),
            );
        }
    }

    /// Handle the request for the list of active users
    ///
    /// Send the list of active users to the requester
    pub(crate) fn handle_request_active_users(&mut self, src_id: NodeId) {
        // Send the list of active users
        let active_users: Vec<User> = self
            .active_users
            .iter()
            .map(|(id, name)| User::new(id.clone(), name.to_string()))
            .collect();
        self.send_message(
            src_id,
            HostMessage::FromServer(ServerToClientMessage::ActiveUsersList {
                users: active_users,
            }),
        );
    }

    /// Handle the private message
    ///
    /// Send the message to the recipient
    pub(crate) fn handle_send_private_message(
        &mut self,
        src_id: NodeId,
        dest_id: NodeId,
        message: &MessageBody,
    ) {
        // Check if the recipient is an active user
        if !self.active_users.contains_key(&dest_id) {
            self.send_message(
                dest_id,
                HostMessage::FromServer(ServerToClientMessage::UserNotFound { user_id: dest_id }),
            );
            return;
        }
        // Send the message to the recipient
        self.send_message(
            dest_id,
            HostMessage::FromServer(ServerToClientMessage::PrivateMessage {
                sender_id: src_id,
                message: message.clone(),
            }),
        );
        // Save message to local database
        if let Ok(db_manager) = &self.db_manager {
            println!("Inserting in db");
            let message_content = match message.content.clone() {
                MessageContent::Text(text) => text,
                MessageContent::Image(image) => image,
                _ => "".to_string(),
            };
            db_manager.insert(DbMessage::new(src_id, dest_id, message_content));
        }
    }
}
