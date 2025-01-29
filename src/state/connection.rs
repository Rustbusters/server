use std::time::{Duration, Instant};
use wg_2024::config::Server;
use wg_2024::network::NodeId;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{Fragment, NodeType, Packet, PacketType};

use crate::websocket::message::{InternalMessage, WebSocketMessage};
use common_utils::{HostMessage, ServerToClientMessage, Stats, User};

use crossbeam_channel::{select_biased, unbounded, Receiver, RecvTimeoutError, Sender};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, LazyLock, Mutex};
use tungstenite::{Message, WebSocket};

static CONNECTIONS: LazyLock<Mutex<HashMap<NodeId, Connection>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct Connection {
    pub sender: Sender<Stats>,
    pub receiver: Receiver<Stats>,
}

impl Connection {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = unbounded::<Stats>();
        Self { sender, receiver }
    }
}

pub struct ConnectionsWrapper;

impl ConnectionsWrapper {
    pub fn is_empty() -> bool {
        let mut connections = CONNECTIONS.lock().unwrap();
        connections.is_empty()
    }

    pub fn send_stats(server_id: NodeId, stats: Stats) {
        println!("SENDING STATS: {stats:?}");
        let mut connections = CONNECTIONS.lock().unwrap();
        let conn = connections
            .get(&server_id)
            .expect("No connection found while sending stats");
        conn.sender.send(stats);
    }

    pub fn receive_and_forward_message(ws_stream: &mut WebSocket<TcpStream>) {
        let mut connections = CONNECTIONS.lock().unwrap();
        for (server_id, conn) in connections.iter() {
            if let Ok(msg) = conn.receiver.try_recv() {
                println!(
                    "Received message from network server on WEBSOCKET: {:?}",
                    msg
                );
                let ws_message = format!(
                    "{{\"message\":{}}}",
                    serde_json::to_string(&msg).expect("Should be serializable")
                );
                ws_stream.write(Message::Text(ws_message));
                ws_stream.flush();
            }
        }
    }

    pub fn add_connection(server_id: NodeId) {
        let mut connections = CONNECTIONS.lock().unwrap();
        connections.entry(server_id).or_insert_with(Connection::new);
    }
}
