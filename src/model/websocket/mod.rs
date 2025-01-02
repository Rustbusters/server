use tokio::task::{self, JoinHandle};
use tokio_tungstenite::tungstenite::handshake::server;
use std::sync::{Arc, Mutex};
use std::thread;
use futures::{SinkExt, StreamExt};
use tokio::runtime::Runtime;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, connect_async};
use tokio_tungstenite::tungstenite::Message;
use url::Url;
use wg_2024::config::Server;
use wg_2024::network::NodeId;

mod message;

pub struct WebSocketServer {
    pub(crate) address: String,
}

impl WebSocketServer {
    pub fn new(address: String) -> Self {
        Self { address }
    }

    pub async fn run(&self, address: String) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&address).await?;
        println!("WebSocket Server running at ws://{}", address);

        while let Ok((stream, _)) = listener.accept().await {
            println!("New connection!");

            tokio::spawn(async move {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        println!("WebSocket connection established!");
                        let (mut _ws_sender, mut ws_receiver) = ws_stream.split();

                        while let Some(Ok(msg)) = ws_receiver.next().await {
                            if let Message::Text(text) = msg {
                                println!("Received: {}", text);
                            }
                        }
                    }
                    Err(e) => eprintln!("WebSocket handshake failed: {}", e),
                }
            });
        }

        Ok(())
    }
}

pub struct WebSocketClient { 
    pub(crate) address: String,
}

impl WebSocketClient {
    pub fn new(address: String) -> Self {
        Self { address }
    }

    pub async fn connect_to_websocket_server(&self, client_id: NodeId) -> Result<(), Box<dyn std::error::Error>> {
        let (ws_stream, _) = connect_async(&self.address).await.expect("Failed to connect");
        println!("WebSocket Client {} connected to ws://{}", client_id, self.address);

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Send a message from the client
        let message = format!("Hello from client {}", client_id);
        ws_sender.send(Message::Text(message)).await?;

        // Receive messages
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => println!("Client {}: Received: {}", client_id, text),
                Ok(_) => println!("Client {}: Received a non-text message", client_id),
                Err(e) => {
                    eprintln!("Client {}: Error receiving message: {}", client_id, e);
                    break;
                }
            }
        }

        Ok(())
    }
}