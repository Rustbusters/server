use crate::server::db::DbMessage;
use common_utils::Stats;
use serde::{Deserialize, Serialize};
use wg_2024::network::NodeId;

// This file specifies the types of messages exchanged by some server components

/// WebSocket Messages
/// This message is sent as a request from the HTTP server to the single Server channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessage {
    GetServerMessages(NodeId),
}

/// Internal Server Messages
/// This message is exchanged between the Network Server and the WebSocket Server.
pub enum InternalMessage {
    Stats(Stats),
    ServerMessages(ServerMessages),
}

/// Server Messages
/// This message is sent from the Network Server to the WebSocket Server for some specific server info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMessages {
    pub(crate) server_id: NodeId,
    pub(crate) messages: Vec<DbMessage>,
}

impl ServerMessages {
    pub fn new(server_id: NodeId, messages: Vec<DbMessage>) -> Self {
        Self {
            server_id,
            messages,
        }
    }
}
