use tokio::task::{self, JoinHandle};
use tokio::sync::Mutex;
use std::sync::{Arc};
use std::thread;
use tokio::runtime::Runtime;
use tokio::net::TcpListener;
use url::Url;
use wg_2024::config::Server;
use wg_2024::network::NodeId;

use std::collections::HashMap;
use common_utils::Stats;
use uuid::Uuid;

use super::db::DbManager;

// use futures_util::stream::{StreamExt, };
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use futures::SinkExt;

// Type aliases for convenience
type Writer = SplitSink<WebSocketStream<TcpStream>, Message>;

type ConnectionStore = Arc<Mutex<HashMap<Uuid, Writer>>>;

pub struct WebSocketServer {
    pub(crate) address: String,
    pub(crate) stats: Arc<Mutex<Stats>>,
    pub(crate) db_manager: DbManager,
    connections: ConnectionStore
}

impl WebSocketServer {
    pub fn new(address: String) -> Self {
        Self { 
            address, 
            stats: Arc::new(Mutex::new(Stats::new())), 
            db_manager: DbManager::new("rustbusters_db".to_string()),
            connections: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn run(self) {
        thread::spawn(move || {
            println!("[WS] Server running at ws://{}", self.address);
            let rt = Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async move {
                // Launch WebSocket server
                let handle = tokio::spawn(async move {
                    if let Err(e) = self.listen(self.address.clone()).await {
                        eprintln!("Error in WebSocket Server task: {}", e);
                    }
                });
    
                // Await server task
                handle.await.expect("Failed to join handle");
            });
        });
    }

    pub async fn listen(&self, address: String) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&address).await?;
        println!("[WS] Server running at ws://{}", address);

        // Accept incoming connections
        while let Ok((stream, _)) = listener.accept().await {
            let connections = self.connections.clone();
            let stats = self.stats.clone();

            tokio::spawn(async move {
                match accept_async(stream).await {
                    // Accept the WebSocket handshake
                    Ok(ws_stream) => {
                        println!("[WS] Connection established");
                        let client_id = Uuid::new_v4(); // Generate a unique UUID for each client
                        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                        let mut connections_lock = connections.lock().await;
                        connections_lock.insert(client_id, ws_sender);
 
                        // Handle incoming messages from the client
                        while let Some(Ok(msg)) = ws_receiver.next().await {
                            if let Message::Text(text) = msg {
                                println!("[WS] Received: {}", text);

                                let mut stats_lock = stats.lock().await;
                                stats_lock.inc_messages_received();

                                let stats_json = serde_json::to_string(&*stats_lock).expect("Failed to serialize stats");

                                // Send message to all connected clients
                                for (id, writer) in connections_lock.iter_mut() {
                                    writer.send(Message::Text(stats_json.clone())).await.expect("Failed to send message");
                                    println!("[WS] Sending stats to client {}", id);
                                }
                            }
                        }
                    }
                    //Reject the connection
                    Err(e) => eprintln!("WebSocket handshake failed: {}", e),
                }
            });
        }

        Ok(())
    }
}