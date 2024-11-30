use wg_2024::network::NodeId;
use crate::node::SimpleHost;

#[derive(Debug, Clone)]
pub enum HostCommand {
    SendRandomMessage(NodeId),
    DiscoverNetwork,
    EnableEchoMode,
    DisableEchoMode,
    EnableAutoSend(u64),
    DisableAutoSend,
}

impl SimpleHost {
    pub(crate) fn handle_command(&mut self, command: HostCommand) {
        match command {
            HostCommand::SendRandomMessage(dest) => {
                self.send_random_message(dest);
            }
            HostCommand::DiscoverNetwork => {
                self.discover_network();
            },
            HostCommand::EnableEchoMode => {
                self.echo_mode_on();
            },
            HostCommand::DisableEchoMode => {
                self.echo_mode_off();
            },
            HostCommand::EnableAutoSend(interval) => {
                self.auto_send_on(interval);
            },
            HostCommand::DisableAutoSend => {
                self.auto_send_off();
            },
        }
    }
}