use crate::state::Stats;
use common_utils::message;
use serde::Deserialize;
use serde::Serialize;
use std::time::{Duration, Instant};
use tungstenite::handshake::server;
use wg_2024::config::Server;
use wg_2024::network::NodeId;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};

use crate::server::db::DbMessage;
use crate::utils::message::ActiveUsers;
use crate::utils::message::{InternalMessage, ServerMessage, ServerMessages, WebSocketRequest};
use common_utils::{HostMessage, ServerToClientMessage, User};

use crossbeam_channel::{select_biased, unbounded, Receiver, RecvTimeoutError, Sender};
use log::info;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, LazyLock, Mutex};
use tungstenite::{Message, WebSocket};

// Internal Channels: communication between the Network Server and the WebSocket Server
static INTERNAL_CHANNELS: LazyLock<Mutex<HashMap<NodeId, InternalChannel>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
struct InternalChannel {
    pub sender: Sender<InternalMessage>,
    pub receiver: Receiver<InternalMessage>,
}

impl InternalChannel {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = unbounded::<InternalMessage>();
        Self { sender, receiver }
    }
}

// Struct for managing internal channels between the WebSocket Server and the various Network Servers
pub struct InternalChannelsManager;

impl InternalChannelsManager {
    pub fn get_servers() -> Vec<NodeId> {
        let channels = INTERNAL_CHANNELS.lock().unwrap();
        let servers = channels.keys().copied().collect();
        servers
    }

    pub fn is_empty() -> bool {
        let mut channels = INTERNAL_CHANNELS.lock().unwrap();
        channels.is_empty()
    }

    pub fn send_stats(server_id: NodeId, stats: Stats) {
        let mut channels = INTERNAL_CHANNELS.lock().unwrap();
        let conn = channels
            .get(&server_id)
            .expect("No connection found while sending stats");
        conn.sender.send(InternalMessage::SendStats(stats));
    }

    pub fn send_message(server_id: NodeId, message: DbMessage) {
        let mut channels = INTERNAL_CHANNELS.lock().unwrap();
        let conn = channels
            .get(&server_id)
            .expect("No connection found while sending server messages");
        let server_message = ServerMessage::new(server_id, message);
        conn.sender
            .send(InternalMessage::SendServerMessage(server_message));
    }

    pub fn send_messages(server_id: NodeId, messages: Vec<DbMessage>) {
        let mut channels = INTERNAL_CHANNELS.lock().unwrap();
        let conn = channels
            .get(&server_id)
            .expect("No connection found while sending server messages");
        let server_message = ServerMessages::new(server_id, messages);
        conn.sender
            .send(InternalMessage::SendServerMessages(server_message));
    }

    pub fn send_active_users(server_id: NodeId, active_users: Vec<User>) {
        let mut channels = INTERNAL_CHANNELS.lock().unwrap();
        let conn = channels
            .get(&server_id)
            .expect("Not connection found while sending active users");
        let active_users_message = ActiveUsers::new(server_id, active_users);
        conn.sender
            .send(InternalMessage::SendActiveUsers(active_users_message));
    }

    pub fn receive_and_forward_message(ws_stream: &mut WebSocket<TcpStream>) {
        let mut channels = INTERNAL_CHANNELS.lock().unwrap();
        for (&server_id, conn) in channels.iter() {
            while let Ok(message) = conn.receiver.try_recv() {
                let ws_message = Self::handle_internal_message(server_id, message);
                ws_stream.write(Message::Text(ws_message));
                ws_stream.flush();
            }
        }
    }

    fn handle_internal_message(server_id: NodeId, message: InternalMessage) -> String {
        let mut ws_message = String::from("Easter egg 🐣: quack 🦆");
        match message {
            InternalMessage::SendStats(stats) => {
                ws_message = format!(
                    "{{\"serverId\":{server_id},\"stats\":{}}}",
                    serde_json::to_string(&stats).expect("Should be serializable")
                );
            }
            InternalMessage::SendServerMessage(server_message) => {
                ws_message = format!(
                    "{{\"serverId\":{},\"newMessage\":{}}}",
                    server_message.server_id,
                    serde_json::to_string(&server_message.message).expect("Should be serializable")
                );
            }
            InternalMessage::SendServerMessages(server_messages) => {
                ws_message = format!(
                    "{{\"serverId\":{},\"messages\":{}}}",
                    server_messages.server_id,
                    serde_json::to_string(&server_messages.messages)
                        .expect("Should be serializable")
                );
            }
            InternalMessage::SendActiveUsers(active_users) => {
                ws_message = format!(
                    "{{\"serverId\":{},\"activeUsers\":{}}}",
                    active_users.server_id,
                    serde_json::to_string(&active_users.active_users)
                        .expect("Should be serializable")
                );
            }
            _ => {}
        }
        ws_message
    }

    pub fn add_channel(server_id: NodeId) {
        let mut channels = INTERNAL_CHANNELS.lock().unwrap();
        channels
            .entry(server_id)
            .or_insert_with(InternalChannel::new);
    }

    pub fn remove_channels() {
        let mut channels = INTERNAL_CHANNELS.lock().unwrap();
        channels.clear();
    }
}

// WebSocket Message Channels
static WS_CHANNELS: LazyLock<Mutex<HashMap<NodeId, Sender<WebSocketRequest>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct WSChannelsManager;

impl WSChannelsManager {
    pub fn get_server_stats(server_id: NodeId) {
        // Send message to Internal channel of the specified Network Server
        let ws_channels = WS_CHANNELS.lock().unwrap();
        let channel = ws_channels
            .get(&server_id)
            .expect("No channel found while retrieving stats");
        channel.send(WebSocketRequest::GetStats);
    }

    pub fn get_server_messages(server_id: NodeId) {
        // Send message to Internal channel of the specified Network Server
        let ws_channels = WS_CHANNELS.lock().unwrap();
        let channel = ws_channels
            .get(&server_id)
            .expect("No channel found while retrieving messages");
        channel.send(WebSocketRequest::GetMessages);
    }

    pub fn get_server_active_users(server_id: NodeId) {
        // Send message to Internal channel of the specified Network Server
        let ws_channels = WS_CHANNELS.lock().unwrap();
        let channel = ws_channels
            .get(&server_id)
            .expect("No channel found while retrieving messages");
        channel.send(WebSocketRequest::GetActiveUsers);
    }

    pub fn add_channel(server_id: NodeId) -> Receiver<WebSocketRequest> {
        // Adds a channels channel for the specified server_id
        let (sender, receiver) = unbounded::<WebSocketRequest>();
        let mut ws_channels = WS_CHANNELS.lock().unwrap();
        ws_channels.insert(server_id, sender);
        receiver
    }

    pub fn remove_channels() {
        // Removes all the channels, can be use in case of a Stop command from the simulation controller
        let mut channels = INTERNAL_CHANNELS.lock().unwrap();
        channels.clear();
    }
}
