use std::time::Instant;

use crate::{RustBustersServer, StatsManager};
use common_utils::{HostEvent, HostMessage};
use log::{info, warn};

impl RustBustersServer {
    /// Handles an Acknowledgment (ACK) packet for a sent message fragment.
    ///
    /// ### Parameters
    /// - `session_id: u64` – The unique identifier for the session to which the acknowledged fragment belongs.
    /// - `fragment_index: u64` – The index of the acknowledged fragment within the session.
    ///
    /// ### Behavior
    /// 1. Removes the acknowledged fragment `(session_id, fragment_index)` from `pending_sent`.
    /// 2. Checks if all fragments for `session_id` have been acknowledged.
    /// 3. If all fragments are acknowledged:
    ///    - Removes session information from `sessions_info` map.
    ///    - Computes the delay since the session started.
    ///    - Sends a `HostMessageSent` event to the simulation controller.
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
            if let Some((dest_id, start, host_message)) = self.sessions_info.remove(&session_id) {
                let delay = Instant::now() - start;
                self.send_to_sc(HostEvent::HostMessageSent(dest_id, host_message, delay));

                info!(
                    "Server {}: All fragments of session {} acked",
                    self.id, session_id
                );
            } else {
                warn!(
                    "Server {}: Cannot find session_info for specified session_id {}",
                    self.id, session_id
                );
            }
        }
    }
}
