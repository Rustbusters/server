use wg_2024::config::Server;
use wg_2024::network::NodeId;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    messages_sent: u64,
    messages_received: u64,

    message_fragments_sent: u64,
    message_fragments_received: u64,

    flood_requests_sent: u64,
    flood_requests_received: u64,

    flood_responses_sent: u64,
    flood_responses_received: u64,

    acks_sent: u64,
    acks_received: u64,

    nacks_received: u64,
}

// Setters
impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

impl Stats {
    pub fn new() -> Self {
        Self {
            messages_sent: 0,
            messages_received: 0,

            message_fragments_sent: 0,
            message_fragments_received: 0,

            flood_requests_sent: 0,
            flood_requests_received: 0,

            flood_responses_sent: 0,
            flood_responses_received: 0,

            acks_sent: 0,
            acks_received: 0,

            nacks_received: 0,
        }
    }
}

// Getters
impl Stats {
    // Messages
    pub fn inc_messages_sent(&mut self) {
        self.messages_sent += 1;
    }

    pub fn inc_messages_received(&mut self) {
        self.messages_received += 1;
    }

    // Fragments
    pub fn inc_message_fragments_sent(&mut self) {
        self.message_fragments_sent += 1;
    }

    pub fn inc_message_fragments_received(&mut self) {
        self.message_fragments_received += 1;
    }

    // Flooding
    // Requests
    pub fn inc_flood_requests_sent(&mut self) {
        self.flood_requests_sent += 1;
    }

    pub fn inc_flood_requests_received(&mut self) {
        self.flood_requests_received += 1;
    }

    // Responses
    pub fn inc_flood_responses_sent(&mut self) {
        self.flood_responses_sent += 1;
    }

    pub fn inc_flood_responses_received(&mut self) {
        self.flood_responses_received += 1;
    }

    // Acks
    pub fn inc_acks_sent(&mut self) {
        self.acks_sent += 1;
    }

    pub fn inc_acks_received(&mut self) {
        self.acks_received += 1;
    }

    // Nacks
    pub fn inc_nacks_received(&mut self) {
        self.nacks_received += 1;
    }
}

static STATS: LazyLock<Mutex<HashMap<NodeId, Stats>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Wrapper for safely interacting with the global stats
pub struct StatsManager;

impl StatsManager {
    pub fn get_or_create_stats(server_id: NodeId) -> Stats {
        let mut stats = STATS.lock().unwrap();
        stats.entry(server_id).or_insert_with(Stats::new).clone()
    }

    // Messages
    pub fn inc_messages_sent(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_messages_sent();
    }

    pub fn inc_messages_received(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_messages_received();
    }

    // Fragments
    pub fn inc_message_fragments_sent(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_message_fragments_sent();
    }

    pub fn inc_message_fragments_received(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_message_fragments_received();
    }

    // Flooding
    // Requests
    pub fn inc_flood_requests_sent(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_flood_requests_sent();
    }

    pub fn inc_flood_requests_received(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_flood_requests_received();
    }

    // Responses
    pub fn inc_flood_responses_sent(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_flood_responses_sent();
    }

    pub fn inc_flood_responses_received(server_id: NodeId) {
        let mut stats = STATS.lock().unwrap();
        let server_stats = stats.entry(server_id).or_insert_with(Stats::new);
        server_stats.inc_flood_responses_received();
    }

    // Acks
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
