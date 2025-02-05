use std::time::Instant;

use crate::{RustBustersServer, StatsManager};
use common_utils::{HostEvent, HostMessage};
use log::info;

impl RustBustersServer {
    pub(crate) fn handle_ack(&mut self, session_id: u64, fragment_index: u64) {
        // Remove the acked fragment from the pending_sent list
        self.pending_sent.remove(&(session_id, fragment_index));

        // Check if all fragments with key (session_id, _) have been acked
        if self
            .pending_sent
            .iter()
            .filter(|((key, _), _)| *key == session_id)
            .collect::<Vec<_>>()
            .is_empty()
        {
            // Sending host message sent to simulation controller
            let (host_message, dest_id) = self
                .sessions_messages
                .remove(&session_id)
                .expect("No session message found for specified session_id");
            let start = self
                .sessions_start_instants
                .remove(&session_id)
                .expect("No session instant found for specified session_id");
            let delay = Instant::now() - start;
            self.send_to_sc(HostEvent::HostMessageSent(dest_id, host_message, delay));

            info!(
                "Server {}: All fragments of session {} acked",
                self.id, session_id
            );
        }
    }
}
