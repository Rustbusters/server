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

pub struct WebSocketController {
    address: String,
    servers: Vec<Server>,
}

impl WebSocketController {
    pub fn new(address: String, servers: Vec<Server>) -> Self {
        Self { address, servers }
    }

    pub fn run(&self) {
        let server_address = self.address.clone();
        let servers = self.servers.clone();
        let server_url = format!("ws://{}", server_address.clone());

        thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create Tokio runtime");

            rt.block_on(async move {
                // Launch WebSocket server
                let server_task = task::spawn(async move {
                    if let Err(e) = run_websocket_server(server_address.clone()).await {
                        eprintln!("Error in WebSocket Server task: {}", e);
                    }
                });

                // Launch WebSocket clients
                let mut client_tasks = Vec::<JoinHandle<_>>::new();
                for server in servers {
                    let server_url: String = server_url.clone();
                    let handle = task::spawn(async move {
                        if let Err(e) = run_websocket_client(server_url, server.id).await {
                            eprintln!("Error in WebSocket Client task: {}", e);
                        }
                    });
                    client_tasks.push(handle);
                }

                // Await server task
                if let Err(e) = server_task.await {
                    eprintln!("WebSocket server error: {}", e);
                }

                // Await all client tasks
                for client_task in client_tasks {
                    if let Err(e) = client_task.await {
                        eprintln!("WebSocket client task error: {}", e);
                    }
                }
            });
        });
    }
}

pub struct WebSocketServer;

impl WebSocketServer {
    pub async fn run(address: String) -> Result<(), Box<dyn std::error::Error>> {
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

pub struct WebSocketClient;

impl WebSocketClient {
    pub async fn run(server_url: String, client_id: NodeId) -> Result<(), Box<dyn std::error::Error>> {
        println!("Connecting to: {}", server_url);
        let (ws_stream, _) = connect_async(&server_url).await.expect("Failed to connect");
        println!("WebSocket Client {} connected to ws://{}", client_id, server_url);

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

async fn run_websocket_server(address: String) -> Result<(), Box<dyn std::error::Error>> {
    WebSocketServer::run(address).await
}

async fn run_websocket_client(server_url: String, client_id: NodeId) -> Result<(), Box<dyn std::error::Error>> {
    WebSocketClient::run(server_url, client_id).await
}
