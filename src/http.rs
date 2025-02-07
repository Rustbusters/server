use crate::controller::InternalCommand;
use crate::utils::traits::{Runnable, Service};
use crate::{InternalChannelsManager, WSChannelsManager};
use crossbeam_channel::Receiver;
use log::info;
use serde_json::json;
use std::fs;
use std::str::FromStr;
use std::thread::JoinHandle;
use std::{io::Error, thread};
use tiny_http::{Header, Method, Request, Response, StatusCode};
use wg_2024::config::Server;
use wg_2024::network::NodeId;

pub struct HttpServer {
    address: String,
    public_path: String,
    internal_command_receiver: Receiver<InternalCommand>,
}

impl Runnable for HttpServer {
    fn run(self) -> Option<JoinHandle<()>> {
        let handle = thread::spawn(move || {
            self.start();
        });
        Some(handle)
    }
}

impl Service for HttpServer {
    fn start(self) {
        info!(
            "[SERVER-HTTP] Visit http://{} for the server UI",
            self.address
        );
        println!(
            "[SERVER-HTTP] Visit http://{} for the server UI",
            self.address
        );
        let http_server = tiny_http::Server::http(self.address.clone()).unwrap();
        loop {
            if let Ok(Some(request)) = http_server.try_recv() {
                if let Err(e) = self.handle_request(request) {
                    eprintln!("Error handling request: {e}");
                }
            }

            if let Ok(internal_command) = self.internal_command_receiver.try_recv() {
                match internal_command {
                    InternalCommand::Stop => {
                        info!("[SERVER-HTTP] Terminating HTTP server");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

type ResponseType = Response<std::io::Cursor<Vec<u8>>>;

impl HttpServer {
    pub fn new(
        address: String,
        public_path: String,
        internal_command_receiver: Receiver<InternalCommand>,
    ) -> Self {
        Self {
            address,
            public_path,
            internal_command_receiver,
        }
    }

    fn handle_request(&self, mut req: Request) -> Result<(), Error> {
        let method = req.method();
        let url = req.url();
        info!("[SERVER-HTTP] @GET Received request: {method} {url}");

        let response = match (method, url) {
            // @ GET method
            // Description: serve index.html as root file
            (Method::Get, "/") => self.handle_main(),
            // @Get Method
            // Description: apis for retrieving some server information
            (Method::Get, path) if path.starts_with("/api/servers") => self.handle_server_apis(url),
            // @Get Method
            // Description: serve static content
            (Method::Get, path) if path.starts_with('/') => self.handle_static_files(path),
            // Undefined route
            _ => self.handle_wrong_path(),
        };

        req.respond(response)
    }

    fn handle_main(&self) -> ResponseType {
        let file: String = fs::read_to_string(format!("{}/index.html", self.public_path))
            .unwrap_or("DEFAULT_HTML".to_string());
        Response::from_string(file)
            .with_header(Header::from_str("Content-Type: text/html").unwrap())
    }

    fn handle_server_apis(&self, url: &str) -> ResponseType {
        let parts: Vec<&str> = url.split('/').collect();

        // Check the URL
        if parts.len() == 5 && parts[1] == "api" && parts[2] == "servers" {
            let server_id = parts[4].parse::<NodeId>().unwrap(); // Extract server_id

            if parts[3] == "stats" {
                // URL matches the pattern "/api/servers/stats/:server_id"
                // Fetch server stats based on server_id
                WSChannelsManager::get_server_stats(server_id);

                let json_response =
                    json!({ "status": "success", "message": "Waiting for stats..." }).to_string();

                Response::from_string(json_response)
                    .with_header(Header::from_str("Content-Type: application/json").unwrap())
                    .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap())
            } else if parts[3] == "messages" {
                // URL matches the pattern "/api/servers/messages/:server_id"
                // Fetch server messages based on server_id
                WSChannelsManager::get_server_messages(server_id);
                let json_response =
                    json!({ "status": "success", "message": "Waiting for messages..." })
                        .to_string();

                Response::from_string(json_response)
                    .with_header(Header::from_str("Content-Type: application/json").unwrap())
                    .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap())
            } else if parts[3] == "users" {
                // URL matches the pattern "/api/servers/users/:server_id"
                // Fetch server active users based on server_id
                WSChannelsManager::get_server_active_users(server_id);
                let json_response =
                    json!({ "status": "success", "message": "Waiting for active users..." })
                        .to_string();

                Response::from_string(json_response)
                    .with_header(Header::from_str("Content-Type: application/json").unwrap())
                    .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap())
            } else {
                // URL doesn't match anything of the above
                let json_response =
                    json!({ "status": "failure", "message": "Wrong url provided" }).to_string();
                Response::from_string(json_response)
                    .with_header(Header::from_str("Content-Type: application/json").unwrap())
                    .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap())
            }
        } else {
            // Fetch list of servers on the network
            let servers = InternalChannelsManager::get_servers();
            // Serialize the servers to JSON
            let json_data = serde_json::to_string(&servers).unwrap();

            // Create the final JSON object with the "servers" key
            let json_response = format!("{{\"servers\": {}}}", json_data);

            // Return the response with the proper Content-Type header
            Response::from_string(json_response)
                .with_header(Header::from_str("Content-Type: application/json").unwrap())
                .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap())
        }
    }

    fn handle_static_files(&self, path: &str) -> ResponseType {
        let sanitized_path = String::from(&path[1..]); // Remove initial slash '/'
        match fs::read(format!("{}/{}", self.public_path, sanitized_path)) {
            Ok(content) => Response::from_data(content).with_header(
                Header::from_str(&format!(
                    "Content-Type: {}",
                    self.get_mime_type(sanitized_path.as_str())
                ))
                .unwrap(),
            ),
            Err(err) => {
                let file = fs::read_to_string(format!("{}/index.html", self.public_path))
                    .unwrap_or("DEFAULT_HTML".to_string());
                Response::from_string(file)
                    .with_header(Header::from_str("Content-Type: text/html").unwrap())
            }
        }
    }

    fn handle_wrong_path(&self) -> ResponseType {
        let response = Response::from_string("[SERVER-HTTP] @GET 404 Not Found");
        response.with_status_code(404)
    }

    fn get_mime_type(&self, path: &str) -> &'static str {
        if path.ends_with(".html") {
            "text/html"
        } else if path.ends_with(".css") {
            "text/css"
        } else if path.ends_with(".js") {
            "application/javascript"
        } else if path.ends_with(".png") {
            "image/png"
        } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
            "image/jpeg"
        } else if path.ends_with(".ico") {
            "image/x-icon"
        } else if path.ends_with(".svg") {
            "image/svg+xml"
        } else if path.ends_with(".gif") {
            "image/gif"
        } else if path.ends_with(".mp4") {
            "video/mp4"
        } else if path.ends_with(".webm") {
            "video/webm"
        } else if path.ends_with(".ogg") {
            "video/ogg"
        } else if path.ends_with(".avi") {
            "video/x-msvideo"
        } else if path.ends_with(".mpeg") {
            "video/mpeg"
        } else {
            "application/octet-stream"
        }
    }
}
