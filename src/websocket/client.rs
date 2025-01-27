use tokio::task::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use std::thread;
use futures::{lock, SinkExt, StreamExt};
use futures::stream::{SplitSink, SplitStream};
use tokio::runtime::Runtime;
use tokio::net::{TcpListener, TcpStream};
use url::Url;
use crossbeam::channel::{unbounded, Receiver, Sender};
use tokio_tungstenite::{accept_async, connect_async};
use tokio_tungstenite::tungstenite::Message;

use std::error::Error;
use wg_2024::config::Server;
use wg_2024::network::NodeId;

use super::message::InternalMessage;

pub struct WebSocketClient;

impl WebSocketClient {
    pub async fn run(
        ws_url: String,
        tx: Sender<InternalMessage>,
        rx: Receiver<InternalMessage>,
    ) -> Result<(), Box<dyn Error>> {
        // Client will do the following:
        // 1. Connect to the WebSocket server.
        // 2. Listen for incoming tokio channels messages from the websocket server.
        // 3. Listen for incoming tokio channels messages from the network listener and forward them to the websocket server.

        // 1. Connect to the WebSocket server
        let (ws_stream, _) = connect_async(ws_url.as_str()).await?;
        println!("Connected to WebSocket server at {}", ws_url);
    
        // Split the WebSocket stream into sender and receiver
        let (mut ws_write, _) = ws_stream.split();
    
        // Forward messages from the channel to the WebSocket server
        // 3. Listen for incoming tokio channels messages from the network listener and forward them to the websocket server.
        while let Ok(message) = rx.recv() {
            println!("Message received: {:?}", message);
            match message {
                InternalMessage::FragmentReceived(id) => {
                    println!("[WS CLIENT]: Fragment received from network server {}", id);
                    if ws_write.send(Message::Text(id.to_string())).await.is_err() {
                        eprintln!("Failed to send message to WebSocket server");
                        break;
                    }
                },
                _ => {}
            }
        }
    
        // println!("WebSocket client terminated.");
        Ok(())
    }    
}