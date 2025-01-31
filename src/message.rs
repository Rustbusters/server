use crate::db::DbMessage;
use common_utils::Stats;
use serde::{Deserialize, Serialize};
use wg_2024::network::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessage {
    GetServerMessages(NodeId),
}

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

pub enum InternalMessage {
    Stats(Stats),
    ServerMessages(ServerMessages),
}
