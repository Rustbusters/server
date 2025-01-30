use serde::Deserialize;
use serde::Serialize;
use std::time::{Duration, Instant};
use wg_2024::config::Server;
use wg_2024::network::NodeId;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};

use crate::db::DbMessage;
use crate::websocket::message::WebSocketMessage;
use common_utils::{HostMessage, ServerToClientMessage, Stats, User};

use crossbeam_channel::{select_biased, unbounded, Receiver, RecvTimeoutError, Sender};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, LazyLock, Mutex};
use tungstenite::{Message, WebSocket};

static WS_CHANNELS: LazyLock<Mutex<HashMap<NodeId, Sender<WebSocketMessage>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static CONNECTIONS: LazyLock<Mutex<HashMap<NodeId, Connection>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMessages {
    server_id: NodeId,
    messages: Vec<DbMessage>,
}

pub enum InternalMessage {
    Stats(Stats),
    ServerMessages(ServerMessages),
}

pub struct Connection {
    pub sender: Sender<InternalMessage>,
    pub receiver: Receiver<InternalMessage>,
}

impl Connection {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = unbounded::<InternalMessage>();
        Self { sender, receiver }
    }
}

pub struct ConnectionsWrapper;

impl ConnectionsWrapper {
    pub fn get_servers() -> Vec<NodeId> {
        let connections = CONNECTIONS.lock().unwrap();
        let servers = connections.keys().copied().collect();
        servers
    }

    pub fn get_messages(server_id: NodeId) -> Vec<DbMessage> {
        todo!()
    }

    pub fn is_empty() -> bool {
        let mut connections = CONNECTIONS.lock().unwrap();
        connections.is_empty()
    }

    pub fn send_stats(server_id: NodeId, stats: Stats) {
        let mut connections = CONNECTIONS.lock().unwrap();
        let conn = connections
            .get(&server_id)
            .expect("No connection found while sending stats");
        conn.sender.send(InternalMessage::Stats(stats));
    }

    pub fn send_server_messages(server_id: NodeId, messages: Vec<DbMessage>) {
        let mut connections = CONNECTIONS.lock().unwrap();
        let conn = connections
            .get(&server_id)
            .expect("No connection found while sending stats");
        let server_message = ServerMessages {
            server_id,
            messages,
        };
        conn.sender
            .send(InternalMessage::ServerMessages(server_message));
    }

    pub fn receive_and_forward_message(ws_stream: &mut WebSocket<TcpStream>) {
        let mut connections = CONNECTIONS.lock().unwrap();
        for (server_id, conn) in connections.iter() {
            if let Ok(message) = conn.receiver.try_recv() {
                // TODO: add multiple message handling
                match message {
                    InternalMessage::Stats(stats) => {
                        let ws_message = format!(
                            "{{\"server_id\":{server_id},\"stats\":{}}}",
                            serde_json::to_string(&stats).expect("Should be serializable")
                        );
                        ws_stream.write(Message::Text(ws_message));
                        ws_stream.flush();
                    }
                    InternalMessage::ServerMessages(messages) => {
                        let ws_message = format!(
                            "{{\"server_id\":{},\"messages\":{}}}",
                            messages.server_id,
                            serde_json::to_string(&messages.messages)
                                .expect("Should be serializable")
                        );
                        ws_stream.write(Message::Text(ws_message));
                        ws_stream.flush();
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn add_connection(server_id: NodeId) {
        let mut connections = CONNECTIONS.lock().unwrap();
        connections.entry(server_id).or_insert_with(Connection::new);
    }

    pub fn add_ws_channel(server_id: NodeId) -> Receiver<WebSocketMessage> {
        let (sender, receiver) = unbounded::<WebSocketMessage>();
        let mut ws_channels = WS_CHANNELS.lock().unwrap();
        ws_channels.entry(server_id).or_insert(sender);
        receiver
    }
}
