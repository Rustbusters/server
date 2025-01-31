use crate::{RustBustersServer, StatsManager};
use common_utils::Stats;
use common_utils::{
    HostCommand, HostEvent, HostMessage, MessageBody, MessageContent, ServerToClientMessage,
};
use crossbeam_channel::Sender;
use log::warn;
use tiny_http::Server;
use url::Host;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

impl RustBustersServer {
    pub(crate) fn handle_command(&mut self, command: HostCommand) {
        match command {
            HostCommand::SendRandomMessage(dest_id) => {
                self.send_message(
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
                self.discover_network();
                warn!("Server {}: Network Discovery initiated", self.id);
            }
            HostCommand::StatsRequest => {
                let stats = StatsManager::get_or_create_stats(self.id);
                if let Err(err) = self.controller_send.send(HostEvent::StatsResponse(stats)) {
                    warn!(
                        "Server {}: Unable to send StatsResponse(...) to simulation controller: {}",
                        self.id, err
                    );
                }
                warn!(
                    "Server {}: StatsResponse sent to simulation controller",
                    self.id
                );
            }
            HostCommand::AddSender(sender_id, sender) => {
                self.packet_send.insert(sender_id, sender);
                self.discover_network();
                warn!("Server {}: Sender added", self.id);
            }
            HostCommand::RemoveSender(sender_id) => {
                self.packet_send.remove(&sender_id);
                self.discover_network();
                warn!("Server {}: Sender removed", self.id);
            }
            _ => {}
        }
    }
}
