use common_utils::HostMessage;
use crate::RustBustersServer;

impl RustBustersServer {
    pub(crate) fn reassemble_fragments(&mut self, session_id: u64) -> Result<HostMessage, String> {
        match self.pending_received.remove(&session_id) {
            None => Err(format!("No fragments for session {}", session_id)),
            Some(fragments) => {
                let concatenated: Result<Vec<u8>, &str> =
                    fragments.0
                        .into_iter()
                        .try_fold(Vec::new(), |mut acc, f| match f {
                            Some(frammento) => {
                                acc.extend_from_slice(&frammento.data);
                                Ok(acc)
                            }
                            None => Err("Frammento mancante"),
                        });
                
                if let Ok(byte_array) = concatenated {
                    // Trova la lunghezza effettiva della stringa (fino al primo zero)
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
