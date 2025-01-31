use wg_2024::config::Server;
use wg_2024::network::NodeId;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use common_utils::Stats;

static STATS: LazyLock<Mutex<HashMap<NodeId, Stats>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Wrapper for safely interacting with the global stats
pub struct StatsManager;

impl StatsManager {
    pub fn get_or_create_stats(server_id: NodeId) -> Stats {
        let mut stats = STATS.lock().unwrap();
        stats.entry(server_id).or_insert_with(Stats::new).clone()
    }

    pub fn inc_messages_sent(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_messages_sent();
    }

    pub fn inc_fragments_sent(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_fragments_sent();
    }

    pub fn inc_messages_received(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_messages_received();
    }

    pub fn inc_fragments_received(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_fragments_received();
    }

    pub fn inc_acks_sent(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_acks_sent();
    }

    pub fn inc_acks_received(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_acks_received();
    }

    pub fn inc_nacks_received(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_nacks_received();
    }

    pub fn get_stats(server_id: NodeId) -> Stats {
        let stats = STATS.lock().unwrap();
        stats.get(&server_id).cloned().unwrap_or_default()
    }
}
