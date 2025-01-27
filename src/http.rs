use std::{io::Error, thread};
use std::str::FromStr;
use tiny_http::{Header, Method, Request, Response, StatusCode};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use tokio_stream::StreamExt;
use std::fs;
use log::info;
use wg_2024::config::Server;

// pub struct RustBustersServerUI {
//     ip_addr: String,
//     port: u16,
//     servers: Vec<Server>,
//     websocket_server_address: String,
//     http_server_address: String,
// }

// impl RustBustersServerUI {
//     pub fn new(ip_addr: &str, port: u16, servers: Vec<Server>) -> Self {
//         let ip_addr = ip_addr.to_string();
//         let websocket_server_address = String::from(format!("{}:{}", ip_addr.clone(), port));
//         let http_server_address = String::from(format!("{}:{}", ip_addr.clone(), port + 1));

//         Self { ip_addr, port, servers, websocket_server_address, http_server_address }
//     }
    
//     pub fn run(self) {
//         // 1. Create WebSocketController that will spawn the server + clients
//         let websocket_controller = WebSocketController::new(self.websocket_server_address, self.servers.clone());
//         websocket_controller.run();

//         // 2. Create and launch HTTP Server for hosting UI
//         let http_server = HttpServer::new(self.http_server_address.clone(), "static/server/emeliyanov");
//         http_server.run();
//     }
// }

pub struct HttpServer {
    address: String,
    routes: Vec<String>,
    public_path: String
}

impl HttpServer {
    pub fn new(address: String, public_path: String) -> Self {
        let mut routes = Vec::new();
        routes.push(String::from("dashboard"));
        routes.push(String::from("servers"));
        routes.push(String::from("messages"));
        Self { address, routes, public_path }
    }

    pub fn run(self) {
        thread::spawn(move || {
            println!("[HTTP] Server running at http://{}", self.address);
            let http_server = tiny_http::Server::http(self.address.clone()).unwrap();
            loop {
                if let Ok(Some(request)) = http_server.try_recv() {
                    match self.handle_request(request) {
                        Ok(()) => {}
                        Err(e) => eprintln!("Error handling request: {e}"),
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
            // Description: serve static content
            (Method::Get, path) if path.starts_with('/') => {
                let sanitized_path = String::from(&path[1..]); // Remove initial slash '/'
                match fs::read(format!("{}/{}", self.public_path, sanitized_path)) {
                    Ok(content) => {
                        Response::from_data(content).with_header(
                            Header::from_str(&format!(
                                "Content-Type: {}",
                                self.get_mime_type(sanitized_path.as_str())
                            ))
                            .unwrap(),
                        )
                    }
                    Err(err) => {
                        let file = fs::read_to_string(format!("{}/index.html", self.public_path))
                            .unwrap_or("DEFAULT_HTML".to_string());
                        Response::from_string(file)
                            .with_header(Header::from_str("Content-Type: text/html").unwrap())
                    }
                }
            } 
            // @Post Method
            // Description: 
            (Method::Post, "/api/send-to") => {
                let mut body = String::new();
                req.as_reader()
                    .read_to_string(&mut body)
                    .unwrap_or_else(|_| {
                        println!("Failed to read request body");
                        0
                    });

                // println!("POST request body: {}", body);

                Response::from_string("[HTTP] @POST request received")
            }
            // @Put Method
            (Method::Put, "/api") => {
                // println!("PUT request received");
                Response::from_string("[HTTP] @PUT request received")
            }
            // @Delete Method
            (Method::Delete, "/api") => {
                // println!("DELETE request received");
                Response::from_string("[HTTP] @DELETE request received")
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
