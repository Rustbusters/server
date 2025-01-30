use common_utils::Stats;
use serde::{Deserialize, Serialize};
use wg_2024::network::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessage {
    GetServerMessages(NodeId),
}
