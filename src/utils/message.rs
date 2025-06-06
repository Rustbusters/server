use crate::server::db::DbMessage;
use crate::state::Stats;
use common_utils::User;
use serde::{Deserialize, Serialize};
use wg_2024::network::NodeId;

// This module defines the message types used for communication between the Network Listener and the WebSocket Server. These messages are exchanged via crossbeam channels to facilitate real-time updates during the simulation.

/// WebSocket Messages
/// This message is sent as a request from the HTTP server to the WebSocket Server through a crossbeam channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketRequest {
    GetStats,
    GetMessages,
    GetActiveUsers,
}

/// Internal Server Messages
/// This message is exchanged between the Network Server and the WebSocket Server through a crossbeam channel.
pub enum InternalMessage {
    SendStats(Stats),
    SendServerMessage(ServerMessage),
    SendServerMessages(ServerMessages),
    SendActiveUsers(ActiveUsers),
}

/// Server Message
/// Wrapper for the users' message on a specific server: this is used to update the users' message during the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMessage {
    pub(crate) server_id: NodeId,
    pub(crate) message: DbMessage,
}

impl ServerMessage {
    pub fn new(server_id: NodeId, message: DbMessage) -> Self {
        Self { server_id, message }
    }
}

/// Server Messages
/// Wrapper for the users' messages on a specific server
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

/// Active Users
/// Wrapper for the active user on a specific server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveUsers {
    pub(crate) server_id: NodeId,
    pub(crate) active_users: Vec<User>,
}

impl ActiveUsers {
    pub(crate) fn new(server_id: NodeId, active_users: Vec<User>) -> Self {
        Self {
            server_id,
            active_users,
        }
    }
}
