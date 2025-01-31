use crate::ConnectionsWrapper;
use log::info;
use serde_json::json;
use std::fs;
use std::str::FromStr;
use std::{io::Error, thread};
use tiny_http::{Header, Method, Request, Response, StatusCode};
use wg_2024::config::Server;
use wg_2024::network::NodeId;

pub struct HttpServer {
    address: String,
    routes: Vec<String>,
    public_path: String,
}

impl HttpServer {
    pub fn new(address: String, public_path: String) -> Self {
        let mut routes = Vec::new();
        routes.push(String::from("dashboard"));
        // routes.push(String::from("servers"));
        routes.push(String::from("messages"));
        Self {
            address,
            routes,
            public_path,
        }
    }

    pub fn run(self) {
        thread::spawn(move || {
            println!("[HTTP] Server running at http://{}", self.address);
            let http_server = tiny_http::Server::http(self.address.clone()).unwrap();
            loop {
                if let Ok(Some(request)) = http_server.try_recv() {
                    if let Err(e) = self.handle_request(request) {
                        eprintln!("Error handling request: {e}");
                    }
                }
            }
        });
    }

    fn handle_request(&self, mut req: Request) -> Result<(), Error> {
        let method = req.method();
        let url = req.url();
        // println!("[HTTP] @GET Received request: {method} {url}");

        let response = match (method, url) {
            // @ GET method
            // Description: serve index.html as root file
            (Method::Get, "/") => {
                let file: String = fs::read_to_string(format!("{}/index.html", self.public_path))
                    .unwrap_or("DEFAULT_HTML".to_string());
                Response::from_string(file)
                    .with_header(Header::from_str("Content-Type: text/html").unwrap())
            }
            // @Get Method
            // Description:
            (Method::Get, path) if path.starts_with("/api/servers") => {
                let parts: Vec<&str> = url.split('/').collect();

                // Check if the URL matches the pattern "/api/servers/:server_id"
                if parts.len() == 4 && parts[1] == "api" && parts[2] == "servers" {
                    let server_id = parts[3].parse::<NodeId>().unwrap(); // Extract server_id

                    // Fetch server details based on server_id
                    let _ = ConnectionsWrapper::get_messages(server_id);
                    let json_response =
                        json!({ "status": "success", "message": "Waiting for messages..." })
                            .to_string();

                    Response::from_string(json_response)
                        .with_header(Header::from_str("Content-Type: application/json").unwrap())
                        .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap())
                } else {
                    let servers = ConnectionsWrapper::get_servers();
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
            // Description: serve static content
            (Method::Get, path) if path.starts_with('/') => {
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
            // Undefined route
            _ => {
                let response = Response::from_string("[HTTP] @GET 404 Not Found");
                response.with_status_code(404)
            }
        };

        req.respond(response)
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
