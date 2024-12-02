use crate::node::SimpleHost;
use log::info;

impl SimpleHost {
    pub(crate) fn handle_ack(&mut self, session_id: u64, fragment_index: u64) {
        self.stats.inc_acks_received();
        self.pending_sent.remove(&(session_id, fragment_index));
        
        // Check if all fragments with key (session_id, _) have been acked
        if self.pending_sent.iter().filter(|((key, _), _)| *key == session_id).collect::<Vec<_>>().is_empty() {
            info!("Node {}: All fragments of session {} acked", self.id, session_id);
            self.stats.inc_messages_sent();
        }
    }
}
