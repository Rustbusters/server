use crossbeam_channel::Sender;
use log::warn;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
use common_utils::HostMessage;
use crate::RustBustersServer;
use common_utils::Stats;
use common_utils::{HostEvent, HostCommand};

impl RustBustersServer {
    pub(crate) fn handle_command(&mut self, command: HostCommand) {
        todo!();
        match command {
            _ => {}
        }
    }
}
