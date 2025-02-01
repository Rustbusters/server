use common_utils::message;
use serde::Deserialize;
use serde::Serialize;
use std::time::{Duration, Instant};
use wg_2024::config::Server;
use wg_2024::network::NodeId;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};

use crate::server::db::DbMessage;
use crate::utils::message::{InternalMessage, ServerMessages, WebSocketMessage};
use common_utils::{HostMessage, ServerToClientMessage, Stats, User};

use crossbeam_channel::{select_biased, unbounded, Receiver, RecvTimeoutError, Sender};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, LazyLock, Mutex};
use tungstenite::{Message, WebSocket};

// Internal Channels: communication between the Network Server and the WebSocket Server
static INTERNAL_CHANNELS: LazyLock<Mutex<HashMap<NodeId, InternalChannel>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
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

pub struct InternalChannelsManager;

impl InternalChannelsManager {
    pub fn get_servers() -> Vec<NodeId> {
        let connections = INTERNAL_CHANNELS.lock().unwrap();
        let servers = connections.keys().copied().collect();
        servers
    }

    pub fn is_empty() -> bool {
        let mut connections = INTERNAL_CHANNELS.lock().unwrap();
        connections.is_empty()
    }

    pub fn send_stats(server_id: NodeId, stats: Stats) {
        let mut connections = INTERNAL_CHANNELS.lock().unwrap();
        let conn = connections
            .get(&server_id)
            .expect("No connection found while sending stats");
        conn.sender.send(InternalMessage::Stats(stats));
    }

    pub fn send_server_messages(server_id: NodeId, messages: Vec<DbMessage>) {
        let mut connections = INTERNAL_CHANNELS.lock().unwrap();
        let conn = connections
            .get(&server_id)
            .expect("No connection found while sending stats");
        let server_message = ServerMessages::new(server_id, messages);
        conn.sender
            .send(InternalMessage::ServerMessages(server_message));
    }

    pub fn receive_and_forward_message(ws_stream: &mut WebSocket<TcpStream>) {
        let mut connections = INTERNAL_CHANNELS.lock().unwrap();
        for (server_id, conn) in connections.iter() {
            // println!("Cycling on INTERNAL_CHANNELS");
            while let Ok(message) = conn.receiver.try_recv() {
                // println!("Trying to receive on Server {server_id}");
                match message {
                    InternalMessage::Stats(stats) => {
                        // println!("Received Stats on Server {server_id}");
                        let ws_message = format!(
                            "{{\"server_id\":{server_id},\"stats\":{}}}",
                            serde_json::to_string(&stats).expect("Should be serializable")
                        );
                        ws_stream.write(Message::Text(ws_message));
                        ws_stream.flush();
                    }
                    InternalMessage::ServerMessages(messages) => {
                        // println!("Received ServerMessages on Server {server_id}");
                        let ws_message = format!(
                            "{{\"server_id\":{},\"messages\":{}}}",
                            messages.server_id,
                            serde_json::to_string(&messages.messages)
                                .expect("Should be serializable")
                        );
                        println!("Sending messages to websocket client: {ws_message}");
                        ws_stream.write(Message::Text(ws_message));
                        ws_stream.flush();
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn add_channel(server_id: NodeId) {
        let mut connections = INTERNAL_CHANNELS.lock().unwrap();
        connections
            .entry(server_id)
            .or_insert_with(InternalChannel::new);
    }
}

// WebSocket Message Channels
static WS_CHANNELS: LazyLock<Mutex<HashMap<NodeId, Sender<WebSocketMessage>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct WSChannelsManager;

impl WSChannelsManager {
    pub fn get_messages(server_id: NodeId) {
        // Send message to ws_channel
        let ws_channels = WS_CHANNELS.lock().unwrap();
        let channel = ws_channels
            .get(&server_id)
            .expect("No channel found while retrieving messages");
        channel.send(WebSocketMessage::GetServerMessages(server_id));
    }

    pub fn add_channel(server_id: NodeId) -> Receiver<WebSocketMessage> {
        let (sender, receiver) = unbounded::<WebSocketMessage>();
        let mut ws_channels = WS_CHANNELS.lock().unwrap();
        ws_channels.entry(server_id).or_insert(sender);
        receiver
    }
}
