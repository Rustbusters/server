use crate::{RustBustersServer, StatsManager};
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
            info!(
                "Node {}: All fragments of session {} acked",
                self.id, session_id
            );
        }
    }
}
