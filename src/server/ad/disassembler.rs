use crate::RustBustersServer;
use common_utils::HostMessage;
use wg_2024::packet::{Fragment, FRAGMENT_DSIZE};

impl RustBustersServer {
    /// Splits a `HostMessage` into multiple fragments for transmission.
    ///
    /// # Parameters
    /// - `message: &HostMessage` – The message to be fragmented.
    ///
    /// # Returns
    /// - `Vec<Fragment>`: A vector containing all the fragments of the message. Each fragment is a `Fragment` structure containing:
    ///   - `fragment_index`: The index of the fragment (starting from 0).
    ///   - `total_n_fragments`: The total number of fragments that will represent the message.
    ///   - `length`: The actual length of the data in the fragment.
    ///   - `data`: The chunk of data in the fragment, padded to the size of `FRAGMENT_DSIZE` if necessary.
    ///
    /// # Behavior
    /// - Serializes the provided `HostMessage` into a JSON string and converts it to bytes.
    /// - Breaks the byte data into chunks, each of size `FRAGMENT_DSIZE` or less.
    /// - Creates a `Fragment` for each chunk, which includes:
    ///   - The fragment's index and the total number of fragments.
    ///   - The chunk's data and its length.
    /// - Collects all fragments and returns them as a vector.
    pub(crate) fn disassemble_message(&self, message: &HostMessage) -> Vec<Fragment> {
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
