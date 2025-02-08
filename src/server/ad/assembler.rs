use crate::RustBustersServer;
use common_utils::HostMessage;

impl RustBustersServer {
    /// Reassembles message fragments into a complete `HostMessage` using the session's fragments.
    ///
    /// ### Parameters
    /// - `session_id: u64` – The identifier of the session for which fragments need to be reassembled.
    ///
    /// ### Returns
    /// - `Result<HostMessage, String>`:
    ///   - `Ok(HostMessage)` if the fragments are successfully reassembled and deserialized into a valid `HostMessage`.
    ///   - `Err(String)` containing an error message if any issue occurs during reassembly or deserialization.
    ///
    /// ### Behavior
    /// - Retrieves the fragments associated with the given `session_id` from the `pending_received` map.
    /// - Attempts to concatenate the byte data from each fragment into a complete byte array.
    /// - If all fragments are successfully concatenated:
    ///   - Trims the byte array up to the first null byte (0) to determine the message's actual length.
    ///   - Attempts to convert the byte array into a UTF-8 string.
    ///   - If successful, it deserializes the string into a `HostMessage` using `serde_json`.
    /// - Returns an error if any fragment is missing, concatenation fails, UTF-8 conversion fails, or deserialization fails.
    pub(crate) fn reassemble_fragments(&mut self, session_id: u64) -> Result<HostMessage, String> {
        match self.pending_received.remove(&session_id) {
            None => Err(format!("No fragments for session {}", session_id)),
            Some(fragments) => {
                let concatenated: Result<Vec<u8>, &str> =
                    fragments
                        .0
                        .into_iter()
                        .try_fold(Vec::new(), |mut acc, f| match f {
                            Some(fragment) => {
                                acc.extend_from_slice(&fragment.data);
                                Ok(acc)
                            }
                            None => Err("Missing fragment"),
                        });

                if let Ok(byte_array) = concatenated {
                    // Find the actual string length (till the first 0)
                    let len = byte_array
                        .iter()
                        .position(|&x| x == 0)
                        .unwrap_or(byte_array.len());

                    // Converti l'array di byte in una stringa
                    let serialized_str = std::str::from_utf8(&byte_array[..len]);
                    let serialized_str = match serialized_str {
                        Ok(s) => s,
                        Err(_) => return Err("Error in JSON string conversion".to_string()),
                    };

                    if let Ok(msg) = serde_json::from_str(serialized_str) {
                        Ok(msg)
                    } else {
                        Err("Error in deserialization".to_string())
                    }
                } else {
                    Err("Error in reassembly".to_string())
                }
            }
        }
    }
}
