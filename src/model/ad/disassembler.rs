use common_utils::HostMessage;
use crate::model::RustBustersServer;
use wg_2024::packet::{Fragment, FRAGMENT_DSIZE};

impl RustBustersServer {
    pub(crate) fn disassemble_message(&self, message: HostMessage) -> Vec<Fragment> {
        let serialized_str = serde_json::to_string(&message).unwrap();
        let bytes = serialized_str.as_bytes();
        
        // Fragment the data into chunks of FRAGMENT_DSIZE bytes
        let total_size = bytes.len();
        let total_n_fragments = ((total_size + FRAGMENT_DSIZE - 1) / FRAGMENT_DSIZE) as u64;
        
        let mut fragments = Vec::new();
        for (i, chunk) in bytes.chunks(FRAGMENT_DSIZE).enumerate() {
            let mut data_array = [0u8; FRAGMENT_DSIZE];
            let length = chunk.len();
            data_array[..length].copy_from_slice(chunk);
            
            let fragment = Fragment {
                fragment_index: i as u64,
                total_n_fragments,
                length: length as u8,
                data: data_array,
            };
            
            fragments.push(fragment);
        }
     
        fragments
    }
}
