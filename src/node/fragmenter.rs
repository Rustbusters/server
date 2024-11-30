use crate::node::SimpleHost;
use rand::{random, rng, Rng};
use wg_2024::packet::Fragment;

impl SimpleHost {
    pub(crate) fn generate_random_fragments(&self) -> Vec<Fragment> {
        let serialized_data: Vec<u8> = (0..rng().random_range(100..200))
            .map(|_| random())
            .collect();

        // Fragment the data into chunks of 80 bytes
        let chunk_size = 80;
        let total_size = serialized_data.len();
        let total_n_fragments = ((total_size + chunk_size - 1) / chunk_size) as u64;

        let mut fragments = Vec::new();

        for (i, chunk) in serialized_data.chunks(chunk_size).enumerate() {
            let mut data_array = [0u8; 80];
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
