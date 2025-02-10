use std::thread;
use std::time::Duration;

use crate::state::Stats;
use crate::{RustBustersServer, StatsManager};
use common_utils::{
    HostCommand, HostEvent, HostMessage, MessageBody, MessageContent, ServerToClientMessage,
};
use crossbeam_channel::Sender;
use log::{error, info, warn};
use tiny_http::Server;
use url::Host;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

impl RustBustersServer {
    pub(crate) fn send_to_sc(&self, event: HostEvent) {
        if self.controller_send.send(event).is_ok() {
            info!("Server {} - Sent HostEvent to SC", self.id);
        } else {
            error!("Server {} - Error in sending event to SC", self.id);
        }
    }

    /// Handles various simulation controller commands received from the control interface.
    ///
    /// # Parameters
    /// - `command: HostCommand` – The command to be processed, which could include actions like sending messages,
    ///   initiating network discovery, adding/removing senders, or stopping the server.
    ///
    /// # Behavior
    /// - Matches the `HostCommand` variant and performs the corresponding action:
    ///   - **SendRandomMessage**: Sends a random private message to a specified destination node.
    ///   - **DiscoverNetwork**: Initiates a network discovery process to learn about the network topology.
    ///   - **AddSender**: Adds a sender to the `packet_send` map and triggers a network discovery.
    ///   - **RemoveSender**: Removes a sender from the `packet_send` map and triggers a network discovery.
    ///   - **Stop**: Stops the server, sends a stop command to the `RustbusterServerController`, and marks the server as stopped.
    ///   - Other commands are ignored (default case).
    pub(crate) fn handle_command(&mut self, command: HostCommand) {
        match command {
            HostCommand::SendRandomMessage(dest_id) => {
                self.send_network_message(
                    dest_id,
                    HostMessage::FromServer(ServerToClientMessage::PrivateMessage {
                        sender_id: self.id,
                        message: MessageBody {
                            sender_id: self.id,
                            content: MessageContent::Text("Random message from Server".to_string()),
                            timestamp: "now".to_string(),
                        },
                    }),
                );
                warn!(
                    "Server {}: Random Private Message sent to {}",
                    self.id, dest_id
                );
            }
            HostCommand::DiscoverNetwork => {
                self.launch_network_discovery();
                warn!("Server {}: Network Discovery initiated", self.id);
            }
            HostCommand::AddSender(sender_id, sender) => {
                self.packet_send.insert(sender_id, sender);
                self.topology
                    .get_mut(&self.id)
                    .expect("Cannot unwrap topology")
                    .push(sender_id);
                self.launch_network_discovery();
                warn!("Server {}: Sender added", self.id);
            }
            HostCommand::RemoveSender(sender_id) => {
                self.packet_send.remove(&sender_id);
                self.topology
                    .get_mut(&self.id)
                    .expect("Cannot unwrap topology")
                    .retain(|&id| id != sender_id);
                self.launch_network_discovery();
                warn!("Server {}: Sender removed", self.id);
            }
            HostCommand::Stop => {
                // Sending stop command to RustbusterServerController
                println!("Stopping server");
                self.has_stopped = true;
                self.server_controller_sender.send(HostCommand::Stop);
                thread::sleep(Duration::from_millis(200));
            }
            _ => {}
        }
    }
}
